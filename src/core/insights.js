// ============================================================
// Insights: instruções individuais recomendadas, relatório do
// adversário, suplentes e sucessores. Puro JS, testável em Node.
// ============================================================

const A_ = (p, k) => (p && p.a[k] != null ? p.a[k] : null);
const avgOf = (p, ks) => { const v = ks.map(k => A_(p, k)).filter(x => x != null); return v.length ? v.reduce((a, b) => a + b, 0) / v.length : null; };
const avgPlayers = (ps, ks) => { const v = ps.map(p => avgOf(p, ks)).filter(x => x != null); return v.length ? v.reduce((a, b) => a + b, 0) / v.length : null; };

// ---------- Instruções individuais recomendadas ----------
// Cada regra: condição sobre atributos + papel → instrução + porquê
function recommendPIs(p, ipRole, oopRole, group) {
  if (!p) return [];
  const out = [], add = (pi, why) => { if (PLAYER_INSTRUCTIONS.some(x => x.n === pi)) out.push({ pi, why }); };
  const a = (k) => A_(p, k) ?? 0, tags = ipRole ? ipRole.tags : [];
  const attacker = ['ST','AMC','AM_LR','M_LR'].includes(group);
  if (group === 'GK') return out;
  if (attacker && (a('pac') >= 15 || a('acc') >= 15) && a('otb') >= 13) add('Attack the Defensive Line', `Velocidade ${Math.max(a('pac'), a('acc'))} e Sem Bola ${a('otb')}: ganha nas costas da defesa.`);
  if (group === 'ST' && a('pac') >= 14 && a('otb') >= 12 && !tags.includes('DROP')) add('Move into channels', 'Rápido e com bom movimento: arrasta os centrais para os corredores.');
  if (['DM','MC','AMC'].includes(group) && a('otb') >= 13 && tags.some(t => ['ADV','BOX','RUN'].includes(t))) add('Arrive Late', `Sem Bola ${a('otb')}: entra na área em segunda vaga, quando a defesa já está presa.`);
  if (a('lon') >= 15 && a('tec') >= 13) add('Shoot more', `Remates de Longe ${a('lon')}: vale a pena arriscar de fora da área.`);
  else if (attacker && a('fin') && a('fin') <= 10 && a('lon') <= 11) add('Shoot less', `Finalização ${a('fin')} e Remates de Longe ${a('lon')}: melhor procurar o passe.`);
  if (a('dri') >= 15 && a('agi') >= 13) add('Dribble more', `Drible ${a('dri')}, Agilidade ${a('agi')}: ganha duelos individuais.`);
  else if (a('dri') && a('dri') <= 9 && attacker) add('Dribble less', `Drible ${a('dri')}: perde muitas bolas a conduzir.`);
  if (a('pas') >= 15 && a('vis') >= 15) add('Take more risks', `Passe ${a('pas')} e Visão ${a('vis')}: pode tentar o passe de ruptura.`);
  else if (a('pas') && a('pas') <= 10 && (tags.includes('CR') || ['DC','DM'].includes(group))) add('Take fewer risks', `Passe ${a('pas')}: passes simples evitam perdas perigosas.`);
  if (['D_LR','WB_LR','M_LR','AM_LR'].includes(group) && a('cro') >= 15) add(tags.includes('IN') ? 'Cross from deep' : 'Cross from byline', `Cruzamentos ${a('cro')}: é uma arma, use-a.`);
  if (a('sta') && a('sta') <= 10) add('Press less', `Resistência ${a('sta')}: guarda energia para os 90 minutos.`);
  else if (a('wor') >= 15 && a('sta') >= 14 && attacker) add('Press more', `Índice de Trabalho ${a('wor')} e Resistência ${a('sta')}: aguenta pressionar mais.`);
  if (['DC','D_LR','WB_LR','DM'].includes(group)) {
    if (a('tck') >= 15 && a('agg') >= 13) add('Tackle harder', `Desarme ${a('tck')}, Agressividade ${a('agg')}: ganha duelos.`);
    else if (a('tck') && a('tck') <= 10) add('Ease off tackles', `Desarme ${a('tck')}: evita faltas e cartões.`);
    if (a('mar') >= 15) add('Mark tighter', `Marcação ${a('mar')}: anula o adversário directo.`);
  }
  if (group === 'ST') {
    if (tags.includes('AIR') || (a('hea') >= 15 && a('jum') >= 14)) add('Target Centre', `Cabeceamento ${a('hea')}: ataca o centro da área nos cruzamentos.`);
    else if (ipRole && ipRole.id === 'poa') add('Target Near Post', 'Poacher: o primeiro poste é onde aparece primeiro.');
  }
  if (ipRole && ipRole.id === 'wf_ip') add('Target Far Post', 'Wide Forward: ataca o segundo poste enquanto o avançado prende os centrais.');
  if (['DM','MC'].includes(group) && tags.includes('RD') && a('dec') >= 12) add('Hold position', 'Garante a rest-defence enquanto os outros sobem.');
  return out.slice(0, 3);
}

