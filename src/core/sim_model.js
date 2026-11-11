// ============================================================
// Campo animado: cada jogador tem uma posição por fase do jogo,
// calculada a partir da formação, do papel e das instruções de equipa.
// Coordenadas: x = largura 0 (esq.) → 100 (dir.); y = profundidade 0 (baliza própria) → 100 (adversária)
// ============================================================

const PHASES = [
  { k:'build', n:'Construção', ph:'IP' },
  { k:'prog',  n:'Progressão', ph:'IP' },
  { k:'final', n:'Último terço', ph:'IP' },
  { k:'loss',  n:'Perda da bola', ph:'T' },
  { k:'oop',   n:'Bloco sem bola', ph:'OOP' },
  { k:'win',   n:'Recuperação', ph:'T' },
];

// Movimento de cada papel com bola. y = profundidade; x = palavra-chave de corredor (ver xKey)
const MOVES = {
  gk_ip:{ build:{y:4}, prog:{y:10}, final:{y:14} },
  bpgk:{ build:{y:9}, prog:{y:16}, final:{y:21} },
  nngk:{ build:{y:4}, prog:{y:8}, final:{y:12} },
  cb_ip:{ build:{y:12, xs:1.45}, prog:{y:24}, final:{y:36} },
  bpcb:{ build:{y:12, xs:1.45}, prog:{y:27}, final:{y:38} },
  nncb:{ build:{y:11, xs:1.35}, prog:{y:22}, final:{y:33} },
  acb:{ build:{y:15}, prog:{y:40, x:'in'}, final:{y:52, x:'in'} },
  ocb:{ build:{y:14, xs:1.6}, prog:{y:44, x:'wide'}, final:{y:74, x:'wide'} },
  fb_ip:{ build:{y:17, x:'wide'}, prog:{y:31, x:'wide'}, final:{y:46, x:'wide'} },
  ifb:{ build:{y:12, x:'cbw'}, prog:{y:22, x:'cbw'}, final:{y:34, x:'cbw'} },
  wb_ip:{ build:{y:23, x:'wide'}, prog:{y:48, x:'wide'}, final:{y:77, x:'wide'} },
  iwb:{ build:{y:28, x:'half'}, prog:{y:40, x:'half'}, final:{y:52, x:'half'} },
  pwb:{ build:{y:30, x:'half'}, prog:{y:47, x:'half'}, final:{y:63, x:'half'} },
  awb:{ build:{y:27, x:'wide'}, prog:{y:60, x:'wide'}, final:{y:85, x:'wide'} },
  dm_ip:{ build:{y:28}, prog:{y:38}, final:{y:46} },
  dlp:{ build:{y:25}, prog:{y:38}, final:{y:52} },
  hb:{ build:{y:11, x:'c'}, prog:{y:21, x:'c'}, final:{y:35, x:'c'} },
  b2b:{ build:{y:34}, prog:{y:55}, final:{y:84, x:'in'} },
  b2bp:{ build:{y:34}, prog:{y:52}, final:{y:72, x:'half'} },
  cm_ip:{ build:{y:40}, prog:{y:52}, final:{y:66} },
  mp:{ build:{y:38}, prog:{y:50}, final:{y:64, x:'half'} },
  wcm:{ build:{y:42}, prog:{y:57, x:'wide2'}, final:{y:71, x:'wide2'} },
  chm:{ build:{y:42}, prog:{y:58}, final:{y:86, x:'ch'} },
  ap:{ build:{y:47}, prog:{y:63, x:'half'}, final:{y:75, x:'half'} },
  am_ip:{ build:{y:49}, prog:{y:65}, final:{y:82, x:'in'} },
  wm_ip:{ build:{y:44, x:'wide'}, prog:{y:60, x:'wide'}, final:{y:72, x:'wide'} },
  w_ip:{ build:{y:50, x:'wide'}, prog:{y:70, x:'wide'}, final:{y:91, x:'wide'} },
  iw_ip:{ build:{y:50, x:'wide2'}, prog:{y:66, x:'half'}, final:{y:77, x:'half'} },
  pw:{ build:{y:48, x:'wide2'}, prog:{y:62, x:'half'}, final:{y:72, x:'half'} },
  if_ip:{ build:{y:52, x:'wide2'}, prog:{y:70, x:'half'}, final:{y:89, x:'in2'} },
  wf_ip:{ build:{y:52, x:'wide'}, prog:{y:72, x:'wide'}, final:{y:88, x:'wide3'} },
  fr:{ build:{y:50}, prog:{y:64, x:'ball'}, final:{y:77, x:'ballhalf'} },
  ss:{ build:{y:56}, prog:{y:73}, final:{y:88, x:'in'} },
  cf_ip:{ build:{y:62}, prog:{y:78}, final:{y:90} },
  chf:{ build:{y:64}, prog:{y:80, x:'ch'}, final:{y:91, x:'ch'} },
  dlf:{ build:{y:57}, prog:{y:67}, final:{y:78} },
  f9:{ build:{y:55}, prog:{y:63, x:'c'}, final:{y:72, x:'c'} },
  poa:{ build:{y:66}, prog:{y:84}, final:{y:94} },
  tf:{ build:{y:64}, prog:{y:82}, final:{y:91, x:'c'} },
};

