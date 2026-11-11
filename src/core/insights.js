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