// ---------- Suplentes e sucessores por posição ----------
function benchAndSuccessors(t, players) {
  const used = new Set(t.slots.map(s => s.pid).filter(Boolean));
  const rest = players.filter(p => !used.has(p.id));
  return t.slots.map(s => {
    const ranked = rest.map(p => ({ p, sc: slotScore(p, t, s) })).sort((a, b) => b.sc - a.sc);
    const bench = ranked[0] && ranked[0].sc >= 30 ? ranked[0] : null;
    // sucessor: jovem que já serve minimamente para a posição (não um GR num lugar de campo)
    const succ = ranked.find(x => x.p.age != null && x.p.age <= 21 && x.sc >= 40) || null;
    return { bench, succ };
  });
}

// ---------- Relatório do adversário ----------
const OPP_SECTORS = [
  { k:'cb_pace', n:'Centrais: velocidade', them:['DC'], a:['pac','acc'], us:['ST','AM_LR','AMC'], ua:['pac','acc'], dir:'attack' },
  { k:'cb_air', n:'Centrais: jogo aéreo', them:['DC'], a:['hea','jum','str'], us:['ST'], ua:['hea','jum','str'], dir:'attack' },
  { k:'fb_def', n:'Laterais a defender', them:['D_LR','WB_LR'], a:['tck','mar','pos','acc'], us:['AM_LR','M_LR'], ua:['dri','acc','agi','tec'], dir:'attack' },
  { k:'mid_create', n:'Criação no meio-campo', them:['DM','MC','AMC'], a:['pas','vis','dec','tec','fir'], us:['DM','MC'], ua:['pos','tck','ant','dec'], dir:'defend' },
  { k:'press', n:'Pressão', them:['DM','MC','AMC','AM_LR','ST'], a:['wor','sta','agg','acc'], us:['DC','DM','GK'], ua:['pas','cmp','fir','dec'], dir:'defend' },
  { k:'wing', n:'Extremos: perigo', them:['AM_LR','M_LR'], a:['dri','acc','pac','cro'], us:['D_LR','WB_LR'], ua:['tck','mar','acc','pos'], dir:'defend' },
  { k:'fin', n:'Finalização', them:['ST','AMC'], a:['fin','cmp','otb'], us:['DC'], ua:['pos','ant','cnt','mar'], dir:'defend' },
  { k:'fw_pace', n:'Avançados: velocidade', them:['ST','AM_LR'], a:['pac','acc'], us:['DC'], ua:['pac','acc'], dir:'defend' },
  { k:'air_att', n:'Ataque aéreo', them:['ST'], a:['hea','jum','str'], us:['DC'], ua:['hea','jum','str'], dir:'defend' },
];