// Notas curtas por papel, mostradas na fase em que o comportamento se vê melhor
const MOVE_NOTES = {
  bpgk:['build','sai da baliza para servir de apoio na construção'], acb:['prog','sobe para o meio-campo'],
  ocb:['final','sobrepõe pelo corredor como um ala'], ifb:['build','fecha como terceiro central'],
  wb_ip:['final','sobe pelo corredor e dá a largura'], iwb:['build','entra para o meio, ao lado do médio defensivo'],
  pwb:['prog','entra para o meio e organiza'], awb:['final','vai até à linha de fundo, quase como extremo'],
  dlp:['prog','fica atrás a distribuir o jogo'], hb:['build','baixa entre os centrais (linha de 3)'],
  b2b:['final','chega à área em segunda vaga'], b2bp:['final','sobe para criar entre linhas'],
  wcm:['prog','abre para o corredor a partir do meio'], chm:['final','ataca o canal entre central e lateral'],
  ap:['prog','procura espaço entre linhas'], am_ip:['final','ocupa a zona 14 e ameaça de fora'],
  w_ip:['final','vai à linha de fundo para cruzar'], iw_ip:['final','entra pelo meio-espaço para criar'],
  pw:['prog','parte da ala e entra para organizar'], if_ip:['final','corta para dentro em direcção à baliza'],
  wf_ip:['final','fica aberto e ataca o segundo poste'], fr:['prog','vagueia para o lado da bola'],
  ss:['final','junta-se ao avançado na área'], cf_ip:['final','fixa os centrais e finaliza'],
  chf:['final','ataca o canal'], dlf:['final','baixa para ligar o jogo'], f9:['prog','baixa até ao meio-campo e arrasta centrais'],
  poa:['final','vive na última linha à espera da bola'], tf:['final','referência aérea na área'],
  swk:['oop','posiciona-se alto para varrer bolas nas costas'], lhk:['oop','fica na linha de golo'],
  ccb:['oop','fica atrás da linha a cobrir a profundidade'], scb:['oop','sai da linha para apertar entre linhas'],
  cwcb:['oop','cobre o canal, mais fundo'], swcb:['oop','sai agressivamente ao meio-espaço'],
  hfb:['oop','fica fundo e estreito a proteger o central'], pfb:['oop','sobe a pressionar o extremo'],
  prwb:['oop','sobe a pressionar'], hwb:['oop','mantém a linha de 5'], ddm:['oop','baixa para a linha de defesas'],
  sdm:['oop','fecha linhas de passe centrais'], wcdm:['oop','desloca-se para cobrir o flanco'], pdm:['oop','sai a pressionar do médio defensivo'],
  pcm:['oop','sobe a pressionar'], scm:['oop','protege a zona central'], wccm:['oop','cobre o flanco do lateral'],
  twm:['oop','acompanha o lateral adversário até atrás'], wowm:['oop','fica alto e aberto para a saída'], tam:['oop','recua para defender'],
  coam:['oop','fica alto e central para a saída'], soam:['oop','fica alto, desviado para o canal'], tw:['oop','acompanha o recuo'],
  wow:['oop','fica aberto à espera do contra-ataque'], iow:['oop','fica alto e por dentro'], cocf:['oop','fica como válvula de escape'],
  socf:['oop','fica alto, desviado para o canal'], tcf:['oop','recua e pressiona o médio adversário'],
};