function opponentXI(oppPlayers) {
  const tp = TEMPLATES.find(x => x.id === 'balanced4231') || TEMPLATES[0];
  const t = tacticFromTemplate(tp);
  autoPickXI(t, oppPlayers);
  const pm = Object.fromEntries(oppPlayers.map(p => [p.id, p]));
  return t.slots.map(s => ({ p: pm[s.pid], code: s.ipCode, group: slotInfo(t.ipForm, s.ipCode).group, side: slotInfo(t.ipForm, s.ipCode).side })).filter(x => x.p);
}
function myXI(t, players) {
  const pm = Object.fromEntries(players.map(p => [p.id, p]));
  let tt = t;
  if (t.slots.filter(s => pm[s.pid]).length < 9 && players.length >= 11) { tt = JSON.parse(JSON.stringify(t)); autoPickXI(tt, players); }
  return tt.slots.map(s => ({ p: pm[s.pid], code: s.ipCode, group: slotInfo(tt.ipForm, s.ipCode).group, side: slotInfo(tt.ipForm, s.ipCode).side })).filter(x => x.p);
}

function opponentReport(t, myPlayers, oppPlayers) {
  const them = opponentXI(oppPlayers), us = myPlayers.length ? myXI(t, myPlayers) : [];
  const pick = (xi, gs) => xi.filter(x => gs.includes(x.group)).map(x => x.p);
  const sectors = OPP_SECTORS.map(s => {
    const tv = avgPlayers(pick(them, s.them), s.a), uv = us.length ? avgPlayers(pick(us, s.us), s.ua) : null;
    return { ...s, tv, uv, diff: tv != null && uv != null ? uv - tv : null };
  });
  const S = Object.fromEntries(sectors.map(s => [s.k, s]));
  const plan = [], strengths = [], weaknesses = [];
  const push = (ti, v, why) => plan.push({ ti, v, why });
  const d = (k) => S[k] && S[k].diff;
  // ataque
  if (d('cb_pace') >= 1.5) push('reception', 0, `Os teus atacantes são mais rápidos do que os centrais deles (${S.cb_pace.uv.toFixed(1)} vs ${S.cb_pace.tv.toFixed(1)}): joga para o espaço nas costas.`);
  if (d('cb_pace') <= -1.5) push('reception', 2, `Os centrais deles são mais rápidos do que os teus atacantes: passe ao pé, sem bolas nas costas.`);
  if (d('cb_air') >= 1.5) { push('crossing', 3, `Os teus avançados ganham no ar aos centrais deles (${S.cb_air.uv.toFixed(1)} vs ${S.cb_air.tv.toFixed(1)}): cruzamentos altos.`); push('set_pieces', 2, 'Vantagem aérea: procura cantos e livres.'); }
  if (d('cb_air') <= -1.5) push('crossing', 0, `Os centrais deles dominam no ar (${S.cb_air.tv.toFixed(1)}): só cruzamentos rasteiros.`);
  if (S.fb_def && S.fb_def.tv != null) {
    const dl = them.find(x => x.code === 'DL'), dr = them.find(x => x.code === 'DR');
    const vl = dl ? avgOf(dl.p, S.fb_def.a) : null, vr = dr ? avgOf(dr.p, S.fb_def.a) : null;
    if (vl != null && vr != null && Math.abs(vl - vr) >= 1.2) {
      const weakLeft = vl < vr; // o lateral esquerdo deles defende o teu flanco direito
      push('progress', weakLeft ? 3 : 0, `O lateral ${weakLeft ? 'esquerdo' : 'direito'} deles (${(weakLeft ? dl : dr).p.name}) é o elo fraco: ataca pelo teu flanco ${weakLeft ? 'direito' : 'esquerdo'}.`);
    }
  }
  // defesa
  const pr = S.press && S.press.tv;
  if (pr != null && pr >= 13.5) push('buildup', 2, `Pressionam muito (${pr.toFixed(1)}): salta a pressão com bola longa em vez de sair a jogar.`);
  if (pr != null && pr <= 10.5) push('buildup', 0, `Pressionam pouco (${pr.toFixed(1)}): sai a jogar curto com calma.`);
  if (d('fw_pace') <= -1.5) push('dline', 1, `Os avançados deles são mais rápidos do que os teus centrais (${S.fw_pace.tv.toFixed(1)} vs ${S.fw_pace.uv.toFixed(1)}): baixa a linha.`);
  if (d('fw_pace') >= 1.5) push('dline', 3, `Os teus centrais são mais rápidos do que os avançados deles: podes subir a linha.`);
  if (d('air_att') <= -1.5) push('cross_eng', 2, `Os avançados deles são fortes no ar (${S.air_att.tv.toFixed(1)}): impede os cruzamentos.`);
  if (S.mid_create && S.mid_create.tv != null && S.mid_create.tv >= 14) push('press_trap', 2, `O meio-campo deles cria muito (${S.mid_create.tv.toFixed(1)}): empurra o jogo para as alas.`);
  if (S.wing && S.wing.tv != null && S.wing.tv >= 14.5 && d('wing') <= -1) push('def_trans', 0, `Os extremos deles são perigosos em transição: reagrupa antes de pressionar.`);
  // fortes e fracos
  sectors.forEach(s => { if (s.tv == null) return; if (s.tv >= 14) strengths.push(s); if (s.tv <= 10.5) weaknesses.push(s); });
  // jogadores perigosos
  const threat = (x) => { const p = x.p; const base = ['ST','AMC'].includes(x.group) ? ['fin','otb','cmp','acc','dri'] : ['AM_LR','M_LR'].includes(x.group) ? ['dri','acc','pac','cro','fin'] : ['pas','vis','dec','tec','lon']; return avgOf(p, base) || 0; };
  const dangers = them.filter(x => !['GK','DC','D_LR','WB_LR'].includes(x.group)).map(x => ({ ...x, th: threat(x) })).sort((a, b) => b.th - a.th).slice(0, 3).map(x => ({
    p: x.p, code: x.code, th: x.th,
    ois: ['AM_LR','M_LR'].includes(x.group) ? ['Show onto weaker foot', 'Tight marking'] : ['ST'].includes(x.group) ? ['Tight marking', 'Tackle harder'] : ['Close down more', 'Tight marking'],
  }));
  // unir instruções repetidas (fica a primeira razão)
  const seen = new Set(), uplan = plan.filter(x => !seen.has(x.ti) && seen.add(x.ti));
  return { them, us, sectors, plan: uplan, strengths, weaknesses, dangers, club: majorityClub(oppPlayers).club };
}

// ---------- clube maioritário de uma exportação ----------
function majorityClub(players) {
  const c = {}; players.forEach(p => { const k = p.meta && p.meta.club; if (k) c[k] = (c[k] || 0) + 1; });
  const e = Object.entries(c).sort((a, b) => b[1] - a[1])[0];
  return e ? { club: e[0], share: e[1] / players.length } : { club: null, share: 0 };
}

// ---------- Bolas paradas e capitães ----------
// Pontuações em escala ~1–20 a partir dos atributos; cada função devolve também o porquê.
const SP = {
  aerial: (p) => { const a = (k) => A_(p, k) ?? 8; let h = 0; const cm = parseFloat(String(p.meta && p.meta.height || '').replace(',', '.')); if (cm > 150) h = (cm - 183) / 5 * 0.6;
    return (a('hea') * 1.3 + a('jum') * 1.2 + a('str') * 0.6 + a('bra') * 0.5 + a('ant') * 0.4) / 4 + h; },
  corner: (p) => { const a = (k) => A_(p, k) ?? 8; return (a('cor') * 1.6 + a('cro') + a('tec') * 0.5) / 3.1; },
  fkDirect: (p) => { const a = (k) => A_(p, k) ?? 8; return (a('fre') * 1.7 + a('tec') + a('lon') * 0.6 + a('cmp') * 0.3) / 3.6; },
  fkCross: (p) => { const a = (k) => A_(p, k) ?? 8; return (a('fre') + a('cro') + a('tec') * 0.5) / 2.5; },
  penalty: (p) => { const a = (k) => A_(p, k) ?? 8; return (a('pen') * 2 + a('cmp') + a('tec') * 0.3) / 3.3; },
  marker: (p) => { const a = (k) => A_(p, k) ?? 8; return (a('mar') * 1.2 + a('hea') + a('str') * 0.6 + a('pos') * 0.6 + a('jum') * 0.6 + a('cnt') * 0.4) / 4.4; },
  zone: (p) => { const a = (k) => A_(p, k) ?? 8; return (a('pos') * 1.2 + a('hea') + a('jum') * 0.8 + a('ant') * 0.6 + a('cnt') * 0.4) / 4; },
  cover: (p) => { const a = (k) => A_(p, k) ?? 8; return (a('pac') + a('acc') + a('pos') * 0.6 + a('tck') * 0.5 + a('ant') * 0.4) / 3.5; },
  outlet: (p) => { const a = (k) => A_(p, k) ?? 8; return (a('pac') + a('acc') + a('otb') * 0.6 + a('fir') * 0.3) / 2.9; },
  edge: (p) => { const a = (k) => A_(p, k) ?? 8; return (a('lon') * 1.2 + a('tec') + a('fir') * 0.5 + a('cmp') * 0.3) / 3; },
  captain: (p) => { const a = (k) => A_(p, k) ?? 8; return (a('ldr') * 1.6 + a('det') + a('tea') * 0.4 + a('cmp') * 0.3) / 3.3 + Math.min(4, Math.max(0, ((p.age || 24) - 22) * 0.4)); },
};
const footOf = (p) => { const f = String((p.meta && p.meta.foot) || '').toLowerCase(); return /left|esq/.test(f) ? 'L' : /right|dir/.test(f) ? 'R' : /either|ambos|both/.test(f) ? 'B' : null; };
// apelido para o campo: última palavra, mantendo partículas (De Bruyn, Van der Berg, Da Silva)
const NAME_PARTICLES = new Set(['de','da','das','do','dos','di','del','della','van','von','der','den','ten','ter','le','la','du','mac','al','el','bin','st.']);
function surname(name) {
  const w = String(name || '').trim().split(/\s+/); if (w.length < 2) return w[0] || '—';
  let i = w.length - 1; if (i > 1 && /^(j[uú]nior|jr\.?|filho|neto|sobrinho|ii|iii)$/i.test(w[i])) i--;   // Silva Júnior
  while (i > 0 && NAME_PARTICLES.has(w[i - 1].toLowerCase())) i--;
  return w.slice(i).join(' ');
}
const shortName = (p) => p ? surname(p.name) : '—';