const OPP_SHAPE = [[50,4],[12,22],[36,18],[64,18],[88,22],[40,36],[60,36],[14,62],[50,60],[86,62],[50,78]];
const clamp = (v, a, b) => Math.max(a, Math.min(b, v));
const ease = (u) => u < .5 ? 2 * u * u : 1 - Math.pow(-2 * u + 2, 2) / 2;

function simModel(t) {
  const mi = MENTALITIES.indexOf(t.ment), ms = (mi - 3) * 1.5;
  const v = (k) => tiVal(t, k);
  const W = [0.8, 0.9, 1, 1.05, 1.1][v('width')];
  const prog = v('progress');
  const bs = prog === 3 ? 1 : prog === 0 ? -1 : -1;             // lado da bola (esquerda por defeito)
  const P = t.slots.map((s, i) => {
    const a = slotInfo(t.ipForm, s.ipCode), b = slotInfo(t.oopForm, s.oopCode);
    const sg = a.x < 45 ? -1 : a.x > 55 ? 1 : 0;
    return { i, s, a, b, ip: ROLE_BY_ID[s.ip], oop: ROLE_BY_ID[s.oop], sg, sgb: b.x < 45 ? -1 : b.x > 55 ? 1 : 0, pos: {} };
  });
  const sideOr = (p, alt) => p.sg || (p.a.x < 50 ? -1 : p.a.x > 50 ? 1 : alt);
  const xKey = (p, key, x) => {
    const s = sideOr(p, bs);
    switch (key) {
      case 'wide': return 50 + s * 45 * W; case 'wide2': return 50 + s * 37 * W; case 'wide3': return 50 + s * 31 * W;
      case 'half': return 50 + s * 18 * Math.sqrt(W); case 'in': return 50 + s * 9; case 'in2': return 50 + s * 13;
      case 'ch': return 50 + s * 22; case 'cbw': return 50 + s * 30; case 'c': return 50;
      case 'ball': return 50 + bs * 14; case 'ballhalf': return 50 + bs * 19; default: return x;
    }
  };
  const hasHB = P.some(p => p.ip && p.ip.id === 'hb');
  // ---- fases com bola
  ['build', 'prog', 'final'].forEach(ph => {
    const yBase = { build: (y) => y * 0.6 + 3, prog: (y) => y * 0.85 + 8, final: (y) => Math.min(96, y * 0.75 + 25) }[ph];
    P.forEach(p => {
      let x = 50 + (p.a.x - 50) * W, y = yBase(p.a.y);
      const m = p.ip && MOVES[p.ip.id] && MOVES[p.ip.id][ph];
      if (m) { if (m.y != null) y = m.y; if (m.xs) x = 50 + (p.a.x - 50) * m.xs * (hasHB ? 1.25 : 1); if (m.x) x = xKey(p, m.x, x); }
      if (p.ip && ['fb_ip','wb_ip','awb'].includes(p.ip.id) && ph === 'final') {
        const sr = v('support_runs'); if (sr === 2) { x = xKey(p, 'wide'); y += 4; } if (sr === 0) { x = xKey(p, 'half'); y -= 2; }
      }
      if (p.a.group !== 'GK') y += ms;
      if (ph === 'prog' && v('passing') >= 3 && p.a.group === 'ST') y += 4;
      p.pos[ph] = { x: clamp(x, 5, 95), y: clamp(y, 2, 96) };
    });
  });
  // ---- bola com bola
  const byGroup = (gs) => P.filter(p => gs.includes(p.a.group));
  const gk = P.find(p => p.a.group === 'GK');
  const cbs = byGroup(['DC']).sort((p, q) => Math.abs(p.pos.build.x - (50 + bs * 30)) - Math.abs(q.pos.build.x - (50 + bs * 30)));
  const mids = byGroup(['DM','MC']).sort((p, q) => (q.ip && q.ip.tags.includes('CR') ? 1 : 0) - (p.ip && p.ip.tags.includes('CR') ? 1 : 0) || p.pos.build.y - q.pos.build.y);
  const strikers = byGroup(['ST','AMC']).sort((p, q) => (q.ip && q.ip.tags.includes('AIR') ? 1 : 0) - (p.ip && p.ip.tags.includes('AIR') ? 1 : 0) || q.pos.final.y - p.pos.final.y);
  const direct = v('passing') >= 3 || v('buildup') === 2 || v('goal_kicks') === 2;
  const holder = mids[0] || cbs[0];
  const sideWide = P.filter(p => p.a.group !== 'GK' && sideOr(p, bs) === bs).sort((p, q) => q.pos.final.y - p.pos.final.y);
  const wideProv = sideWide.find(p => p.ip && p.ip.tags.includes('W') && p.pos.final.y > 70);
  const boxTarget = [...P].filter(p => p.ip && (p.ip.tags.includes('BOX') || p.ip.tags.includes('AIR'))).sort((p, q) => q.pos.final.y - p.pos.final.y)[0] || strikers[0];
  const creator = [...P].filter(p => p.ip && p.ip.tags.includes('CR') && p.a.group !== 'GK').sort((p, q) => q.pos.final.y - p.pos.final.y)[0];
  const runner = [...P].filter(p => p.ip && p.ip.tags.includes('RUN')).sort((p, q) => q.pos.final.y - p.pos.final.y)[0] || boxTarget;
  const at = (p, ph, dx = 0, dy = 0) => p ? { x: p.pos[ph].x + dx, y: p.pos[ph].y + dy } : { x: 50, y: 50 };
  const ball = {};
  ball.build = direct ? [at(gk, 'build', 0, 1.5), at(strikers[0], 'build')] : [at(gk, 'build', 0, 1.5), at(cbs[0], 'build'), at(holder, 'build')];
  const progTarget = byGroup(['D_LR','WB_LR','M_LR','AM_LR','AMC','MC']).filter(p => sideOr(p, bs) === bs || p.a.group === 'AMC').sort((p, q) => q.pos.prog.y - p.pos.prog.y)[0];
  ball.prog = direct ? [at(holder, 'prog'), at(strikers[0], 'prog')] : [at(holder, 'prog'), at(progTarget || strikers[0], 'prog')];
  ball.final = wideProv ? [at(wideProv, 'final', 0, -6), at(wideProv, 'final'), at(boxTarget, 'final', 0, 1)]
    : [at(creator || holder, 'final'), at(runner, 'final', 0, 2)];
  const lost = ball.final[ball.final.length - 1];
  // ---- bloco sem bola
  const dl = v('dline'), loe = v('loe'), trig = v('trigger'), dlb = tiVal(t, 'dl_behaviour');
  const dLine = [16, 21, 27, 33, 39][dl] + ms * 0.6 + (dlb - 1) * 2;
  const front = [44, 56, 68][loe] + ms * 0.6;
  const trap = v('press_trap');
  const oppFront = dLine + 8, oppBack = Math.max(80, front + 16);
  const oppAt = (k, D, F) => { const [ox, oy] = OPP_SHAPE[k]; const tt = clamp((oy - 18) / 60, 0, 1); return k === 0 ? { x: 100 - ox, y: Math.min(97, D + 10) } : { x: 100 - ox, y: D + (F - D) * tt }; };
  const oppStart = oppAt(2, oppBack, oppFront);
  const ballOOPend = trap === 2 ? { x: 50 + bs * 38, y: front + 2 } : trap === 0 ? { x: 50, y: front + 3 } : { x: 50 + bs * 20, y: front + 4 };
  ball.oop = [oppStart, ballOOPend];
  const bxo = ballOOPend.x;
  P.forEach(p => {
    const b = p.b, r = p.oop;
    let x = 50 + (b.x - 50) * 0.82 + (bxo - 50) * 0.22;
    let y;
    if (p.b.group === 'GK') y = 5 + (dl >= 3 ? 4 : 0);
    else {
      const tt = clamp((b.y - 18) / 62, 0, 1); y = dLine + (front - dLine) * tt;
    }
    const id = r && r.id, tags = r ? r.tags : [];
    if (id === 'swk') y = Math.max(8, dLine - 12); if (id === 'lhk') y = 3;
    if (tags.includes('HP')) y += 5; if (tags.includes('LOW')) y -= 3; if (tags.includes('TRK')) y -= 5;
    if (['pfb','prwb'].includes(id)) y += 6;
    if (id === 'ddm') { y = dLine + 1; x = 50; }
    if (['wcdm','wccm'].includes(id)) x = 50 + (p.sgb || -bs) * 30;
    if (['wowm','wow'].includes(id)) { y = Math.max(y, front + 5); x = 50 + (p.sgb || bs) * 42; }
    if (id === 'iow') { y = Math.max(y, front + 5); x = 50 + (p.sgb || bs) * 20; }
    if (['coam','cocf'].includes(id)) { y = Math.max(y, front + 7); x = 50; }
    if (['soam','socf'].includes(id)) { y = Math.max(y, front + 9); x = 50 + (p.sgb || bs) * 26; }
    if (id === 'tcf') y = front - 1;
    if (v('short_gk') === 1 && p.b.group === 'ST') y += 7;
    // distância mínima entre linhas: mesmo num bloco muito compacto cada linha mantém-se legível
    if (p.b.group !== 'GK') {
      const minOff = { DC:0, D_LR:0, WB_LR:0, DM:7, MC:9, M_LR:10, AMC:14, AM_LR:13, ST:18 }[p.b.group] || 0;
      y = Math.max(y, dLine + minOff - (tags.includes('TRK') ? 3 : 0));
      if (p.b.group === 'M_LR' || p.b.group === 'AM_LR') x = 50 + (x - 50) * 0.94;
    }
    p.pos.oop = { x: clamp(x, 5, 95), y: clamp(y, 2, 96) };
  });
  // pressão: os mais próximos da bola (ou com papel assertivo) saem ao portador
  const nPress = [0, 1, 1, 2, 3][trig];
  const cand = P.filter(p => p.b.group !== 'GK' && !(p.oop && p.oop.tags.includes('OUT')))
    .map(p => ({ p, d: Math.hypot(p.pos.oop.x - bxo, p.pos.oop.y - ballOOPend.y) })).sort((a, b) => a.d - b.d);
  const pressers = new Set(cand.filter(c => c.p.pos.oop.y > dLine + 6).slice(0, nPress).map(c => c.p.i));
  P.forEach(p => { if (p.oop && p.oop.tags.includes('HP') && Math.hypot(p.pos.oop.x - bxo, p.pos.oop.y - ballOOPend.y) < 30) pressers.add(p.i); });
  P.forEach(p => { if (pressers.has(p.i)) { const o = p.pos.oop; p.pos.oop = { x: o.x + (bxo - o.x) * 0.55, y: o.y + (ballOOPend.y - 3 - o.y) * 0.55 }; p.pressing = true; } });
  // ---- perda da bola
  const dtr = v('def_trans');
  P.forEach(p => {
    const f = p.pos.final, o = p.pos.oop, d = Math.hypot(f.x - lost.x, f.y - lost.y);
    let pt;
    if (dtr === 2 && d < 27 && p.a.group !== 'GK') pt = { x: f.x + (lost.x - f.x) * 0.7, y: f.y + (lost.y - 4 - f.y) * 0.7 };
    else if (dtr === 2) pt = { x: f.x, y: f.y - 4 };
    else if (dtr === 0) pt = { x: f.x + (o.x - f.x) * 0.75, y: f.y + (o.y - f.y) * 0.75 };
    else pt = d < 18 && p.a.group !== 'GK' ? { x: f.x + (lost.x - f.x) * 0.6, y: f.y + (lost.y - f.y) * 0.6 } : { x: f.x + (o.x - f.x) * 0.45, y: f.y + (o.y - f.y) * 0.45 };
    p.pos.loss = pt;
  });
  ball.loss = [lost, { x: lost.x - bs * 6, y: lost.y - (dtr === 0 ? 22 : 10) }];
  // ---- recuperação
  const atr = v('att_trans');
  const rec = { x: bxo + (50 - bxo) * 0.3, y: Math.max(dLine + 10, front - 8) };
  P.forEach(p => {
    const o = p.pos.oop, r = p.ip, ro = p.oop;
    const runs = (r && (r.tags.includes('RUN') || r.tags.includes('BOX'))) || (ro && ro.tags.includes('OUT')) || p.a.group === 'ST';
    let pt = { ...o };
    if (atr === 2 && runs) pt = { x: o.x + (p.pos.final.x - o.x) * 0.4, y: Math.min(92, o.y + 24) };
    else if (atr === 2) pt = { x: o.x, y: o.y + 6 };
    else if (atr === 0) pt = { x: o.x + (p.pos.build.x - o.x) * 0.6, y: o.y + (p.pos.build.y - o.y) * 0.6 };
    else pt = runs ? { x: o.x, y: Math.min(90, o.y + 13) } : { x: o.x, y: o.y + 4 };
    p.pos.win = { x: clamp(pt.x, 5, 95), y: clamp(pt.y, 2, 96) };
  });
  const recoverer = [...P].filter(p => p.a.group !== 'GK').sort((p, q) => Math.hypot(p.pos.oop.x - rec.x, p.pos.oop.y - rec.y) - Math.hypot(q.pos.oop.x - rec.x, q.pos.oop.y - rec.y))[0];
  const target = atr === 0 ? cbs[0] : [...P].sort((p, q) => q.pos.win.y - p.pos.win.y)[0];
  ball.win = [at(recoverer, 'oop'), at(target, 'win', 0, atr === 2 ? 1.5 : 0)];
  // ---- adversário (cinzento)
  const OPP = { build: [64, 30], prog: [74, 44], final: [86, 60], loss: [80, 46], oop: [oppBack, oppFront], win: [70, dLine + 4] };
  const opp = {}; Object.entries(OPP).forEach(([ph, [D, F]]) => { opp[ph] = OPP_SHAPE.map((_, k) => oppAt(k, D, F)); });
  // ---- avisos visuais no bloco
  const lineY = (gs) => { const q = P.filter(p => gs.includes(p.b.group) && !p.pressing); return q.length ? q.reduce((a, p) => a + p.pos.oop.y, 0) / q.length : null; };
  const defY = lineY(['DC']), midY = lineY(['DM','MC','M_LR']);
  const overlays = [];
  if (defY != null && midY != null && midY - defY > 21) overlays.push({ kind: 'gap', y1: defY + 2, y2: midY - 2, label: 'espaço entre linhas' });
  const gkO = P.find(p => p.b.group === 'GK');
  if (dl >= 3 && gkO && gkO.oop && gkO.oop.id !== 'swk' && loe < 2) overlays.push({ kind: 'behind', y1: 8, y2: (defY || dLine) - 2, label: 'espaço nas costas sem cobertura' });
  return { P, ball, opp, overlays, dLine, front, bs };
}

// ---------- texto de cada fase
function phaseText(t, k, M) {
  const v = (x) => tiLabel(x, tiVal(t, x));
  const T = {
    build: `Build-up ${v('buildup')}, Goal Kicks ${v('goal_kicks')}. ${tiVal(t,'buildup') === 2 || tiVal(t,'goal_kicks') === 2 || tiVal(t,'passing') >= 3 ? 'A bola salta o meio-campo directamente para a frente.' : 'Saída curta: os centrais abrem e um médio baixa para receber.'}`,
    prog: `Passing ${v('passing')}, Tempo ${v('tempo')}, Pass Reception ${v('reception')}. ${tiVal(t,'passing') >= 3 ? 'Jogo vertical: procura o avançado cedo.' : 'Circula pelo meio e pelo corredor do lado da bola.'}`,
    final: `Attacking Width ${v('width')}, Supporting Runs ${v('support_runs')}, Crossing ${v('crossing')}. Repara em quem fica atrás (rest-defence) e em quem ataca a área.`,
    loss: `Defensive Transition ${v('def_trans')}. ${tiVal(t,'def_trans') === 2 ? 'Contra-pressão: os mais próximos atacam a bola de imediato.' : tiVal(t,'def_trans') === 0 ? 'Reagrupar: a equipa recua para o bloco antes de pressionar.' : 'Os mais próximos pressionam; os restantes recuam.'}`,
    oop: `Line of Engagement ${v('loe')}, Defensive Line ${v('dline')}, Trigger Press ${v('trigger')}, Pressing Trap ${v('press_trap')}. Os jogadores com anel vermelho saem a pressionar.`,
    win: `Attacking Transition ${v('att_trans')}. ${tiVal(t,'att_trans') === 2 ? 'Contra-ataque: os corredores e outlets arrancam e a bola vai longa.' : tiVal(t,'att_trans') === 0 ? 'Hold Shape: a bola volta atrás e a equipa reorganiza a posse.' : 'Saída equilibrada: alguns arrancam, o resto acompanha.'}`,
  };
  const notes = [];
  M.P.forEach(p => {
    [p.ip, p.oop].forEach(r => { if (!r) return; const n = MOVE_NOTES[r.id]; if (n && n[0] === k) notes.push(`<b>${esc(r.ab)}</b> (${esc(p.s.ipCode)}) ${esc(n[1])}`); });
  });
  return { text: T[k], notes };
}