function setPiecePlan(t, players, oppList) {
  const xi = players.length >= 11 ? myXI(t, players) : [];
  if (xi.length < 10) return null;
  const out = xi.filter(x => x.group !== 'GK').map(x => ({ ...x, s: Object.fromEntries(Object.keys(SP).map(k => [k, SP[k](x.p)])) }));
  const used = new Set(), take = (list, key, n = 1, filter = () => true) => {
    const r = list.filter(x => !used.has(x.p.id) && filter(x)).sort((a, b) => b.s[key] - a.s[key]).slice(0, n); r.forEach(x => used.add(x.p.id)); return r;
  };
  const rank = (key, filter = () => true) => [...out].filter(filter).sort((a, b) => b.s[key] - a.s[key]);
  const why = (x, keys) => keys.map(k => `${attrName(k)} ${Math.round(A_(x.p, k) ?? 0)}`).join(', ');
  // ---- marcadores
  const cornerRank = rank('corner');
  const left = cornerRank.find(x => footOf(x.p) === 'L'), right = cornerRank.find(x => footOf(x.p) === 'R');
  const best = cornerRank[0];
  const corners = {
    // canto da direita batido com o pé esquerdo = curva para dentro (inswinger)
    right: left && left.s.corner >= best.s.corner - 1.5 ? { x: left, swing: 'para dentro (inswinger)' } : { x: best, swing: footOf(best.p) === 'R' ? 'para fora (outswinger)' : footOf(best.p) === 'L' ? 'para dentro (inswinger)' : null },
    left: right && right.s.corner >= best.s.corner - 1.5 ? { x: right, swing: 'para dentro (inswinger)' } : { x: best, swing: footOf(best.p) === 'L' ? 'para fora (outswinger)' : footOf(best.p) === 'R' ? 'para dentro (inswinger)' : null },
  };
  const fkD = rank('fkDirect')[0], fkC = rank('fkCross')[0];
  const directOk = fkD && (A_(fkD.p, 'fre') ?? 0) >= 14;
  const pens = rank('penalty').slice(0, 3);
  const lt = rank('fkCross', x => (A_(x.p, 'lth') ?? 0) >= 14).sort((a, b) => (A_(b.p, 'lth') ?? 0) - (A_(a.p, 'lth') ?? 0))[0] || null;
  const caps = [...xi.map(x => ({ ...x, c: SP.captain(x.p) }))].sort((a, b) => b.c - a.c).slice(0, 2);
  // ---- canto ofensivo
  const taker = corners.right.x;
  used.add(taker.p.id);
  const oppFast = oppList && oppList.length ? avgPlayers(opponentXI(oppList).filter(x => ['ST','AM_LR'].includes(x.group)).map(x => x.p), ['pac','acc']) : null;
  const nBack = oppFast != null && oppFast >= 14 ? 3 : 2;
  // os melhores no ar não ficam atrás; dos restantes, os mais rápidos defensores ficam a proteger
  const topAir = new Set(rank('aerial').slice(0, 4).map(x => x.p.id));
  const back = take(out, 'cover', nBack, x => ['DC','D_LR','WB_LR','DM','MC'].includes(x.group) && !topAir.has(x.p.id));
  const box = take(out, 'aerial', 4);
  // 3.º e 4.º melhores no ar: o mais rápido vai ao 1.º poste, o outro fica na marca de penálti
  const rest = box.slice(2).sort((a, b) => (A_(b.p, 'acc') ?? 0) - (A_(a.p, 'acc') ?? 0));
  const [farP, centre, nearP, spot] = [box[0], box[1], rest[0], rest[1]];
  const edge = take(out, 'edge', 1)[0];
  const shortOpt = take(out, 'fkCross', 1, x => ['AM_LR','M_LR','D_LR','WB_LR','MC','AMC'].includes(x.group))[0];
  const attack = [
    { k: 'taker', n: 'Marcador', x: taker, w: corners.right.swing ? 'canto ' + corners.right.swing : why(taker, ['cor', 'cro']) },
    { k: 'far', n: '2.º poste', x: farP, w: why(farP, ['hea', 'jum']) + ' — o melhor no ar ataca a bola que passa todos' },
    { k: 'centre', n: 'Centro da pequena área', x: centre, w: why(centre, ['hea', 'jum', 'str']) },
    { k: 'near', n: '1.º poste', x: nearP, w: why(nearP, ['hea', 'acc']) + ' — antecipa-se e desvia' },
    { k: 'spot', n: 'Marca de penálti (segunda bola)', x: spot, w: why(spot, ['hea', 'ant']) },
    { k: 'edge', n: 'Entrada da área', x: edge, w: why(edge, ['lon', 'tec']) + ' — remata as sobras' },
    { k: 'short', n: 'Opção curta', x: shortOpt, w: shortOpt ? why(shortOpt, ['pas', 'cro']) : '' },
    ...back.map((b, i) => ({ k: 'back' + i, n: 'Fica atrás', x: b, w: why(b, ['pac', 'acc', 'pos']) + ' — protege o contra-ataque' })),
  ].filter(r => r.x);
  // ---- canto defensivo
  used.clear();
  const outlet = take(out, 'outlet', tiVal(t, 'att_trans') === 2 ? 2 : 1, x => ['ST','AM_LR','AMC','M_LR'].includes(x.group));
  const theirAir = oppList && oppList.length ? opponentXI(oppList).filter(x => x.group !== 'GK').map(x => ({ p: x.p, a: SP.aerial(x.p) })).sort((a, b) => b.a - a.a).slice(0, 3) : [];
  const ourAirAvg = avgPlayers(out.map(x => x.p), ['hea', 'jum']);
  const system = ourAirAvg != null && ourAirAvg >= 13.5 ? 'Zonal' : theirAir.length && theirAir[0].a >= 15 ? 'Misto (zonal + marcação ao mais perigoso)' : 'Misto';
  const zones = take(out, 'zone', 3);
  const markers = take(out, 'marker', Math.max(2, theirAir.length));
  const dEdge = take(out, 'cover', 1, x => ['DM','MC','AMC','AM_LR','M_LR'].includes(x.group))[0];
  const defend = [
    { k: 'zn', n: 'Zona 1.º poste', x: zones[1] || zones[0], w: 'posicionamento e jogo aéreo' },
    { k: 'zc', n: 'Zona centro (pequena área)', x: zones[0], w: why(zones[0], ['pos', 'hea', 'jum']) },
    { k: 'zf', n: 'Zona 2.º poste', x: zones[2], w: 'posicionamento e jogo aéreo' },
    ...markers.map((m, i) => ({ k: 'm' + i, n: theirAir[i] ? `Marca ${shortName(theirAir[i].p)}` : 'Marcação individual', x: m, w: why(m, ['mar', 'hea', 'str']) + (theirAir[i] ? ` · ${theirAir[i].p.name} é dos melhores no ar deles` : '') })),
    { k: 'de', n: 'Entrada da área (segundas bolas)', x: dEdge, w: dEdge ? why(dEdge, ['acc', 'ant']) : '' },
    ...outlet.map((o, i) => ({ k: 'o' + i, n: 'Fica na frente (contra-ataque)', x: o, w: why(o, ['pac', 'acc']) })),
  ].filter(r => r.x);
  // ---- conselhos gerais
  const tips = [];
  const ourAirTop = rank('aerial').slice(0, 4).map(x => x.s.aerial);
  const airTop = ourAirTop.reduce((a, b) => a + b, 0) / Math.max(1, ourAirTop.length);
  if (airTop >= 14) tips.push({ ti: 'set_pieces', v: 2, t: `Tens ${ourAirTop.length} jogadores fortes no ar (média ${airTop.toFixed(1)}): procura cantos e livres.` });
  else if (airTop <= 11) tips.push({ ti: 'set_pieces', v: 0, t: `Pouca presença aérea (média ${airTop.toFixed(1)}): mais vale manter a bola em jogo; cantos curtos.` });
  if (!directOk) tips.push({ t: `Ninguém com Livres ≥ 14 (melhor: ${fkD ? fkD.p.name + ' ' + Math.round(A_(fkD.p, 'fre') ?? 0) : '—'}): nos livres frontais, cruza em vez de rematar.` });
  if (lt) tips.push({ t: `${lt.p.name} tem Lançamentos Longos ${Math.round(A_(lt.p, 'lth'))}: use lançamentos longos para a área no último terço.` });
  if (nBack === 3) tips.push({ t: 'Os avançados deles são rápidos: deixa 3 atrás nos cantos ofensivos.' });
  return { xi, corners, fk: { direct: directOk ? fkD : null, cross: fkC }, pens, longThrow: lt, caps, attack, defend, system, theirAir, tips, airTop };
}