// demonstração de um papel: tática simples com o papel em destaque
function demoTacticForRole(r) {
  const g = r.groups[0];
  const F = { GK:'4-3-3 DM Wide', DC: r.wideCB ? '3-4-2-1' : '4-3-3 DM Wide', D_LR:'4-3-3 DM Wide', WB_LR:'3-4-2-1', DM:'4-3-3 DM Wide', MC:'4-3-3 DM Wide', M_LR:'4-4-2', AMC:'4-2-3-1 Wide', AM_LR:'4-3-3 DM Wide', ST:'4-3-3 DM Wide' }[g];
  const t = newTactic(F, F); t.name = 'Demo: ' + r.name; t.ment = 'Balanced';
  let idx = t.slots.findIndex(s => { const a = slotInfo(F, s.ipCode); return a.group === g && (!r.wideCB || a.wideCB) && a.side !== 'R'; });
  if (idx < 0) idx = t.slots.findIndex(s => slotInfo(F, s.ipCode).group === g);
  t.slots[idx][r.phase === 'IP' ? 'ip' : 'oop'] = r.id;
  return { t, idx };
}
const TI_PHASE = { buildup:0, goal_kicks:0, gk_speed:0, gk_target:0, passing:1, tempo:1, reception:1, progress:1, support_runs:2, width:2, dribbling:1, patience:2, crossing:2, long_shots:2, creative:2,
  def_trans:3, loe:4, dline:4, dl_behaviour:4, trigger:4, press_trap:4, short_gk:4, tackling:4, cross_eng:4, att_trans:5, timewaste:2, set_pieces:2 };
const SIM_TI = new Set(['buildup','goal_kicks','passing','tempo','width','support_runs','progress','att_trans','loe','dline','dl_behaviour','trigger','press_trap','short_gk','def_trans']);
