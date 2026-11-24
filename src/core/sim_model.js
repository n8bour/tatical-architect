// ============================================================
// Campo animado: modelo tático por fase (formação + papéis + instruções)
// e cenários realistas individuais para cada um dos 77 papéis (IP e OOP).
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

// Movimento base de cada papel com bola. y = profundidade; x = palavra-chave de corredor (ver xKey)
const MOVES = {
  gk_ip:{ build:{y:5}, prog:{y:11}, final:{y:16} },
  bpgk:{ build:{y:11}, prog:{y:18}, final:{y:24} },
  nngk:{ build:{y:4}, prog:{y:8}, final:{y:12} },
  cb_ip:{ build:{y:13, xs:1.4}, prog:{y:26}, final:{y:38} },
  bpcb:{ build:{y:14, xs:1.45}, prog:{y:30}, final:{y:42} },
  nncb:{ build:{y:11, xs:1.22}, prog:{y:21}, final:{y:32} },
  acb:{ build:{y:16}, prog:{y:42, x:'in'}, final:{y:55, x:'in'} },
  ocb:{ build:{y:15, xs:1.55}, prog:{y:48, x:'wide'}, final:{y:78, x:'wide'} },
  fb_ip:{ build:{y:18, x:'wide'}, prog:{y:33, x:'wide'}, final:{y:48, x:'wide'} },
  ifb:{ build:{y:13, x:'cbw'}, prog:{y:24, x:'cbw'}, final:{y:36, x:'cbw'} },
  wb_ip:{ build:{y:24, x:'wide'}, prog:{y:52, x:'wide'}, final:{y:80, x:'wide'} },
  iwb:{ build:{y:27, x:'half'}, prog:{y:42, x:'half'}, final:{y:54, x:'half'} },
  pwb:{ build:{y:29, x:'half'}, prog:{y:48, x:'half'}, final:{y:64, x:'half'} },
  awb:{ build:{y:29, x:'wide'}, prog:{y:64, x:'wide'}, final:{y:88, x:'wide'} },
  dm_ip:{ build:{y:27}, prog:{y:38}, final:{y:46} },
  dlp:{ build:{y:24}, prog:{y:39}, final:{y:53} },
  hb:{ build:{y:12, x:'c'}, prog:{y:22, x:'c'}, final:{y:35, x:'c'} },
  b2b:{ build:{y:33}, prog:{y:56}, final:{y:85, x:'in'} },
  b2bp:{ build:{y:34}, prog:{y:54}, final:{y:74, x:'half'} },
  cm_ip:{ build:{y:39}, prog:{y:52}, final:{y:66} },
  mp:{ build:{y:37}, prog:{y:50, x:'half'}, final:{y:64, x:'half'} },
  wcm:{ build:{y:41}, prog:{y:58, x:'wide2'}, final:{y:73, x:'wide2'} },
  chm:{ build:{y:42}, prog:{y:60, x:'ch'}, final:{y:87, x:'ch'} },
  ap:{ build:{y:47}, prog:{y:63, x:'half'}, final:{y:76, x:'half'} },
  am_ip:{ build:{y:49}, prog:{y:66}, final:{y:84, x:'in'} },
  wm_ip:{ build:{y:44, x:'wide'}, prog:{y:60, x:'wide'}, final:{y:73, x:'wide'} },
  w_ip:{ build:{y:50, x:'wide'}, prog:{y:71, x:'wide'}, final:{y:92, x:'wide'} },
  iw_ip:{ build:{y:50, x:'wide2'}, prog:{y:66, x:'half'}, final:{y:78, x:'half'} },
  pw:{ build:{y:48, x:'wide2'}, prog:{y:62, x:'half'}, final:{y:73, x:'half'} },
  if_ip:{ build:{y:52, x:'wide2'}, prog:{y:71, x:'half'}, final:{y:90, x:'in2'} },
  wf_ip:{ build:{y:52, x:'wide'}, prog:{y:72, x:'wide2'}, final:{y:90, x:'wide3'} },
  fr:{ build:{y:50}, prog:{y:64, x:'ball'}, final:{y:77, x:'ballhalf'} },
  ss:{ build:{y:56}, prog:{y:75}, final:{y:90, x:'in'} },
  cf_ip:{ build:{y:64}, prog:{y:78}, final:{y:90} },
  chf:{ build:{y:65}, prog:{y:80, x:'ch'}, final:{y:92, x:'ch'} },
  dlf:{ build:{y:64}, prog:{y:59}, final:{y:74} },
  f9:{ build:{y:63, x:'c'}, prog:{y:54, x:'c'}, final:{y:67, x:'c'} },
  poa:{ build:{y:68}, prog:{y:85}, final:{y:95} },
  tf:{ build:{y:65}, prog:{y:81, x:'c'}, final:{y:91, x:'c'} },
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
  const mi = MENTALITIES.indexOf(t.ment), ms = (mi - 3) * 1.8;
  const v = (k) => tiVal(t, k);
  const W = [0.68, 0.84, 1.0, 1.12, 1.24][v('width')];
  const prog = v('progress'); // 0:Left, 1:Balanced, 2:Centre, 3:Right
  const bs = prog === 3 ? 1 : prog === 2 ? 0 : -1;
  const flankSide = bs || -1;
  const passV = v('passing'), buildV = v('buildup'), gkKick = v('goal_kicks'), gkTgt = v('gk_target');
  const recV = v('reception'), supV = v('support_runs'), dribV = v('dribbling'), patV = v('patience');
  const lsV = v('long_shots'), crossV = v('crossing'), creV = v('creative');

  const P = t.slots.map((s, i) => {
    const a = slotInfo(t.ipForm, s.ipCode), b = slotInfo(t.oopForm, s.oopCode);
    const sg = a.x < 45 ? -1 : a.x > 55 ? 1 : 0;
    return { i, s, a, b, ip: ROLE_BY_ID[s.ip], oop: ROLE_BY_ID[s.oop], sg, sgb: b.x < 45 ? -1 : b.x > 55 ? 1 : 0, pos: {} };
  });
  const sideOr = (p, alt) => p.sg || (p.a.x < 50 ? -1 : p.a.x > 50 ? 1 : alt);
  const xKey = (p, key, x) => {
    const s = sideOr(p, flankSide);
    switch (key) {
      case 'wide': return clamp(50 + s * 44 * W, 5, 95);
      case 'wide2': return clamp(50 + s * 36 * W, 7, 93);
      case 'wide3': return clamp(50 + s * 29 * W, 9, 91);
      case 'half': return clamp(50 + s * 18 * Math.sqrt(W), 14, 86);
      case 'in': return 50 + s * 9;
      case 'in2': return 50 + s * 13;
      case 'ch': return 50 + s * 23;
      case 'cbw': return 50 + s * 29;
      case 'c': return 50;
      case 'ball': return 50 + bs * 15;
      case 'ballhalf': return 50 + bs * 20;
      default: return x;
    }
  };
  const hasHB = P.some(p => p.ip && p.ip.id === 'hb');
  const hasDDM = P.some(p => p.oop && p.oop.id === 'ddm');
  const arrows = { build: [], prog: [], final: [], loss: [], oop: [], win: [] };
  const zones = { build: [], prog: [], final: [], loss: [], oop: [], win: [] };
  const lines = { build: [], prog: [], final: [], loss: [], oop: [], win: [] };

  // ---- fases com bola
  ['build', 'prog', 'final'].forEach(ph => {
    const yBase = { build: (y) => y * 0.6 + 3, prog: (y) => y * 0.85 + 8, final: (y) => Math.min(96, y * 0.75 + 25) }[ph];
    P.forEach(p => {
      let x = 50 + (p.a.x - 50) * W, y = yBase(p.a.y);
      const m = p.ip && MOVES[p.ip.id] && MOVES[p.ip.id][ph];
      if (m) {
        if (m.y != null) y = m.y;
        if (m.xs) x = 50 + (p.a.x - 50) * m.xs * (hasHB ? 1.25 : 1);
        if (m.x) x = xKey(p, m.x, x);
      }
      // Build-up & Goal Kicks
      if (ph === 'build') {
        if (buildV === 0) { // Play Through Press
          if (p.a.group === 'GK') y += 6;
          if (p.a.group === 'DC') x = 50 + (x - 50) * 1.24;
          if (p.a.group === 'DM') y -= 5;
        } else if (buildV === 2 || gkKick === 2) { // Bypass Press / Long Goal Kicks
          if (p.a.group === 'DC') x = 50 + (x - 50) * 0.85;
          if (['DM','MC','M_LR','AMC'].includes(p.a.group)) y += 9;
          if (p.a.group === 'ST') y += 7;
        }
        if (gkKick === 0 && p.a.group === 'DC') { y -= 4; x = 50 + (x - 50) * 1.15; }
      }
      // Passing directness & Pass reception
      if (ph === 'prog') {
        if (passV <= 1 && p.a.group !== 'GK') {
          const pull = passV === 0 ? 0.22 : 0.12;
          x += (50 + bs * 18 - x) * pull;
          if (['ST','AMC'].includes(p.a.group)) y -= passV === 0 ? 6 : 3;
        }
        if (passV >= 3 && p.a.group !== 'GK') {
          if (['ST','AM_LR'].includes(p.a.group)) y += passV === 4 ? 9 : 5;
          if (p.a.group === 'DC') y -= 3;
        }
        if (recV === 0 && ['ST','AM_LR','AMC','WB_LR','D_LR'].includes(p.a.group)) y += 9; // Pass into Space
        if (recV === 2 && ['ST','AM_LR','AMC','MC','WB_LR','D_LR'].includes(p.a.group)) y -= 6; // Pass to Feet
        if (dribV === 2 && ['AM_LR','M_LR','WB_LR','D_LR'].includes(p.a.group) && sideOr(p, flankSide) === flankSide) y += 11;
        if (prog === 2 && ['MC','AMC','AM_LR','M_LR'].includes(p.a.group)) x = 50 + (x - 50) * 0.72; // Focus Centre
        if (creV === 2 && (['MC','AMC','AM_LR'].includes(p.a.group) || (p.ip && p.ip.tags.includes('CR')))) {
          x += (50 + flankSide * 16 - x) * 0.42;
          y += 6;
        }
        if (creV === 0 && ['MC','AMC','AM_LR'].includes(p.a.group)) x = p.a.x; // Disciplined: rigid lanes
      }
      // Supporting runs (Overlap vs Underlap) in progression & final third
      if (ph === 'final' || ph === 'prog') {
        const isFB = ['D_LR','WB_LR'].includes(p.a.group) && (!p.ip || !['ifb','iwb','pwb'].includes(p.ip.id));
        const isWideAtt = ['AM_LR','M_LR'].includes(p.a.group);
        if (supV === 2) { // Overlap: FB goes wide & high, winger tucks into half-space
          if (isFB) { x = xKey(p, 'wide'); y += ph === 'final' ? 11 : 7; }
          if (isWideAtt && ph === 'final') x = xKey(p, 'half');
        } else if (supV === 0) { // Underlap: winger stays glued wide, FB cuts into half-space
          if (isFB) { x = xKey(p, 'half'); y += ph === 'final' ? 8 : 5; }
          if (isWideAtt) x = xKey(p, 'wide');
        }
      }
      // Final third patience / long shots
      if (ph === 'final') {
        if (patV === 0 && ['D_LR','WB_LR'].includes(p.a.group)) y = clamp(y - 18, 54, 64);
        if (patV === 2 && ['MC','AMC','AM_LR'].includes(p.a.group)) {
          y = p.a.group === 'AM_LR' ? clamp(y - 4, 78, 86) : clamp(y + 4, 70, 84);
          x = 50 + (x - 50) * 0.55;
        }
        if (lsV === 2 && ['MC','AMC'].includes(p.a.group)) y = clamp(y, 68, 76);
      }
      if (p.a.group !== 'GK') y += ms;
      p.pos[ph] = { x: clamp(x, 5, 95), y: clamp(y, 2, 96) };
    });
  });

  // ---- bola e setas com bola
  const byGroup = (gs) => P.filter(p => gs.includes(p.a.group));
  const gk = P.find(p => p.a.group === 'GK');
  const cbs = byGroup(['DC']).sort((p, q) => Math.abs(p.pos.build.x - (50 + flankSide * 30)) - Math.abs(q.pos.build.x - (50 + flankSide * 30)));
  const fbs = byGroup(['D_LR','WB_LR']).sort((p, q) => (sideOr(p, flankSide) === flankSide ? 0 : 1) - (sideOr(q, flankSide) === flankSide ? 0 : 1));
  const mids = byGroup(['DM','MC']).sort((p, q) => (q.ip && q.ip.tags.includes('CR') ? 1 : 0) - (p.ip && p.ip.tags.includes('CR') ? 1 : 0) || p.pos.build.y - q.pos.build.y);
  const strikers = byGroup(['ST','AMC']).sort((p, q) => (q.ip && q.ip.tags.includes('AIR') ? 1 : 0) - (p.ip && p.ip.tags.includes('AIR') ? 1 : 0) || q.pos.final.y - p.pos.final.y);
  const directBuild = buildV === 2 || gkKick === 2 || gkTgt === 4 || passV >= 4;
  const directProg = passV >= 3 || buildV === 2;
  const holder = mids[0] || cbs[0] || gk;
  const mid2 = mids[1] || byGroup(['AMC','MC','DM']).find(p => p !== holder) || holder;
  const sideWide = P.filter(p => p.a.group !== 'GK' && sideOr(p, flankSide) === flankSide)
    .sort((p, q) => (Math.abs(q.pos.final.x - 50) > 28 && q.pos.final.y > 58 ? 1 : 0) - (Math.abs(p.pos.final.x - 50) > 28 && p.pos.final.y > 58 ? 1 : 0) || q.pos.final.y - p.pos.final.y);
  const wideProv = sideWide[0];
  const boxTarget = [...P].filter(p => p !== wideProv && p.a.group !== 'GK' && ((p.ip && (p.ip.tags.includes('BOX') || p.ip.tags.includes('AIR'))) || p.a.group === 'ST'))
    .sort((p, q) => (q.a.group === 'ST' ? 1 : 0) - (p.a.group === 'ST' ? 1 : 0) || q.pos.final.y - p.pos.final.y)[0] || strikers[0] || holder;
  const creator = [...P].filter(p => p.ip && p.ip.tags.includes('CR') && p.a.group !== 'GK').sort((p, q) => q.pos.final.y - p.pos.final.y)[0] || mid2;
  const runner = [...P].filter(p => p !== wideProv && p.ip && p.ip.tags.includes('RUN')).sort((p, q) => q.pos.final.y - p.pos.final.y)[0] || boxTarget;
  const shooter = byGroup(['AMC','MC']).sort((p, q) => q.pos.final.y - p.pos.final.y)[0] || creator;
  const earlyCrosser = fbs[0] || wideProv;
  const at = (p, ph, dx = 0, dy = 0, kind = 'pass') => p ? { x: clamp(p.pos[ph].x + dx, 4, 96), y: clamp(p.pos[ph].y + dy, 2, 99), kind } : { x: 50, y: 50, kind };

  const ball = {};
  const dedup = (pts) => pts.filter((pt, idx) => idx === 0 || Math.hypot(pt.x - pts[idx - 1].x, pt.y - pts[idx - 1].y) > 1.5);
  // Build-up ball path + track who ends with the ball (buildRec) so Progression starts from them
  let buildRec = holder;
  if (buildV === 2 && gkKick !== 2 && gkTgt !== 4) {
    // Bypass Press in open play: CB receives and clips a long ball over the opponent press
    buildRec = wideProv || strikers[0] || runner;
    ball.build = dedup([at(gk, 'build', 0, 1.5), at(cbs[0] || holder, 'build'), at(buildRec, 'build', 0, 0, 'loft')]);
  } else if (directBuild) {
    buildRec = strikers[0] || runner;
    ball.build = dedup([at(gk, 'build', 0, 1.5, 'loft'), at(buildRec, 'build', 0, 0, 'loft')]);
  } else if (gkTgt === 1 && fbs[0]) {
    buildRec = mid2;
    ball.build = dedup([at(gk, 'build', 0, 1.5, 'pass'), at(fbs[0], 'build'), at(buildRec, 'build')]);
  } else if (gkTgt === 3 && wideProv) {
    buildRec = wideProv;
    ball.build = dedup([at(gk, 'build', 0, 1.5, 'loft'), at(buildRec, 'build', 0, 0, 'loft')]);
  } else if (buildV === 0) {
    buildRec = mid2 || holder;
    ball.build = dedup([at(gk, 'build', 0, 1.5), at(cbs[0] || holder, 'build'), at(holder, 'build'), at(buildRec, 'build')]);
  } else if (passV <= 1 || gkKick === 0) {
    buildRec = fbs[0] || mid2;
    ball.build = dedup([at(gk, 'build', 0, 1.5), at(cbs[0] || holder, 'build'), at(buildRec, 'build')]);
  } else {
    buildRec = holder;
    ball.build = dedup([at(gk, 'build', 0, 1.5), at(cbs[0] || holder, 'build'), at(buildRec, 'build')]);
  }
  if (ball.build.length < 2) ball.build.push(at(holder, 'build', 2, 2));

  // Progression ball path: starts from buildRec and feeds the exact player who launches Final Third
  const progCenter = byGroup(['AMC','MC']).sort((p, q) => Math.abs(p.pos.prog.x - 50) - Math.abs(q.pos.prog.x - 50) || q.pos.prog.y - p.pos.prog.y)[0] || creator;
  const flankMid = byGroup(['M_LR','AM_LR']).find(p => sideOr(p, flankSide) === flankSide);
  const patHub = byGroup(['AMC','MC']).sort((p, q) => Math.abs(p.pos.final.x - 50) - Math.abs(q.pos.final.x - 50) || q.pos.final.y - p.pos.final.y)[0] || creator;
  const progTarget = lsV === 2 ? shooter
    : patV === 0 ? earlyCrosser
    : patV === 2 ? patHub
    : prog === 2 ? progCenter
    : (supV !== 1 && flankMid && fbs[0] && fbs[0].pos.final.y >= flankMid.pos.final.y - 6) ? flankMid
    : wideProv || progCenter || strikers[0];
  let progRec = progTarget || strikers[0] || holder;
  if (directBuild && buildRec === (strikers[0] || runner)) {
    const flick = P.filter(p => p !== buildRec && p.a.group !== 'GK' && p.pos.prog.y >= buildRec.pos.prog.y - 12)
      .sort((p, q) => q.pos.prog.y - p.pos.prog.y)[0];
    if (flick) {
      progRec = flick;
      ball.prog = dedup([at(buildRec, 'prog'), at(progRec, 'prog', 0, recV === 0 ? 5 : 0)]);
    } else {
      progRec = buildRec;
      ball.prog = dedup([at(buildRec, 'prog', 0, 0, 'dribble'), at(buildRec, 'prog', 0, 6, 'dribble')]);
    }
  } else if (directProg) {
    progRec = recV === 0 ? runner : (strikers[0] || runner);
    const tgt = recV === 0 ? at(progRec, 'prog', 0, 6, 'loft') : at(progRec, 'prog', 0, 0, 'loft');
    ball.prog = dedup([at(buildRec, 'prog', 0, 0, 'loft'), tgt]);
  } else if (dribV === 2 && progTarget) {
    progRec = progTarget;
    const midPt = at(progTarget, 'prog', 0, -14, 'dribble');
    ball.prog = dedup([at(buildRec, 'prog'), ...(midPt.y >= buildRec.pos.prog.y - 4 ? [midPt] : []), at(progTarget, 'prog', 0, 2, 'dribble')]);
    arrows.prog.push({ from: { x: progTarget.pos.prog.x, y: Math.max(20, progTarget.pos.prog.y - 14) }, to: progTarget.pos.prog, color: '#ffe27a', kind: 'run' });
  } else if (creV === 2 && creator && runner && creator !== buildRec) {
    progRec = runner;
    ball.prog = dedup([at(buildRec, 'prog'), ...(creator.pos.prog.y >= buildRec.pos.prog.y - 2 ? [at(creator, 'prog')] : []), at(runner, 'prog', 0, 4, 'curve')]);
  } else if (dribV === 0 && mid2 && mid2 !== buildRec && mid2 !== progTarget) {
    progRec = progTarget || mid2;
    const pts = [at(buildRec, 'prog')];
    if (mid2.pos.prog.y >= buildRec.pos.prog.y - 2 && mid2.pos.prog.y <= progRec.pos.prog.y + 2) pts.push(at(mid2, 'prog'));
    pts.push(at(progRec, 'prog'));
    ball.prog = dedup(pts);
  } else if (passV <= 1) {
    progRec = progTarget || mid2;
    const pts = [at(buildRec, 'prog')];
    if (mid2 && mid2 !== buildRec && mid2 !== progRec && mid2.pos.prog.y >= buildRec.pos.prog.y - 2 && mid2.pos.prog.y <= progRec.pos.prog.y + 2) {
      pts.push(at(mid2, 'prog'));
    }
    pts.push(at(progRec, 'prog', 0, recV === 0 ? 10 : 0));
    ball.prog = dedup(pts);
  } else {
    progRec = progTarget || strikers[0] || holder;
    ball.prog = dedup([at(buildRec, 'prog'), at(progRec, 'prog', 0, recV === 0 ? 10 : 0)]);
  }
  if (ball.prog.length < 2) ball.prog.push(at(progRec, 'prog', 0, 6));

  // Final third ball path: continues strictly forward from progRec (zero backward jumps)
  const fStart = at(progRec, 'final');
  const fwdPts = (pts) => dedup(pts.reduce((acc, pt) => {
    if (!acc.length || pt.y >= acc[acc.length - 1].y - 6) acc.push(pt);
    return acc;
  }, []));
  if (lsV === 2) {
    ball.final = fwdPts([fStart, at(shooter, 'final', 0, 0, 'shot'), { x: 50, y: 98, kind: 'shot' }]);
  } else if (patV === 0) {
    const crossFrom = at(earlyCrosser, 'final', 0, 0, 'loft');
    ball.final = fwdPts([progRec === earlyCrosser ? crossFrom : fStart, crossFrom, at(boxTarget, 'final', 0, 2, 'loft'), { x: 50, y: 98, kind: 'shot' }]);
  } else if (patV === 2) {
    const centralLinks = P.filter(p => p !== progRec && p !== boxTarget && p.a.group !== 'GK' && Math.abs(p.pos.final.x - 50) <= 30 && p.pos.final.y >= fStart.y + 2 && p.pos.final.y <= 94)
      .sort((p, q) => p.pos.final.y - q.pos.final.y);
    const l1 = centralLinks[0], l2 = centralLinks.find(p => p !== l1 && p.pos.final.y >= (l1 ? l1.pos.final.y + 2 : fStart.y));
    ball.final = fwdPts([fStart, ...(l1 ? [at(l1, 'final')] : []), ...(l2 ? [at(l2, 'final')] : []), at(boxTarget, 'final', 0, 1, 'pass'), { x: 50, y: 98, kind: 'shot' }]);
  } else if (prog === 2) {
    const c1 = creator || holder, c2 = boxTarget || runner;
    ball.final = fwdPts([fStart, at(c1, 'final'), at(c2, 'final', 0, 2, 'shot'), { x: 50, y: 98, kind: 'shot' }]);
  } else if (wideProv) {
    const crossKind = crossV === 3 ? 'loft' : crossV === 2 ? 'curve' : 'pass';
    const boxPt = crossV === 0 ? at(boxTarget, 'final', -flankSide * 4, -4, 'pass')
      : crossV === 3 ? at(boxTarget, 'final', -flankSide * 9, 2, 'loft')
      : at(boxTarget, 'final', 0, 1, crossKind);
    const crosser = (supV !== 1 && fbs[0] && progRec === flankMid) ? fbs[0] : wideProv;
    ball.final = fwdPts([fStart, at(crosser, 'final', 0, 0, crossKind), boxPt]);
  } else {
    ball.final = fwdPts([fStart, at(creator || holder, 'final'), at(runner, 'final', 0, 2, 'shot'), { x: 50, y: 98, kind: 'shot' }]);
  }
  if (ball.final.length < 2) ball.final.push({ x: 50, y: 98, kind: 'shot' });
  const lastFinal = ball.final[ball.final.length - 1];
  const lost = { x: clamp(lastFinal.x, 8, 92), y: clamp(lastFinal.y, 8, 95) };

  // IP tactical arrows & zones
  P.forEach(p => {
    const id = p.ip && p.ip.id;
    if (id === 'hb') arrows.build.push({ from: { x: 50, y: 28 }, to: p.pos.build, color: '#7dd3fc', kind: 'cover' });
    if (id === 'ifb') arrows.build.push({ from: { x: 50 + p.sg * 40, y: 18 }, to: p.pos.build, color: '#7dd3fc', kind: 'cover' });
    if (id === 'iwb' || id === 'pwb') arrows.build.push({ from: { x: 50 + p.sg * 40, y: 22 }, to: p.pos.build, color: '#ffe27a', kind: 'run' });
    if (id === 'f9' || id === 'dlf') arrows.prog.push({ from: p.pos.build, to: p.pos.prog, color: '#ffe27a', kind: 'run' });
    if (['acb','ocb','wcm','chm','b2b','if_ip','awb','ss','chf'].includes(id)) {
      arrows.final.push({ from: p.pos.prog, to: p.pos.final, color: '#ffe27a', kind: 'run' });
    }
    if ((supV === 2 || supV === 0) && ['D_LR','WB_LR'].includes(p.a.group) && !['ifb','iwb','pwb'].includes(id)) {
      arrows.final.push({ from: p.pos.prog, to: p.pos.final, color: '#ffe27a', kind: 'run' });
    }
  });
  if (buildV === 0) zones.build.push({ kind: 'trap', x1: 22, x2: 78, y1: 10, y2: 34, color: 'rgba(125,211,252,.14)', stroke: 'rgba(125,211,252,.65)', label: L('Sair pelo meio', 'Play through press') });
  if (buildV === 2) zones.build.push({ kind: 'lane', x1: 20, x2: 80, y1: 30, y2: 64, color: 'rgba(255,226,122,.12)', stroke: 'rgba(255,226,122,.6)', label: L('Saltar a pressão', 'Bypass press') });
  if (recV === 0 && progRec) {
    const sp = at(progRec, 'prog', 0, 10);
    arrows.prog.push({ from: progRec.pos.prog, to: sp, color: '#ffe27a', kind: 'run' });
    zones.prog.push({ kind: 'circle', x: sp.x, y: sp.y, r: 9, color: 'rgba(255,226,122,.2)', stroke: 'rgba(255,226,122,.75)', label: L('Espaço livre', 'Into space') });
  }
  if (recV === 2 && progRec) {
    zones.prog.push({ kind: 'circle', x: progRec.pos.prog.x, y: progRec.pos.prog.y, r: 8, color: 'rgba(125,211,252,.2)', stroke: 'rgba(125,211,252,.75)', label: L('Passe ao pé', 'To feet') });
  }
  if (dribV === 2 && progTarget) zones.prog.push({ kind: 'lane', x1: clamp(progTarget.pos.prog.x - 10, 2, 80), x2: clamp(progTarget.pos.prog.x + 10, 20, 98), y1: Math.max(24, progTarget.pos.prog.y - 16), y2: Math.min(92, progTarget.pos.prog.y + 4), color: 'rgba(255,226,122,.15)', stroke: 'rgba(255,226,122,.7)', label: L('Conduzir 1×1', 'Run at defence') });
  if (creV === 2) {
    P.filter(p => ['MC','AMC','AM_LR'].includes(p.a.group)).forEach(p => arrows.prog.push({ from: p.pos.build, to: p.pos.prog, color: '#ffe27a', kind: 'run' }));
    zones.prog.push({ kind: 'trap', x1: 24, x2: 76, y1: 42, y2: 78, color: 'rgba(255,226,122,.12)', stroke: 'rgba(255,226,122,.65)', label: L('Mobilidade livre', 'Roam & create') });
  }
  if (creV === 0) zones.prog.push({ kind: 'lane', x1: 14, x2: 86, y1: 38, y2: 72, color: 'rgba(125,211,252,.1)', stroke: 'rgba(125,211,252,.55)', label: L('Posições rígidas', 'Hold lanes') });
  if (prog === 0) zones.prog.push({ kind: 'lane', x1: 3, x2: 32, y1: 28, y2: 88, color: 'rgba(255,226,122,.13)', stroke: 'rgba(255,226,122,.45)', label: L('Corredor esq.', 'Left flank') });
  if (prog === 3) zones.prog.push({ kind: 'lane', x1: 68, x2: 97, y1: 28, y2: 88, color: 'rgba(255,226,122,.13)', stroke: 'rgba(255,226,122,.45)', label: L('Corredor dir.', 'Right flank') });
  if (prog === 2) zones.prog.push({ kind: 'lane', x1: 31, x2: 69, y1: 28, y2: 88, color: 'rgba(255,226,122,.13)', stroke: 'rgba(255,226,122,.45)', label: L('Zona central', 'Centre') });
  if (v('width') <= 1) zones.final.push({ kind: 'lane', x1: 22, x2: 78, y1: 50, y2: 95, color: 'rgba(125,211,252,.08)', stroke: 'rgba(125,211,252,.35)', label: L('Largura estreita', 'Narrow width') });
  if (v('width') >= 3) zones.final.push({ kind: 'lane', x1: 4, x2: 96, y1: 50, y2: 95, color: 'rgba(255,226,122,.08)', stroke: 'rgba(255,226,122,.35)', label: L('Largura máxima', 'Wide stretch') });
  if (patV === 0) {
    const x1 = flankSide < 0 ? 2 : 80, x2 = flankSide < 0 ? 20 : 98;
    zones.final.push({ kind: 'lane', x1, x2, y1: 52, y2: 68, color: 'rgba(255,226,122,.16)', stroke: 'rgba(255,226,122,.72)', label: L('Cruzamento cedo', 'Early cross') });
  }
  if (patV === 2) zones.final.push({ kind: 'trap', x1: 26, x2: 74, y1: 76, y2: 96, color: 'rgba(125,211,252,.15)', stroke: 'rgba(125,211,252,.7)', label: L('Entrar na área', 'Work into box') });

  // ---- bloco sem bola
  const dl = v('dline'), loe = v('loe'), trig = v('trigger'), dlb = v('dl_behaviour'), crossEng = v('cross_eng');
  const dLine = clamp([14, 20, 27, 34, 41][dl] + ms * 0.55 + (dlb - 1) * 4.5, 10, 48);
  const front = clamp([42, 57, 72][loe] + ms * 0.55, dLine + 18, 82);
  const trap = v('press_trap'); // 0: Inside, 1: Balanced, 2: Outside
  const oppFront = dLine + 8, oppBack = Math.max(82, front + 14);
  const oppAt = (k, D, F) => {
    const [ox, oy] = OPP_SHAPE[k];
    const tt = clamp((oy - 18) / 60, 0, 1);
    return k === 0 ? { x: 100 - ox, y: Math.min(97, D + 10) } : { x: 100 - ox, y: D + (F - D) * tt };
  };
  const oppGK = oppAt(0, oppBack, oppFront);
  const oppStart = v('short_gk') === 1 ? oppGK : oppAt(2, oppBack, oppFront);
  const ballOOPend = v('short_gk') === 1 ? { x: 50 + flankSide * 18, y: clamp(dLine + 24, 38, 56), kind: 'loft' }
    : trap === 2 ? { x: 50 + flankSide * 40, y: front + 2, kind: 'opp' }
    : trap === 0 ? { x: 50, y: front + 1, kind: 'opp' }
    : crossEng === 2 ? { x: 50 + flankSide * 42, y: dLine + 12, kind: 'opp' }
    : { x: 50 + flankSide * 22, y: front + 3, kind: 'opp' };
  ball.oop = [{ ...oppStart, kind: 'opp' }, ballOOPend];
  const bxo = ballOOPend.x;

  P.forEach(p => {
    const b = p.b, r = p.oop;
    let x = 50 + (b.x - 50) * 0.8 + (bxo - 50) * 0.22;
    let y;
    if (p.b.group === 'GK') y = 5 + (dl >= 3 ? 4 : 0);
    else {
      const tt = clamp((b.y - 18) / 62, 0, 1);
      y = dLine + (front - dLine) * tt;
    }
    const id = r && r.id, tags = r ? r.tags : [];
    if (id === 'swk') y = Math.max(9, dLine - 11);
    if (id === 'lhk') y = 3;
    if (hasDDM && p.b.group === 'DC') x = 50 + (p.sgb || (p.b.x < 50 ? -1 : 1)) * 22;
    if (tags.includes('HP')) y += 5;
    if (tags.includes('LOW')) y -= 4;
    if (tags.includes('TRK')) y -= 6;
    if (['pfb','prwb'].includes(id)) y += 7;
    if (id === 'hfb') x = 50 + (p.sgb || flankSide) * 26;
    if (id === 'ddm') { y = dLine; x = 50; }
    if (['wcdm','wccm'].includes(id)) x = 50 + (p.sgb || -flankSide) * 32;
    if (['wowm','wow'].includes(id)) { y = Math.max(y, front + 6); x = 50 + (p.sgb || flankSide) * 42; }
    if (id === 'iow') { y = Math.max(y, front + 6); x = 50 + (p.sgb || flankSide) * 20; }
    if (['coam','cocf'].includes(id)) { y = Math.max(y, front + 8); x = 50; }
    if (['soam','socf'].includes(id)) { y = Math.max(y, front + 9); x = 50 + (p.sgb || flankSide) * 26; }
    if (id === 'tcf') y = front - 2;
    if (v('short_gk') === 1 && ['ST','AM_LR','AMC'].includes(p.b.group)) y = Math.max(y, p.b.group === 'ST' ? 84 : 76);
    if (trap === 0 && ['AM_LR','M_LR'].includes(p.b.group)) x = 50 + (x - 50) * 1.15; // wide players force inside
    if (trap === 2 && ['ST','AMC'].includes(p.b.group)) x = 50 - flankSide * 6;       // central players block inside, force outside
    if (crossEng === 0 && p.b.group === 'D_LR') x = 50 + (p.sgb || flankSide) * 25;  // Invite crosses: tuck narrow in box
    if (crossEng === 2 && p.b.group === 'D_LR' && (p.sgb === flankSide)) { x = 50 + flankSide * 43; y += 7; } // Stop crosses: jump wide
    if (p.b.group !== 'GK') {
      const minOff = { DC:0, D_LR:0, WB_LR:0, DM:7, MC:10, M_LR:10, AMC:15, AM_LR:14, ST:19 }[p.b.group] || 0;
      y = Math.max(y, dLine + minOff - (tags.includes('TRK') ? 4 : 0));
      if (p.b.group === 'M_LR' || p.b.group === 'AM_LR') x = 50 + (x - 50) * 0.94;
    }
    p.pos.oop = { x: clamp(x, 5, 95), y: clamp(y, 2, 96) };
  });

  // pressão: os mais próximos da bola (ou com papel assertivo) saem ao portador
  const nPress = [0, 1, 2, 3, 4][trig];
  const cand = P.filter(p => p.b.group !== 'GK' && !(p.oop && p.oop.tags.includes('OUT')))
    .map(p => ({ p, d: Math.hypot(p.pos.oop.x - bxo, p.pos.oop.y - ballOOPend.y) })).sort((a, b) => a.d - b.d);
  const pressers = new Set(cand.filter(c => c.p.pos.oop.y > dLine + 5).slice(0, nPress).map(c => c.p.i));
  P.forEach(p => { if (p.oop && p.oop.tags.includes('HP') && Math.hypot(p.pos.oop.x - bxo, p.pos.oop.y - ballOOPend.y) < 32) pressers.add(p.i); });
  P.forEach(p => {
    if (pressers.has(p.i)) {
      const o = p.pos.oop;
      const to = { x: clamp(o.x + (bxo - o.x) * 0.56, 5, 95), y: clamp(o.y + (ballOOPend.y - 3 - o.y) * 0.56, 4, 95) };
      arrows.oop.push({ from: o, to, color: '#ff6b5a', kind: 'press' });
      p.pos.oop = to;
      p.pressing = true;
    }
  });

  // OOP tactical lines & zones (defensive line, line of engagement, press trap, step-up/drop-off)
  lines.oop.push({ y: dLine, color: 'rgba(125,211,252,.72)', label: L('Linha def.', 'Def. line') });
  lines.oop.push({ y: front, color: 'rgba(255,145,90,.72)', label: L('Pressão', 'Engagement') });
  if (dlb === 2) {
    P.filter(p => p.b.group === 'DC' || p.b.group === 'D_LR').forEach(p => arrows.oop.push({ from: { x: p.pos.oop.x, y: p.pos.oop.y - 7 }, to: p.pos.oop, color: '#ff6b5a', kind: 'press' }));
    zones.oop.push({ kind: 'lane', x1: 8, x2: 92, y1: dLine - 2, y2: dLine + 5, color: 'rgba(255,107,90,.16)', stroke: 'rgba(255,107,90,.72)', label: L('Subir / fora-de-jogo', 'Step up / offside') });
  }
  if (dlb === 0) {
    P.filter(p => p.b.group === 'DC' || p.b.group === 'D_LR').forEach(p => arrows.oop.push({ from: { x: p.pos.oop.x, y: p.pos.oop.y + 7 }, to: p.pos.oop, color: '#7dd3fc', kind: 'cover' }));
    zones.oop.push({ kind: 'lane', x1: 8, x2: 92, y1: Math.max(6, dLine - 10), y2: dLine, color: 'rgba(125,211,252,.16)', stroke: 'rgba(125,211,252,.72)', label: L('Proteger costas', 'Protect behind') });
  }
  if (trap === 0) zones.oop.push({ kind: 'trap', x1: 32, x2: 68, y1: Math.max(dLine + 6, front - 12), y2: front + 8, color: 'rgba(255,107,90,.18)', stroke: 'rgba(255,107,90,.68)', label: L('Armadilha interior', 'Trap inside') });
  if (trap === 2) {
    const x1 = flankSide < 0 ? 3 : 72, x2 = flankSide < 0 ? 28 : 97;
    zones.oop.push({ kind: 'trap', x1, x2, y1: Math.max(dLine + 6, front - 10), y2: front + 10, color: 'rgba(255,107,90,.18)', stroke: 'rgba(255,107,90,.68)', label: L('Armadilha fora', 'Trap outside') });
  }
  if (v('short_gk') === 1) {
    P.filter(p => ['ST','AM_LR','AMC'].includes(p.b.group)).forEach(p => arrows.oop.push({ from: { x: p.pos.oop.x, y: p.pos.oop.y - 9 }, to: p.pos.oop, color: '#ff6b5a', kind: 'press' }));
    zones.oop.push({ kind: 'trap', x1: 16, x2: 84, y1: 74, y2: 96, color: 'rgba(255,107,90,.16)', stroke: 'rgba(255,107,90,.72)', label: L('Forçar pontapé longo', 'Force long kick') });
  }

  // ---- perda da bola
  const dtr = v('def_trans');
  P.forEach(p => {
    const f = p.pos.final, o = p.pos.oop, d = Math.hypot(f.x - lost.x, f.y - lost.y);
    let pt;
    if (dtr === 2 && d < 30 && p.a.group !== 'GK') {
      pt = { x: f.x + (lost.x - f.x) * 0.72, y: f.y + (lost.y - 4 - f.y) * 0.72 };
      arrows.loss.push({ from: f, to: pt, color: '#ff6b5a', kind: 'press' });
    } else if (dtr === 2) {
      pt = { x: f.x, y: clamp(f.y - 4, 4, 95) };
    } else if (dtr === 0) {
      pt = { x: f.x + (o.x - f.x) * 0.8, y: f.y + (o.y - f.y) * 0.8 };
      if (p.a.group !== 'GK') arrows.loss.push({ from: f, to: pt, color: '#7dd3fc', kind: 'cover' });
    } else {
      pt = d < 18 && p.a.group !== 'GK'
        ? { x: f.x + (lost.x - f.x) * 0.6, y: f.y + (lost.y - f.y) * 0.6 }
        : { x: f.x + (o.x - f.x) * 0.45, y: f.y + (o.y - f.y) * 0.45 };
    }
    p.pos.loss = { x: clamp(pt.x, 5, 95), y: clamp(pt.y, 2, 96) };
  });
  if (dtr === 2) zones.loss.push({ kind: 'circle', x: lost.x, y: lost.y, r: 16, color: 'rgba(255,107,90,.16)', stroke: 'rgba(255,107,90,.7)', label: L('Contra-pressão', 'Counter-press') });
  const lossEnd = { x: clamp(lost.x - flankSide * 7, 8, 92), y: clamp(lost.y - (dtr === 0 ? 22 : 9), Math.max(front + 4, 24), 92), kind: 'opp' };
  ball.loss = [{ ...lost, kind: 'opp' }, lossEnd];
  // Chain loss -> oop smoothly instead of teleporting to an unrelated opponent CB
  ball.oop = dedup([lossEnd, { ...oppStart, kind: 'opp' }, ballOOPend]);
  if (ball.oop.length < 2) ball.oop = [{ ...oppStart, kind: 'opp' }, ballOOPend];

  // ---- recuperação
  const atr = v('att_trans');
  P.forEach(p => {
    const o = p.pos.oop, r = p.ip, ro = p.oop;
    const runs = (r && (r.tags.includes('RUN') || r.tags.includes('BOX'))) || (ro && ro.tags.includes('OUT')) || p.a.group === 'ST';
    let pt = { ...o };
    if (atr === 2 && runs) {
      pt = { x: o.x + (p.pos.final.x - o.x) * 0.45, y: Math.min(93, o.y + 26) };
      arrows.win.push({ from: o, to: pt, color: '#ffe27a', kind: 'run' });
    } else if (atr === 2) {
      pt = { x: o.x, y: Math.min(92, o.y + 7) };
    } else if (atr === 0) {
      pt = { x: o.x + (p.pos.build.x - o.x) * 0.62, y: o.y + (p.pos.build.y - o.y) * 0.62 };
      if (p.a.group !== 'GK') arrows.win.push({ from: o, to: pt, color: '#7dd3fc', kind: 'cover' });
    } else {
      pt = runs ? { x: o.x, y: Math.min(90, o.y + 13) } : { x: o.x, y: Math.min(90, o.y + 4) };
    }
    p.pos.win = { x: clamp(pt.x, 5, 95), y: clamp(pt.y, 2, 96) };
  });
  // Chain oop -> win right from where the opponent lost the ball (ballOOPend)
  const target = atr === 0 ? (cbs[0] || holder) : [...P].sort((p, q) => q.pos.win.y - p.pos.win.y)[0];
  ball.win = dedup([{ x: ballOOPend.x, y: ballOOPend.y, kind: atr === 2 ? 'loft' : 'pass' }, at(target, 'win', 0, atr === 2 ? 2 : 0, atr === 2 ? 'loft' : 'pass')]);
  if (ball.win.length < 2) ball.win.push(at(target, 'win', 0, 4));

  // ---- adversário (cinzento)
  const OPP = { build: [64, 30], prog: [74, 44], final: [86, 60], loss: [80, 46], oop: [oppBack, oppFront], win: [70, dLine + 4] };
  const opp = {};
  Object.entries(OPP).forEach(([ph, [D, F]]) => { opp[ph] = OPP_SHAPE.map((_, k) => oppAt(k, D, F)); });

  // ---- avisos visuais no bloco
  const lineY = (gs) => { const q = P.filter(p => gs.includes(p.b.group) && !p.pressing); return q.length ? q.reduce((a, p) => a + p.pos.oop.y, 0) / q.length : null; };
  const defY = lineY(['DC']), midY = lineY(['DM','MC','M_LR']);
  const overlays = [];
  if (defY != null && midY != null && midY - defY > 21) overlays.push({ kind: 'gap', y1: defY + 2, y2: midY - 2, label: L('espaço entre linhas', 'gap between the lines') });
  const gkO = P.find(p => p.b.group === 'GK');
  if (dl >= 3 && gkO && gkO.oop && gkO.oop.id !== 'swk' && loe < 2) overlays.push({ kind: 'behind', y1: 8, y2: (defY || dLine) - 2, label: L('espaço nas costas sem cobertura', 'uncovered space in behind') });
  return { P, ball, opp, overlays, arrows, zones, lines, dLine, front, bs };
}

// ---------- texto de cada fase
function phaseText(t, k, M) {
  const lbl = (key) => tiLabel(key, tiVal(t, key));
  const progLbl = tiVal(t, 'progress');
  const progTxt = progLbl === 2
    ? L('Foco pelo centro: o meio-campo junta-se para progredir pela zona 14.', 'Focus through the centre: midfielders combine through Zone 14.')
    : progLbl === 3 ? L('Progressão concentrada no corredor direito.', 'Progression concentrated down the right flank.')
    : progLbl === 0 ? L('Progressão concentrada no corredor esquerdo.', 'Progression concentrated down the left flank.')
    : L('Circula pelo meio e pelos corredores de forma equilibrada.', 'Circulates through the middle and flanks evenly.');
  const TX = {
    build: L(`Build-up ${lbl('buildup')}, Goal Kicks ${lbl('goal_kicks')}. ` + (tiVal(t,'buildup') === 2 || tiVal(t,'goal_kicks') === 2 || tiVal(t,'passing') >= 4 ? 'A bola salta o meio-campo em passe longo directamente para a frente.' : tiVal(t,'buildup') === 0 ? 'Saída curta através da pressão: GR sobe, centrais abrem e o médio baixa para triangulação.' : 'Saída equilibrada: centrais abrem e ligam com o médio defensivo.'),
             `Build-up ${lbl('buildup')}, Goal Kicks ${lbl('goal_kicks')}. ` + (tiVal(t,'buildup') === 2 || tiVal(t,'goal_kicks') === 2 || tiVal(t,'passing') >= 4 ? 'The ball skips midfield with a long pass straight forward.' : tiVal(t,'buildup') === 0 ? 'Playing through the press: GK steps up, CBs split wide and DM drops for short triangles.' : 'Balanced build-up: CBs split and connect into midfield.')),
    prog: L(`Passing ${lbl('passing')}, Tempo ${lbl('tempo')}, Pass Reception ${lbl('reception')}. ` + (tiVal(t,'passing') >= 3 ? 'Jogo vertical: procura o avançado cedo. ' : '') + progTxt,
            `Passing ${lbl('passing')}, Tempo ${lbl('tempo')}, Pass Reception ${lbl('reception')}. ` + (tiVal(t,'passing') >= 3 ? 'Vertical play: look for the striker early. ' : '') + progTxt),
    final: L(`Attacking Width ${lbl('width')}, Supporting Runs ${lbl('support_runs')}, Crossing ${lbl('crossing')}. ` + (tiVal(t,'support_runs') === 2 ? 'Os laterais sobrepõem por fora enquanto os extremos fecham por dentro.' : tiVal(t,'support_runs') === 0 ? 'Os extremos seguram a linha e os laterais atacam o meio-espaço (underlap).' : 'Repara nas setas douradas (desmarcações) e em quem fica na rest-defence.'),
             `Attacking Width ${lbl('width')}, Supporting Runs ${lbl('support_runs')}, Crossing ${lbl('crossing')}. ` + (tiVal(t,'support_runs') === 2 ? 'Full-backs overlap outside while wingers tuck inside.' : tiVal(t,'support_runs') === 0 ? 'Wingers hold the touchline while full-backs underlap into the half-space.' : 'Watch the gold run arrows and who holds rest-defence.')),
    loss: L(`Defensive Transition ${lbl('def_trans')}. ` + (tiVal(t,'def_trans') === 2 ? 'Contra-pressão: os mais próximos fecham de imediato sobre a bola (anel vermelho).' : tiVal(t,'def_trans') === 0 ? 'Reagrupar: a equipa recua de imediato para o bloco (setas azuis).' : 'Os mais próximos pressionam; os restantes recuam.'),
            `Defensive Transition ${lbl('def_trans')}. ` + (tiVal(t,'def_trans') === 2 ? 'Counter-press: the nearest players swarm the ball immediately (red zone).' : tiVal(t,'def_trans') === 0 ? 'Regroup: the team drops immediately into its block (blue arrows).' : 'The nearest players press; the rest drop back.')),
    oop: L(`Line of Engagement ${lbl('loe')}, Defensive Line ${lbl('dline')}, Trigger Press ${lbl('trigger')}, Pressing Trap ${lbl('press_trap')}. As linhas tracejadas mostram o bloco; anéis vermelhos saem a pressionar.`,
           `Line of Engagement ${lbl('loe')}, Defensive Line ${lbl('dline')}, Trigger Press ${lbl('trigger')}, Pressing Trap ${lbl('press_trap')}. Dashed lines mark the defensive block; red rings step out to press.`),
    win: L(`Attacking Transition ${lbl('att_trans')}. ` + (tiVal(t,'att_trans') === 2 ? 'Contra-ataque: corredores e outlets arrancam (setas douradas) e a bola vai nas costas.' : tiVal(t,'att_trans') === 0 ? 'Hold Shape: passe seguro para trás e a equipa reorganiza a posse.' : 'Saída equilibrada: alguns arrancam, o resto acompanha.'),
           `Attacking Transition ${lbl('att_trans')}. ` + (tiVal(t,'att_trans') === 2 ? 'Counter-attack: runners and outlets sprint forward (gold arrows) and the ball goes in behind.' : tiVal(t,'att_trans') === 0 ? 'Hold Shape: safe pass back while the team settles into possession.' : 'A balanced break: some run, the rest follow.')),
  };

  const notes = [];
  M.P.forEach(p => {
    [p.ip, p.oop].forEach(r => { if (!r) return; const n = MOVE_NOTES[r.id]; if (n && n[0] === k) notes.push(`<b>${esc(r.ab)}</b> (${esc(p.s.ipCode)}) <span>${esc(n[1])}</span>`); });
  });
  return { text: TX[k], notes };
}

// demonstração de um papel: tática simples com o papel em destaque
function demoTacticForRole(r) {
  const g = r.groups[0];
  let F = { GK:'4-3-3 DM Wide', DC: r.wideCB ? '3-4-2-1' : '4-3-3 DM Wide', D_LR:'4-3-3 DM Wide', WB_LR:'3-4-2-1', DM:'4-3-3 DM Wide', MC:'4-3-3 DM Wide', M_LR:'4-4-2', AMC:'4-2-3-1 Wide', AM_LR:'4-3-3 DM Wide', ST:'4-3-3 DM Wide' }[g];
  const t = newTactic(F, F); t.name = 'Demo: ' + r.name; t.ment = 'Balanced';
  const fits = (form, code) => { const a = slotInfo(form, code); return a && validRole(r.id, a.group, r.phase, a); };
  if (!FORMATIONS[F].some(z => fits(F, z.code))) F = Object.keys(FORMATIONS).find(f => FORMATIONS[f].some(z => fits(f, z.code))) || F;
  const t2 = newTactic(F, F); t.slots = t2.slots; t.ipForm = F; t.oopForm = F;
  let idx = t.slots.findIndex(s => fits(F, s.ipCode) && slotInfo(F, s.ipCode).side !== 'R');
  if (idx < 0) idx = t.slots.findIndex(s => fits(F, s.ipCode));
  t.slots[idx][r.phase === 'IP' ? 'ip' : 'oop'] = r.id;
  return { t, idx };
}

// ============================================================
// Cenários realistas por papel (IP e OOP) para os clips da Bíblia.
// Cada papel tem 3 tempos reais: s0 (posição inicial) → s1 (movimento característico) → s2 (acção com/sem bola),
// com interacção directa da bola, reacção dos colegas e do adversário!
// ============================================================
function roleClipModel(roleId) {
  const r = ROLE_BY_ID[roleId], d = demoTacticForRole(r), t = d.t, fi = d.idx;
  const isIP = r.phase === 'IP';
  const baseM = simModel(t);
  const P = baseM.P.map(p => ({ ...p, pos: {}, pressing: false }));
  const me = P[fi];
  const sSide = me.sg || (me.a.x < 50 ? -1 : me.a.x > 50 ? 1 : -1); // -1 esq, +1 dir
  const byC = (re) => P.find(p => p.i !== fi && re.test(p.s.ipCode));
  const cbs = P.filter(p => p.i !== fi && p.a.group === 'DC');
  const gk = P.find(p => p.a.group === 'GK');
  const dm = byC(/^DM/) || byC(/^MC/);
  const cm = byC(/^MC/) || byC(/^DM/);
  const am = byC(/^AMC/) || byC(/^MC/);
  const st = byC(/^ST/) || byC(/^AMC/);
  const wideSame = P.find(p => p.i !== fi && ['AM_LR','M_LR','WB_LR'].includes(p.a.group) && (p.a.x < 50) === (sSide < 0));
  const wideOpp = P.find(p => p.i !== fi && ['AM_LR','M_LR','WB_LR'].includes(p.a.group) && (p.a.x < 50) !== (sSide < 0));
  const fbSame = P.find(p => p.i !== fi && ['D_LR','WB_LR'].includes(p.a.group) && (p.a.x < 50) === (sSide < 0));

  // Helpers para construir os 3 tempos (s0, s1, s2)
  const setBase = (ph, fromPh) => P.forEach(p => { p.pos[ph] = { ...baseM.P[p.i].pos[fromPh] }; });
  const opp = {};
  const setOpp = (ph, D, F) => {
    opp[ph] = OPP_SHAPE.map(([ox, oy], k) => {
      const tt = clamp((oy - 18) / 60, 0, 1);
      return k === 0 ? { x: 100 - ox, y: Math.min(97, D + 10) } : { x: 100 - ox, y: D + (F - D) * tt };
    });
  };
  const shift = (p, ph, dx, dy) => { if (!p) return; p.pos[ph] = { x: clamp(p.pos[ph].x + dx, 5, 95), y: clamp(p.pos[ph].y + dy, 3, 96) }; };
  const place = (p, ph, x, y) => { if (!p) return; p.pos[ph] = { x: clamp(x, 5, 95), y: clamp(y, 3, 96) }; };
  const ptOf = (p, ph, dx = 0, dy = 0, kind = 'pass') => p ? { x: clamp(p.pos[ph].x + dx, 4, 96), y: clamp(p.pos[ph].y + dy, 2, 99), kind } : { x: 50, y: 50, kind };

  const ball = {}, arrows = { s0: [], s1: [], s2: [] }, zones = { s0: [], s1: [], s2: [] }, lines = { s0: [], s1: [], s2: [] };
  const pressAt = { s0: false, s1: false, s2: false };
  let labels;

  if (isIP) {
    // ---- CENÁRIOS COM BOLA (IP): Posição inicial → Movimento do papel → Acção com bola
    labels = [L('1. Posição inicial', '1. Starting shape'), L('2. Movimento do papel', '2. Role movement'), L('3. Acção com bola', '3. Ball action')];
    const startPh = ['gk_ip','bpgk','nngk','cb_ip','bpcb','nncb','ifb','hb','iwb','dm_ip','dlp'].includes(roleId) ? 'build'
      : ['acb','pwb','cm_ip','mp','wcm','b2bp','ap','pw','wm_ip','fr','dlf','f9'].includes(roleId) ? 'prog' : 'prog';
    // s0: forma base real da formação (antes do movimento específico do papel)
    P.forEach(p => {
      const a = p.a;
      const y0 = startPh === 'build' ? clamp(a.y * 0.65 + 4, 4, 72) : clamp(a.y * 0.8 + 8, 5, 80);
      p.pos.s0 = { x: clamp(a.x, 6, 94), y: y0 };
    });
    setBase('s1', startPh === 'build' ? 'build' : 'prog');
    setBase('s2', startPh === 'build' ? 'prog' : 'final');
    // copiar s0 → s1 → s2 como base estável e aplicar o comportamento exacto de cada papel
    const keepTeamStable = (dy1, dy2) => P.forEach(p => {
      if (p.i === fi) return;
      const d1 = p.a.group === 'GK' ? dy1 * 0.2 : dy1, d2 = p.a.group === 'GK' ? dy2 * 0.2 : dy2;
      p.pos.s1 = { x: p.pos.s0.x, y: clamp(p.pos.s0.y + d1, 4, 92) };
      p.pos.s2 = { x: p.pos.s0.x, y: clamp(p.pos.s0.y + d2, 4, 94) };
    });
    setOpp('s0', 70, 38); setOpp('s1', 74, 44); setOpp('s2', 80, 52);

    const xW = 50 + sSide * 43, xH = 50 + sSide * 19, xIn = 50 + sSide * 10, xCh = 50 + sSide * 24, xCB = 50 + sSide * 28;
    switch (roleId) {
      case 'gk_ip':
        keepTeamStable(3, 6);
        place(me, 's0', 50, 5); place(me, 's1', 50, 7); place(me, 's2', 50, 9);
        ball.s0 = [ptOf(me,'s0'), ptOf(me,'s0', 0, 1)];
        ball.s1 = [ptOf(me,'s1'), ptOf(cbs[0] || fbSame, 's1')];
        ball.s2 = [ptOf(cbs[0] || fbSame, 's2'), ptOf(dm || cm, 's2')];
        break;
      case 'bpgk':
        keepTeamStable(3, 7);
        if (cbs[0]) { place(cbs[0],'s1',24,14); place(cbs[0],'s2',24,18); }
        if (cbs[1]) { place(cbs[1],'s1',76,14); place(cbs[1],'s2',76,18); }
        place(me, 's0', 50, 5); place(me, 's1', 50, 14); place(me, 's2', 50, 15);
        ball.s0 = [ptOf(cbs[0],'s0'), ptOf(me,'s0')];
        ball.s1 = [ptOf(me,'s0', 0, 0, 'dribble'), ptOf(me,'s1', 0, 0, 'dribble')];
        ball.s2 = [ptOf(me,'s2'), ptOf(dm || cm,'s2')];
        break;
      case 'nngk':
        keepTeamStable(5, 10);
        place(me, 's0', 50, 4); place(me, 's1', 50, 4); place(me, 's2', 50, 5);
        ball.s0 = [ptOf(cbs[0],'s0'), ptOf(me,'s0')];
        ball.s1 = [ptOf(me,'s0'), ptOf(me,'s1', 0, 1)];
        ball.s2 = [ptOf(me,'s2', 0, 0, 'loft'), ptOf(st,'s2', 0, 4, 'loft')];
        break;
      case 'cb_ip':
        keepTeamStable(3, 6);
        place(me, 's0', 50 + sSide * 15, 16); place(me, 's1', 50 + sSide * 22, 18); place(me, 's2', 50 + sSide * 22, 23);
        ball.s0 = [ptOf(gk,'s0'), ptOf(me,'s0')];
        ball.s1 = [ptOf(me,'s1'), ptOf(dm || fbSame,'s1')];
        ball.s2 = [ptOf(dm || fbSame,'s2'), ptOf(cm || wideSame,'s2')];
        break;
      case 'bpcb':
        keepTeamStable(4, 8);
        place(me, 's0', 50 + sSide * 16, 16); place(me, 's1', 50 + sSide * 18, 28); place(me, 's2', 50 + sSide * 18, 32);
        ball.s0 = [ptOf(gk,'s0'), ptOf(me,'s0')];
        ball.s1 = [ptOf(me,'s0', 0, 0, 'dribble'), ptOf(me,'s1', 0, 0, 'dribble')];
        ball.s2 = [ptOf(me,'s2', 0, 0, 'loft'), ptOf(wideOpp || st,'s2', 0, 4, 'loft')];
        break;
      case 'nncb':
        keepTeamStable(4, 8);
        place(me, 's0', 50 + sSide * 14, 15); place(me, 's1', 50 + sSide * 14, 15); place(me, 's2', 50 + sSide * 14, 17);
        ball.s0 = [ptOf(gk,'s0'), ptOf(me,'s0')];
        ball.s1 = [ptOf(me,'s0'), ptOf(me,'s1', 0, 1)];
        ball.s2 = [ptOf(me,'s2', 0, 0, 'loft'), ptOf(st,'s2', 0, 6, 'loft')];
        break;
      case 'acb': {
        keepTeamStable(4, 7);
        place(me, 's0', 50 + sSide * 15, 18); place(me, 's1', 50 + sSide * 8, 40); place(me, 's2', 50 + sSide * 8, 48);
        const f = cbs[0] || dm || gk;
        ball.s0 = [ptOf(gk,'s0'), ptOf(f,'s0')];
        ball.s1 = [ptOf(f,'s1'), ptOf(me,'s1')];
        ball.s2 = [ptOf(me,'s2'), ptOf(am || st,'s2')];
        break;
      }
      case 'ocb':
        keepTeamStable(5, 10);
        if (wideSame) { place(wideSame,'s1', xH, 54); place(wideSame,'s2', xH, 68); }
        place(me, 's0', xCB, 20); place(me, 's1', xW, 50); place(me, 's2', xW, 78);
        ball.s0 = [ptOf(cbs[0],'s0'), ptOf(me,'s0')];
        ball.s1 = [ptOf(me,'s0'), ptOf(wideSame || cm,'s1')];
        ball.s2 = [ptOf(wideSame || cm,'s2'), ptOf(me,'s2'), ptOf(st,'s2')];
        break;
      case 'fb_ip':
        keepTeamStable(4, 8);
        place(me, 's0', xW, 20); place(me, 's1', xW, 34); place(me, 's2', xW, 46);
        ball.s0 = [ptOf(cbs[0],'s0'), ptOf(me,'s0')];
        ball.s1 = [ptOf(me,'s1'), ptOf(wideSame || cm,'s1')];
        ball.s2 = [ptOf(wideSame || cm,'s2'), ptOf(me,'s2'), ptOf(dm || cm,'s2')];
        break;
      case 'ifb':
        keepTeamStable(5, 9);
        if (cbs[0]) { place(cbs[0],'s1',50,18); place(cbs[0],'s2',50,24); }
        if (cbs[1]) { place(cbs[1],'s1',50 - sSide * 26,18); place(cbs[1],'s2',50 - sSide * 26,24); }
        place(me, 's0', xW, 22); place(me, 's1', xCB, 18); place(me, 's2', xCB, 24);
        ball.s0 = [ptOf(gk,'s0'), ptOf(me,'s0')];
        ball.s1 = [ptOf(me,'s1'), ptOf(dm || cm,'s1')];
        ball.s2 = [ptOf(dm || cm,'s2'), ptOf(wideOpp || st,'s2')];
        break;
      case 'wb_ip':
        keepTeamStable(5, 10);
        if (wideSame) { place(wideSame,'s1', xH, 66); place(wideSame,'s2', xIn, 80); }
        place(me, 's0', xW, 26); place(me, 's1', xW, 56); place(me, 's2', xW, 82);
        ball.s0 = [ptOf(cbs[0],'s0'), ptOf(dm || cm,'s0')];
        ball.s1 = [ptOf(dm || cm,'s1'), ptOf(me,'s1')];
        ball.s2 = [ptOf(me,'s1', 0, 0, 'dribble'), ptOf(me,'s2', 0, 0, 'curve'), ptOf(st,'s2', 0, 2, 'curve')];
        break;
      case 'iwb':
        keepTeamStable(4, 8);
        if (wideSame) { place(wideSame,'s1', xW, 64); place(wideSame,'s2', xW, 76); }
        place(me, 's0', xW, 24); place(me, 's1', xH, 38); place(me, 's2', xH, 46);
        ball.s0 = [ptOf(gk,'s0'), ptOf(cbs[0],'s0')];
        ball.s1 = [ptOf(cbs[0],'s1'), ptOf(me,'s1')];
        ball.s2 = [ptOf(me,'s2'), ptOf(wideSame || cm,'s2')];
        break;
      case 'pwb':
        keepTeamStable(5, 9);
        if (wideSame) { place(wideSame,'s1', xW, 66); place(wideSame,'s2', xW, 80); }
        place(me, 's0', xW, 28); place(me, 's1', xH, 46); place(me, 's2', xH, 58);
        ball.s0 = [ptOf(gk,'s0'), ptOf(cbs[0],'s0')];
        ball.s1 = [ptOf(cbs[0],'s1'), ptOf(me,'s1')];
        ball.s2 = [ptOf(me,'s2'), ptOf(st || wideOpp,'s2', 0, 4)];
        break;
      case 'awb':
        keepTeamStable(6, 11);
        if (wideSame) { place(wideSame,'s1', xH, 68); place(wideSame,'s2', xIn, 84); }
        place(me, 's0', xW, 32); place(me, 's1', xW, 68); place(me, 's2', xW, 91);
        ball.s0 = [ptOf(cm || dm,'s0'), ptOf(wideSame || cm,'s0')];
        ball.s1 = [ptOf(wideSame || cm,'s1'), ptOf(me,'s1')];
        ball.s2 = [ptOf(me,'s1', 0, 0, 'dribble'), ptOf(me,'s2', 0, 0, 'pass'), ptOf(st,'s2', 0, 2)];
        break;
      case 'dm_ip':
        keepTeamStable(4, 8);
        place(me, 's0', 50, 30); place(me, 's1', 46, 33); place(me, 's2', 52, 37);
        ball.s0 = [ptOf(cbs[0],'s0'), ptOf(me,'s0')];
        ball.s1 = [ptOf(me,'s1'), ptOf(cm || fbSame,'s1')];
        ball.s2 = [ptOf(cm || fbSame,'s2'), ptOf(me,'s2'), ptOf(wideOpp || st,'s2')];
        break;
      case 'dlp':
        keepTeamStable(5, 10);
        place(me, 's0', 50, 31); place(me, 's1', 48, 28); place(me, 's2', 50, 36);
        ball.s0 = [ptOf(gk,'s0'), ptOf(cbs[0],'s0')];
        ball.s1 = [ptOf(cbs[0],'s1'), ptOf(me,'s1')];
        ball.s2 = [ptOf(me,'s2', 0, 0, 'loft'), ptOf(wideOpp || wideSame || st,'s2', 0, 6, 'loft')];
        break;
      case 'hb':
        keepTeamStable(5, 9);
        if (cbs[0]) { place(cbs[0],'s1',24,18); place(cbs[0],'s2',24,22); }
        if (cbs[1]) { place(cbs[1],'s1',76,18); place(cbs[1],'s2',76,22); }
        if (fbSame) { place(fbSame,'s1', xW, 46); place(fbSame,'s2', xW, 60); }
        place(me, 's0', 50, 34); place(me, 's1', 50, 17); place(me, 's2', 50, 21);
        ball.s0 = [ptOf(gk,'s0'), ptOf(cbs[0],'s0')];
        ball.s1 = [ptOf(cbs[0],'s1'), ptOf(me,'s1')];
        ball.s2 = [ptOf(me,'s2'), ptOf(fbSame || cm,'s2')];
        break;
      case 'b2b':
        keepTeamStable(5, 10);
        place(me, 's0', 50 + sSide * 12, 34); place(me, 's1', 50 + sSide * 10, 60); place(me, 's2', xIn, 86);
        ball.s0 = [ptOf(cbs[0],'s0'), ptOf(me,'s0')];
        ball.s1 = [ptOf(me,'s0'), ptOf(wideSame || cm,'s1')];
        ball.s2 = [ptOf(wideSame || cm,'s2'), ptOf(me,'s2', 0, 0, 'shot'), { x: 50, y: 98, kind: 'shot' }];
        break;
      case 'b2bp': {
        keepTeamStable(5, 10);
        place(me, 's0', 50 + sSide * 10, 34); place(me, 's1', xH, 58); place(me, 's2', xH, 74);
        const f = dm || fbSame || cbs[0];
        ball.s0 = [ptOf(gk,'s0'), ptOf(f,'s0')];
        ball.s1 = [ptOf(f,'s1'), ptOf(me,'s1')];
        ball.s2 = [ptOf(me,'s2'), ptOf(st || wideSame,'s2', 0, 4)];
        break;
      }
      case 'cm_ip': {
        keepTeamStable(4, 9);
        place(me, 's0', xH, 42); place(me, 's1', xH, 54); place(me, 's2', xH, 64);
        const w = wideSame || fbSame || dm;
        ball.s0 = [ptOf(dm || cbs[0],'s0'), ptOf(me,'s0')];
        ball.s1 = [ptOf(me,'s1'), ptOf(w,'s1')];
        ball.s2 = [ptOf(w,'s2'), ptOf(me,'s2'), ptOf(st,'s2')];
        break;
      }
      case 'mp': {
        keepTeamStable(4, 9);
        place(me, 's0', xH, 44); place(me, 's1', 50 + sSide * 12, 52); place(me, 's2', 50 + sSide * 10, 62);
        const f = dm || cbs[0];
        ball.s0 = [ptOf(gk,'s0'), ptOf(f,'s0')];
        ball.s1 = [ptOf(f,'s1'), ptOf(me,'s1')];
        ball.s2 = [ptOf(me,'s2'), ptOf(wideOpp || st,'s2', 0, 4)];
        break;
      }
      case 'wcm': {
        keepTeamStable(5, 10);
        if (wideSame) { place(wideSame,'s1', xIn, 74); place(wideSame,'s2', xIn, 86); }
        place(me, 's0', xH, 45); place(me, 's1', xW - sSide * 5, 62); place(me, 's2', xW - sSide * 3, 76);
        const f = dm || cm || cbs[0];
        ball.s0 = [ptOf(cbs[0] || gk,'s0'), ptOf(f,'s0')];
        ball.s1 = [ptOf(f,'s1'), ptOf(me,'s1')];
        ball.s2 = [ptOf(me,'s2', 0, 0, 'curve'), ptOf(st,'s2', 0, 2, 'curve')];
        break;
      }
      case 'chm': {
        keepTeamStable(5, 10);
        place(me, 's0', xH, 46); place(me, 's1', xCh, 68); place(me, 's2', xCh, 88);
        const f = cm || dm || cbs[0];
        ball.s0 = [ptOf(dm || cbs[0],'s0'), ptOf(f,'s0')];
        ball.s1 = [ptOf(f,'s0', 0, 0, 'dribble'), ptOf(f,'s1', 0, 0, 'dribble')];
        ball.s2 = [ptOf(f,'s2'), ptOf(me,'s2', 0, 0, 'shot'), { x: 50, y: 98, kind: 'shot' }];
        break;
      }
      case 'ap': {
        keepTeamStable(5, 9);
        place(me, 's0', 50, 54); place(me, 's1', xH, 66); place(me, 's2', xH, 74);
        const f = dm || cm || cbs[0];
        ball.s0 = [ptOf(cbs[0] || gk,'s0'), ptOf(f,'s0')];
        ball.s1 = [ptOf(f,'s1'), ptOf(me,'s1')];
        ball.s2 = [ptOf(me,'s2'), ptOf(st || wideSame,'s2', 0, 5)];
        break;
      }
      case 'am_ip': {
        keepTeamStable(5, 10);
        place(me, 's0', 50, 56); place(me, 's1', 50, 72); place(me, 's2', 50 + sSide * 6, 85);
        const f = cm || dm || cbs[0];
        ball.s0 = [ptOf(cbs[0] || gk,'s0'), ptOf(f,'s0')];
        ball.s1 = [ptOf(f,'s1'), ptOf(me,'s1')];
        ball.s2 = [ptOf(me,'s2', 0, 0, 'shot'), { x: 50, y: 98, kind: 'shot' }];
        break;
      }
      case 'wm_ip': {
        keepTeamStable(5, 9);
        place(me, 's0', xW, 46); place(me, 's1', xW, 60); place(me, 's2', xW, 74);
        const f = fbSame || cm || dm;
        ball.s0 = [ptOf(cbs[0] || gk,'s0'), ptOf(f,'s0')];
        ball.s1 = [ptOf(f,'s1'), ptOf(me,'s1')];
        ball.s2 = [ptOf(me,'s2', 0, 0, 'curve'), ptOf(st,'s2', 0, 2, 'curve')];
        break;
      }
      case 'w_ip':
        keepTeamStable(5, 10);
        place(me, 's0', xW, 54); place(me, 's1', xW, 76); place(me, 's2', xW, 93);
        ball.s0 = [ptOf(cm || fbSame,'s0'), ptOf(me,'s0')];
        ball.s1 = [ptOf(me,'s0', 0, 0, 'dribble'), ptOf(me,'s1', 0, 0, 'dribble')];
        ball.s2 = [ptOf(me,'s1', 0, 0, 'dribble'), ptOf(me,'s2', 0, 0, 'curve'), ptOf(st,'s2', 0, 2, 'curve')];
        break;
      case 'iw_ip':
        keepTeamStable(5, 10);
        if (fbSame) { place(fbSame,'s1', xW, 56); place(fbSame,'s2', xW, 76); }
        place(me, 's0', xW, 54); place(me, 's1', xH, 68); place(me, 's2', xH, 79);
        ball.s0 = [ptOf(fbSame || cm,'s0'), ptOf(me,'s0')];
        ball.s1 = [ptOf(me,'s0', 0, 0, 'dribble'), ptOf(me,'s1', 0, 0, 'dribble')];
        ball.s2 = [ptOf(me,'s2'), ptOf(st || wideOpp,'s2', 0, 4)];
        break;
      case 'pw': {
        keepTeamStable(5, 9);
        if (fbSame) { place(fbSame,'s1', xW, 54); place(fbSame,'s2', xW, 72); }
        place(me, 's0', xW, 52); place(me, 's1', xH, 62); place(me, 's2', 50 + sSide * 12, 71);
        const f = cm || fbSame || dm;
        ball.s0 = [ptOf(cbs[0] || gk,'s0'), ptOf(f,'s0')];
        ball.s1 = [ptOf(f,'s1'), ptOf(me,'s1')];
        ball.s2 = [ptOf(me,'s2', 0, 0, 'loft'), ptOf(wideOpp || st,'s2', 0, 5, 'loft')];
        break;
      }
      case 'if_ip': {
        keepTeamStable(5, 10);
        if (fbSame) { place(fbSame,'s1', xW, 58); place(fbSame,'s2', xW, 80); }
        place(me, 's0', xW, 56); place(me, 's1', xH, 74); place(me, 's2', xIn, 90);
        const f = cm || fbSame || dm;
        ball.s0 = [ptOf(cbs[0] || gk,'s0'), ptOf(f,'s0')];
        ball.s1 = [ptOf(f,'s1'), ptOf(me,'s1')];
        ball.s2 = [ptOf(me,'s1', 0, 0, 'dribble'), ptOf(me,'s2', 0, 0, 'shot'), { x: 50, y: 98, kind: 'shot' }];
        break;
      }
      case 'wf_ip': {
        keepTeamStable(5, 10);
        place(me, 's0', xW, 56); place(me, 's1', xW - sSide * 5, 74); place(me, 's2', xCh - sSide * 6, 91);
        const f = wideOpp || cm || dm;
        ball.s0 = [ptOf(cm || dm,'s0'), ptOf(f,'s0')];
        ball.s1 = [ptOf(f,'s0', 0, 0, 'dribble'), ptOf(f,'s1', 0, 0, 'dribble')];
        ball.s2 = [ptOf(f,'s2', 0, 0, 'curve'), ptOf(me,'s2', 0, 0, 'shot'), { x: 50, y: 98, kind: 'shot' }];
        break;
      }
      case 'fr': {
        keepTeamStable(5, 9);
        place(me, 's0', 48, 56); place(me, 's1', 36, 64); place(me, 's2', 56, 74);
        const f = cm || dm;
        ball.s0 = [ptOf(dm || cbs[0],'s0'), ptOf(me,'s0')];
        ball.s1 = [ptOf(me,'s1'), ptOf(f,'s1')];
        ball.s2 = [ptOf(f,'s2'), ptOf(me,'s2'), ptOf(st || wideOpp,'s2', 0, 4)];
        break;
      }
      case 'ss': {
        keepTeamStable(5, 9);
        if (st) { place(st,'s1',50,68); place(st,'s2',50,65); }
        place(me, 's0', 50, 56); place(me, 's1', 50 + sSide * 8, 76); place(me, 's2', 50 + sSide * 6, 91);
        const f = cm || dm;
        ball.s0 = [ptOf(dm || cbs[0],'s0'), ptOf(f,'s0')];
        ball.s1 = [ptOf(f,'s1'), ptOf(st,'s1')];
        ball.s2 = [ptOf(st,'s2'), ptOf(me,'s2', 0, 0, 'shot'), { x: 50, y: 98, kind: 'shot' }];
        break;
      }
      case 'cf_ip':
        keepTeamStable(5, 10);
        place(me, 's0', 50, 74); place(me, 's1', 50, 78); place(me, 's2', 50, 91);
        ball.s0 = [ptOf(cm || dm,'s0'), ptOf(me,'s0')];
        ball.s1 = [ptOf(me,'s1'), ptOf(wideSame || cm,'s1')];
        ball.s2 = [ptOf(wideSame || cm,'s2'), ptOf(me,'s2', 0, 0, 'shot'), { x: 50, y: 98, kind: 'shot' }];
        break;
      case 'chf': {
        keepTeamStable(5, 10);
        place(me, 's0', 50, 74); place(me, 's1', xCh, 82); place(me, 's2', xCh, 92);
        const f = cm || dm;
        ball.s0 = [ptOf(dm || cbs[0],'s0'), ptOf(f,'s0')];
        ball.s1 = [ptOf(f,'s0', 0, 0, 'dribble'), ptOf(f,'s1', 0, 0, 'dribble')];
        ball.s2 = [ptOf(f,'s2'), ptOf(me,'s2', 0, 0, 'shot'), { x: 50, y: 98, kind: 'shot' }];
        break;
      }
      case 'dlf':
        keepTeamStable(4, 9);
        if (wideSame) { place(wideSame,'s1', xIn, 76); place(wideSame,'s2', xIn, 90); }
        place(me, 's0', 50, 78); place(me, 's1', 50, 62); place(me, 's2', 50, 68);
        ball.s0 = [ptOf(dm || cm,'s0'), ptOf(cm,'s0')];
        ball.s1 = [ptOf(cm,'s1'), ptOf(me,'s1')];
        ball.s2 = [ptOf(me,'s2'), ptOf(wideSame || wideOpp || am,'s2', 0, 2)];
        break;
      case 'f9':
        keepTeamStable(4, 9);
        if (wideSame) { place(wideSame,'s1', xIn, 78); place(wideSame,'s2', xIn, 92); }
        if (wideOpp) { place(wideOpp,'s1', 100 - xIn, 78); place(wideOpp,'s2', 100 - xIn, 92); }
        opp.s1[2] = { x: 55, y: 66 }; opp.s2[2] = { x: 55, y: 63 };
        place(me, 's0', 50, 80); place(me, 's1', 50, 56); place(me, 's2', 50, 58);
        ball.s0 = [ptOf(dm || cm,'s0'), ptOf(cm,'s0')];
        ball.s1 = [ptOf(cm,'s1'), ptOf(me,'s1')];
        ball.s2 = [ptOf(me,'s2'), ptOf(wideSame || wideOpp,'s2', 0, 2)];
        break;
      case 'poa': {
        keepTeamStable(4, 8);
        place(me, 's0', 50, 82); place(me, 's1', 54, 88); place(me, 's2', 52, 95);
        const f = wideSame || cm;
        ball.s0 = [ptOf(cm || dm,'s0'), ptOf(f,'s0')];
        ball.s1 = [ptOf(f,'s0', 0, 0, 'dribble'), ptOf(f,'s1', 0, 0, 'dribble')];
        ball.s2 = [ptOf(f,'s2'), ptOf(me,'s2', 0, 0, 'shot'), { x: 50, y: 98, kind: 'shot' }];
        break;
      }
      case 'tf': {
        keepTeamStable(4, 9);
        if (cm) { place(cm,'s2', 50 + sSide * 8, 82); }
        place(me, 's0', 50, 80); place(me, 's1', 50, 86); place(me, 's2', 50, 88);
        const f = cbs[0] || dm || gk;
        ball.s0 = [ptOf(gk,'s0'), ptOf(f,'s0')];
        ball.s1 = [ptOf(f,'s1', 0, 0, 'loft'), ptOf(me,'s1', 0, 0, 'loft')];
        ball.s2 = [ptOf(me,'s2'), ptOf(cm || wideSame,'s2', 0, 0, 'shot'), { x: 50, y: 98, kind: 'shot' }];
        break;
      }
      default:
        keepTeamStable(4, 9);
        ball.s0 = [ptOf(dm || cbs[0],'s0'), ptOf(me,'s0')];
        ball.s1 = [ptOf(me,'s1'), ptOf(cm || wideSame,'s1')];
        ball.s2 = [ptOf(cm || wideSame,'s2'), ptOf(st || me,'s2')];
    }
    arrows.s1.push({ from: me.pos.s0, to: me.pos.s1, color: '#ffe27a', kind: 'run' });
    arrows.s2.push({ from: me.pos.s0, to: me.pos.s1, color: 'rgba(255,226,122,.45)', kind: 'run' });
    if (Math.hypot(me.pos.s2.x - me.pos.s1.x, me.pos.s2.y - me.pos.s1.y) > 3) {
      arrows.s2.push({ from: me.pos.s1, to: me.pos.s2, color: '#ffe27a', kind: 'run' });
    }
  } else {
    // ---- CENÁRIOS SEM BOLA (OOP): Adversário tem a bola e ataca → Reacção real do papel!
    const isOutlet = r.tags.includes('OUT');
    labels = isOutlet
      ? [L('1. Equipa defende', '1. Team defends'), L('2. Fica alto (outlet)', '2. Stays high (outlet)'), L('3. Arranque em transição', '3. Counter break')]
      : [L('1. Bloco inicial', '1. Block setup'), L('2. Adversário ataca', '2. Opponent attacks'), L('3. Reacção do papel', '3. Role action')];
    setBase('s0', 'oop'); setBase('s1', 'oop'); setBase('s2', 'oop');
    setOpp('s0', 84, 50); setOpp('s1', 78, 42); setOpp('s2', 74, 34);
    // colocar linhas defensivas subtis
    lines.s0.push({ y: baseM.dLine, color: 'rgba(125,211,252,.55)', label: '' });
    lines.s1.push({ y: baseM.dLine, color: 'rgba(125,211,252,.55)', label: '' });
    lines.s2.push({ y: baseM.dLine, color: 'rgba(125,211,252,.55)', label: '' });

    const xW = 50 + sSide * 43, xH = 50 + sSide * 20, xCh = 50 + sSide * 28;
    const oP = (ph, idx, x, y) => { opp[ph][idx] = { x: clamp(x, 5, 95), y: clamp(y, 4, 96) }; };
    const oPt = (ph, idx) => ({ ...opp[ph][idx], kind: 'opp' });
    const roleColor = r.tags.includes('HP') ? '#ff6b5a' : isOutlet ? '#ffe27a' : '#7dd3fc';

    switch (roleId) {
      case 'gk_oop':
        place(me,'s0',50,6); place(me,'s1',50,8); place(me,'s2',50,7);
        oP('s1', 7, 20, 32); oP('s2', 7, 16, 20);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',7)]; ball.s2 = [oPt('s2',7), { x: 46, y: 12, kind: 'opp' }];
        break;
      case 'lhk':
        place(me,'s0',50,4); place(me,'s1',50,3); place(me,'s2',50,3);
        oP('s1', 10, 52, 34); oP('s2', 10, 52, 22);
        ball.s0 = [oPt('s0',5), oPt('s0',8)]; ball.s1 = [oPt('s1',8), oPt('s1',10)]; ball.s2 = [oPt('s2',10), { x: 50, y: 4, kind: 'shot' }];
        zones.s2.push({ kind: 'circle', x: 50, y: 4, r: 7, color: 'rgba(125,211,252,.2)', stroke: 'rgba(125,211,252,.75)', label: L('Linha de golo', 'Goal line') });
        break;
      case 'swk':
        P.forEach(p => { if (p.b.group === 'DC' || p.b.group === 'D_LR') { shift(p,'s0',0,12); shift(p,'s1',0,12); shift(p,'s2',0,12); } });
        place(me,'s0',50,14); place(me,'s1',50,22); place(me,'s2',50,27);
        oP('s1', 8, 52, 52); oP('s2', 10, 50, 31);
        ball.s0 = [oPt('s0',5), oPt('s0',8)];
        ball.s1 = [oPt('s1',8), { x: 50, y: 28, kind: 'opp' }];
        ball.s2 = [{ x: 50, y: 27, kind: 'pass' }, ptOf(fbSame || cbs[0], 's2')];
        pressAt.s2 = true;
        break;
      case 'cb_oop':
        place(me,'s0',50 + sSide * 14,27); place(me,'s1',50 + sSide * 15,27); place(me,'s2',50 + sSide * 15,25);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',8)]; ball.s2 = [oPt('s2',8), oPt('s2',10)];
        break;
      case 'ccb':
        if (cbs[0]) { place(cbs[0],'s1',50 - sSide * 12, 34); place(cbs[0],'s2',50 - sSide * 12, 37); }
        place(me,'s0',50 + sSide * 14,27); place(me,'s1',50 + sSide * 6, 21); place(me,'s2',50, 18);
        oP('s1', 8, 50 - sSide * 12, 40); oP('s2', 10, 50, 26);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',8)]; ball.s2 = [oPt('s2',8), { x: 50, y: 20, kind: 'opp' }];
        break;
      case 'scb':
        if (cbs[0]) { place(cbs[0],'s1',50,22); place(cbs[0],'s2',50,20); }
        place(me,'s0',50 + sSide * 14,27); place(me,'s1',50 + sSide * 10,36); place(me,'s2',50 + sSide * 8,42);
        oP('s1', 8, 50 + sSide * 8, 45); oP('s2', 8, 50 + sSide * 8, 45);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',8)]; ball.s2 = [oPt('s2',8), ptOf(me,'s2')];
        pressAt.s2 = true;
        break;
      case 'cwcb':
        if (fbSame) { place(fbSame,'s0',xW,48); place(fbSame,'s1',xW,44); place(fbSame,'s2',xW,38); }
        place(me,'s0',xCh,26); place(me,'s1',xCh + sSide * 7,21); place(me,'s2',xCh + sSide * 11,18);
        oP('s1', 7, xW, 38); oP('s2', 7, xW, 24);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',7)]; ball.s2 = [oPt('s2',7), { x: xCh + sSide * 8, y: 20, kind: 'opp' }];
        break;
      case 'swcb':
        place(me,'s0',xCh,26); place(me,'s1',xH,36); place(me,'s2',xH,43);
        oP('s1', 8, xH, 46); oP('s2', 8, xH, 46);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',8)]; ball.s2 = [oPt('s2',8), ptOf(me,'s2')];
        pressAt.s2 = true;
        break;
      case 'fb_oop':
        place(me,'s0',xW - sSide * 4,27); place(me,'s1',xW - sSide * 4,27); place(me,'s2',xW - sSide * 2,29);
        oP('s1', 7, xW, 38); oP('s2', 7, xW, 34);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',7)]; ball.s2 = [oPt('s2',7), { x: xW - sSide * 12, y: 31, kind: 'opp' }];
        break;
      case 'hfb':
        place(me,'s0',xW - sSide * 6,27); place(me,'s1',xCh,23); place(me,'s2',xCh - sSide * 3,21);
        oP('s1', 10, 50 + sSide * 14, 34); oP('s2', 10, 50 + sSide * 14, 28);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',8)]; ball.s2 = [oPt('s2',8), oPt('s2',10)];
        break;
      case 'pfb':
        place(me,'s0',xW - sSide * 4,28); place(me,'s1',xW,40); place(me,'s2',xW,48);
        oP('s1', 7, xW, 52); oP('s2', 7, xW, 51);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',7)]; ball.s2 = [oPt('s2',7), ptOf(me,'s2')];
        pressAt.s2 = true;
        break;
      case 'wb_oop':
        place(me,'s0',xW,44); place(me,'s1',xW,34); place(me,'s2',xW,26);
        oP('s1', 7, xW, 44); oP('s2', 7, xW, 32);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',7)]; ball.s2 = [oPt('s2',7), { x: xW - sSide * 12, y: 26, kind: 'opp' }];
        break;
      case 'hwb':
        place(me,'s0',xW,30); place(me,'s1',xW - sSide * 4,24); place(me,'s2',xW - sSide * 6,21);
        oP('s1', 7, xW, 36); oP('s2', 7, xW, 29);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',7)]; ball.s2 = [oPt('s2',7), oPt('s2',10)];
        break;
      case 'prwb':
        place(me,'s0',xW,36); place(me,'s1',xW,52); place(me,'s2',xW,62);
        oP('s1', 4, xW, 66); oP('s2', 4, xW, 65);
        ball.s0 = [oPt('s0',0), oPt('s0',2)]; ball.s1 = [oPt('s1',2), oPt('s1',4)]; ball.s2 = [oPt('s2',4), ptOf(me,'s2')];
        pressAt.s2 = true;
        break;
      case 'dm_oop':
        place(me,'s0',50,37); place(me,'s1',46,36); place(me,'s2',54,36);
        oP('s1', 8, 45, 46); oP('s2', 8, 55, 45);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',6)]; ball.s2 = [oPt('s2',6), oPt('s2',8)];
        break;
      case 'ddm':
        if (cbs[0]) { place(cbs[0],'s1',30,26); place(cbs[0],'s2',28,25); }
        if (cbs[1]) { place(cbs[1],'s1',70,26); place(cbs[1],'s2',72,25); }
        place(me,'s0',50,38); place(me,'s1',50,29); place(me,'s2',50,25);
        oP('s1', 10, 44, 34); oP('s2', 10, 52, 32);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',7)]; ball.s2 = [oPt('s2',7), oPt('s2',10)];
        break;
      case 'sdm':
        place(me,'s0',50,36); place(me,'s1',48,35); place(me,'s2',50,35);
        oP('s0', 5, 46, 58); oP('s1', 5, 46, 52); oP('s1', 8, 50, 31); oP('s2', 5, 50, 52); oP('s2', 8, 50, 30);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',6)]; ball.s2 = [oPt('s2',6), oPt('s2',7)];
        zones.s2.push({ kind: 'trap', x1: 38, x2: 62, y1: 30, y2: 43, color: 'rgba(125,211,252,.18)', stroke: 'rgba(125,211,252,.72)', label: L('Linha fechada', 'Lane screened') });
        break;
      case 'wcdm':
        if (fbSame) { place(fbSame,'s0',xW,52); place(fbSame,'s1',xW,48); place(fbSame,'s2',xW,44); }
        place(me,'s0',50,36); place(me,'s1',xCh,33); place(me,'s2',xW - sSide * 6,30);
        oP('s1', 7, xW, 42); oP('s2', 7, xW, 34);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',7)]; ball.s2 = [oPt('s2',7), { x: xW - sSide * 10, y: 30, kind: 'opp' }];
        break;
      case 'pdm':
        place(me,'s0',50,37); place(me,'s1',48,47); place(me,'s2',48,54);
        oP('s1', 5, 48, 57); oP('s2', 5, 48, 57);
        ball.s0 = [oPt('s0',0), oPt('s0',2)]; ball.s1 = [oPt('s1',2), oPt('s1',5)]; ball.s2 = [oPt('s2',5), ptOf(me,'s2')];
        pressAt.s2 = true;
        break;
      case 'cm_oop':
        place(me,'s0',xH,45); place(me,'s1',xH + sSide * 3,44); place(me,'s2',xH + sSide * 4,42);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',6)]; ball.s2 = [oPt('s2',6), oPt('s2',8)];
        break;
      case 'pcm':
        place(me,'s0',xH,46); place(me,'s1',xH,58); place(me,'s2',xH,67);
        oP('s1', 2, xH, 71); oP('s2', 2, xH, 70);
        ball.s0 = [oPt('s0',0), oPt('s0',2)]; ball.s1 = [oPt('s0',2), oPt('s1',2)]; ball.s2 = [oPt('s2',2), ptOf(me,'s2')];
        pressAt.s2 = true;
        break;
      case 'scm':
        place(me,'s0',xH,44); place(me,'s1',50 + sSide * 10,43); place(me,'s2',50 + sSide * 10,42);
        oP('s0', 5, 50 + sSide * 10, 62); oP('s1', 5, 50 + sSide * 10, 58); oP('s2', 8, 50 + sSide * 10, 36);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',6)]; ball.s2 = [oPt('s2',6), oPt('s2',7)];
        zones.s2.push({ kind: 'trap', x1: 36, x2: 64, y1: 37, y2: 49, color: 'rgba(125,211,252,.18)', stroke: 'rgba(125,211,252,.72)', label: L('Zona protegida', 'Zone screened') });
        break;
      case 'wccm':
        if (fbSame) { place(fbSame,'s0',xW,54); place(fbSame,'s1',xW,50); place(fbSame,'s2',xW,46); }
        place(me,'s0',xH,45); place(me,'s1',xCh + sSide * 6,41); place(me,'s2',xW - sSide * 5,36);
        oP('s1', 7, xW, 46); oP('s2', 7, xW, 39);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',7)]; ball.s2 = [oPt('s2',7), { x: xW - sSide * 10, y: 34, kind: 'opp' }];
        break;
      case 'wm_oop':
        place(me,'s0',xW,52); place(me,'s1',xW - sSide * 4,44); place(me,'s2',xW - sSide * 5,40);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',7)]; ball.s2 = [oPt('s2',7), oPt('s2',8)];
        break;
      case 'twm':
      case 'tw':
        place(me,'s0',xW,58); place(me,'s1',xW,40); place(me,'s2',xW,25);
        oP('s0', 4, xW, 56); oP('s1', 4, xW, 42); oP('s2', 4, xW, 28);
        ball.s0 = [oPt('s0',5), oPt('s0',7)]; ball.s1 = [oPt('s1',7), oPt('s1',4)]; ball.s2 = [oPt('s1',4), oPt('s2',4)];
        break;
      case 'tam':
        place(me,'s0',50,58); place(me,'s1',50,46); place(me,'s2',50,38);
        oP('s0', 5, 50, 58); oP('s1', 5, 50, 48); oP('s2', 8, 50, 40);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',8)]; ball.s2 = [oPt('s2',8), ptOf(me,'s2')];
        break;
      case 'am_oop':
        place(me,'s0',50,56); place(me,'s1',48,52); place(me,'s2',48,50);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',6)]; ball.s2 = [oPt('s2',6), oPt('s2',8)];
        break;
      case 'w_oop':
        place(me,'s0',xW,56); place(me,'s1',xW - sSide * 5,48); place(me,'s2',xW - sSide * 6,44);
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',7)]; ball.s2 = [oPt('s2',7), oPt('s2',8)];
        break;
      case 'cf_oop':
        place(me,'s0',50,62); place(me,'s1',44,64); place(me,'s2',42,66);
        ball.s0 = [oPt('s0',2), oPt('s0',3)]; ball.s1 = [oPt('s1',3), oPt('s1',2)]; ball.s2 = [oPt('s2',2), oPt('s2',5)];
        break;
      case 'tcf':
        place(me,'s0',50,64); place(me,'s1',50,52); place(me,'s2',48,47);
        oP('s1', 5, 48, 50); oP('s2', 5, 48, 50);
        ball.s0 = [oPt('s0',0), oPt('s0',2)]; ball.s1 = [oPt('s1',2), oPt('s1',5)]; ball.s2 = [oPt('s2',5), ptOf(me,'s2')];
        pressAt.s2 = true;
        break;
      case 'wowm':
      case 'wow':
        P.forEach(p => { if (p.i !== fi && p.b.group !== 'GK') { shift(p,'s0',0,-6); shift(p,'s1',0,-8); shift(p,'s2',0,-4); } });
        place(me,'s0',xW,60); place(me,'s1',xW,65); place(me,'s2',xW,86);
        ball.s0 = [oPt('s0',5), oPt('s0',6)]; ball.s1 = [oPt('s1',6), ptOf(dm || cm,'s1')];
        ball.s2 = [ptOf(dm || cm,'s2', 0, 0, 'loft'), ptOf(me,'s2', 0, 2, 'loft')];
        break;
      case 'iow':
        P.forEach(p => { if (p.i !== fi && p.b.group !== 'GK') { shift(p,'s0',0,-6); shift(p,'s1',0,-8); shift(p,'s2',0,-4); } });
        place(me,'s0',xH,60); place(me,'s1',xH,66); place(me,'s2',xH - sSide * 4,88);
        ball.s0 = [oPt('s0',5), oPt('s0',6)]; ball.s1 = [oPt('s1',6), ptOf(dm || cm,'s1')];
        ball.s2 = [ptOf(dm || cm,'s2', 0, 0, 'loft'), ptOf(me,'s2', 0, 2, 'loft')];
        break;
      case 'coam':
      case 'cocf':
        P.forEach(p => { if (p.i !== fi && p.b.group !== 'GK') { shift(p,'s0',0,-6); shift(p,'s1',0,-8); shift(p,'s2',0,-4); } });
        place(me,'s0',50,62); place(me,'s1',50,66); place(me,'s2',50,86);
        ball.s0 = [oPt('s0',5), oPt('s0',8)]; ball.s1 = [oPt('s1',8), ptOf(dm || cbs[0],'s1')];
        ball.s2 = [ptOf(dm || cbs[0],'s2', 0, 0, 'loft'), ptOf(me,'s2', 0, 2, 'loft')];
        break;
      case 'soam':
      case 'socf':
        P.forEach(p => { if (p.i !== fi && p.b.group !== 'GK') { shift(p,'s0',0,-6); shift(p,'s1',0,-8); shift(p,'s2',0,-4); } });
        place(me,'s0',50,62); place(me,'s1',xCh,68); place(me,'s2',xCh + sSide * 4,88);
        ball.s0 = [oPt('s0',5), oPt('s0',8)]; ball.s1 = [oPt('s1',8), ptOf(dm || cbs[0],'s1')];
        ball.s2 = [ptOf(dm || cbs[0],'s2', 0, 0, 'loft'), ptOf(me,'s2', 0, 2, 'loft')];
        break;
      default:
        ball.s0 = [oPt('s0',2), oPt('s0',5)]; ball.s1 = [oPt('s1',5), oPt('s1',8)]; ball.s2 = [oPt('s2',8), oPt('s2',10)];
    }
    if (Math.hypot(me.pos.s1.x - me.pos.s0.x, me.pos.s1.y - me.pos.s0.y) > 2.5) {
      arrows.s1.push({ from: me.pos.s0, to: me.pos.s1, color: roleColor, kind: 'role' });
      arrows.s2.push({ from: me.pos.s0, to: me.pos.s1, color: roleColor, kind: 'role', dim: true });
    }
    if (Math.hypot(me.pos.s2.x - me.pos.s1.x, me.pos.s2.y - me.pos.s1.y) > 2.5) {
      arrows.s2.push({ from: me.pos.s1, to: me.pos.s2, color: roleColor, kind: 'role' });
    }
  }

  const M = { P, ball, opp, overlays: [], arrows, zones, lines, pressAt, dLine: baseM.dLine, front: baseM.front, bs: baseM.bs };
  return { M, phases: ['s0', 's1', 's2'], labels, focus: fi, slot: t.slots[fi], phase: r.phase, isRoleClip: true };
}

const TI_PHASE = { buildup:0, goal_kicks:0, gk_speed:0, gk_target:0, passing:1, tempo:1, reception:1, progress:1, support_runs:2, width:2, dribbling:1, patience:2, crossing:2, long_shots:2, creative:1,
  def_trans:3, loe:4, dline:4, dl_behaviour:4, trigger:4, press_trap:4, short_gk:4, tackling:4, cross_eng:4, att_trans:5, timewaste:2, set_pieces:2 };
const SIM_TI = new Set(['buildup','goal_kicks','gk_target','passing','tempo','reception','width','support_runs','dribbling','progress','patience','long_shots','crossing','creative','att_trans','loe','dline','dl_behaviour','trigger','press_trap','cross_eng','short_gk','def_trans']);

// ============================================================
// Cenários visuais para Conceitos e Contra cada sistema (Bíblia)
// ============================================================
const CONCEPT_CLIPS = new Set(['ip_oop','rest_def','width','box','press_coh','outlets','creativity','asymmetry']);

function oppCoordsFor(form, D, F) {
  const slots = FORMATIONS[form] || FORMATIONS['4-4-2'];
  return slots.map((s, k) => {
    const tt = clamp((s.y - 18) / 66, 0, 1);
    return k === 0 ? { x: 100 - s.x, y: Math.min(97, D + 10) } : { x: 100 - s.x, y: D + (F - D) * tt };
  });
}

function conceptClipModel(key) {
  const mk = (ipF, oopF, roles, ti = {}) => {
    const t = newTactic(ipF, oopF); t.ment = 'Positive'; t.ti = ti;
    Object.entries(roles || {}).forEach(([c, [ip, oop]]) => { const s = t.slots.find(x => x.ipCode === c); if (s) { if (ip) s.ip = ip; if (oop) s.oop = oop; } });
    return simModel(t);
  };
  const wrap = (M, phases, labels, opts = {}) => ({ M, phases, labels, focus: null, isConceptClip: true, phaseByStep: opts.phaseByStep || null });

  if (key === 'ip_oop') {
    const M = mk('4-3-3 DM Wide', '4-4-2', { DL:['wb_ip','fb_oop'], DR:['ifb','fb_oop'], DM:['dlp','cm_oop'], AML:['if_ip','wm_oop'], AMR:['w_ip','wm_oop'], ST:['f9','cf_oop'] });
    M.zones.final.push({ kind: 'lane', x1: 4, x2: 96, y1: 58, y2: 94, color: 'rgba(16,185,129,.12)', stroke: 'rgba(16,185,129,.55)', label: L('Com bola: 3-2-5', 'IP: 3-2-5 attack') });
    M.zones.oop.push({ kind: 'trap', x1: 12, x2: 88, y1: M.dLine - 2, y2: M.front + 2, color: 'rgba(96,165,250,.14)', stroke: 'rgba(96,165,250,.6)', label: L('Sem bola: 4-4-2', 'OOP: 4-4-2 block') });
    return wrap(M, ['final', 'loss', 'oop'], [L('1. Forma com bola (IP)', '1. In-possession shape'), L('2. Transição', '2. Transition'), L('3. Bloco sem bola (OOP)', '3. Out-of-possession block')], { phaseByStep: ['IP', 'OOP', 'OOP'] });
  }
  if (key === 'rest_def') {
    const M = mk('4-3-3 DM Wide', '4-1-4-1 DM Wide', { DL:['wb_ip','fb_oop'], DR:['ifb','hfb'], DM:['dm_ip','sdm'], MCL:['b2b','cm_oop'], AML:['if_ip','w_oop'], AMR:['w_ip','w_oop'] }, { def_trans: 2 });
    M.zones.final.push({ kind: 'trap', x1: 20, x2: 80, y1: 28, y2: 50, color: 'rgba(125,211,252,.18)', stroke: 'rgba(125,211,252,.78)', label: L('Rest-defence (3+1)', 'Rest-defence (3+1)') });
    M.zones.loss.push({ kind: 'trap', x1: 20, x2: 80, y1: 26, y2: 52, color: 'rgba(125,211,252,.18)', stroke: 'rgba(125,211,252,.78)', label: L('Escudo trava contra-ataque', 'Shield stops counter') });
    return wrap(M, ['prog', 'final', 'loss'], [L('1. Equipa sobe', '1. Team advances'), L('2. Rest-defence atrás da bola', '2. Rest-defence behind ball'), L('3. Perda travada atrás', '3. Counter stopped')], { phaseByStep: ['IP', 'IP', 'OOP'] });
  }
  if (key === 'width') {
    const M = mk('4-3-3 DM Wide', '4-1-4-1 DM Wide', { DL:['wb_ip','fb_oop'], DR:['iwb','fb_oop'], AML:['if_ip','w_oop'], AMR:['w_ip','w_oop'] }, { width: 4 });
    M.zones.final.push({ kind: 'lane', x1: 2, x2: 18, y1: 52, y2: 94, color: 'rgba(255,226,122,.16)', stroke: 'rgba(255,226,122,.7)', label: L('WB abre · IF entra', 'WB wide · IF inside') });
    M.zones.final.push({ kind: 'lane', x1: 82, x2: 98, y1: 52, y2: 94, color: 'rgba(255,226,122,.16)', stroke: 'rgba(255,226,122,.7)', label: L('W abre · IWB entra', 'W wide · IWB inside') });
    return wrap(M, ['build', 'prog', 'final'], [L('1. Início', '1. Build-up'), L('2. Esticar os flancos', '2. Stretch flanks'), L('3. 1 aberto por corredor', '3. One wide per flank')], { phaseByStep: ['IP', 'IP', 'IP'] });
  }
  if (key === 'box') {
    const M = mk('4-3-3 DM Wide', '4-1-4-1 DM Wide', { DL:['wb_ip','fb_oop'], MCL:['b2b','cm_oop'], AML:['if_ip','w_oop'], AMR:['wf_ip','w_oop'], ST:['f9','tcf'] });
    M.zones.final.push({ kind: 'trap', x1: 24, x2: 76, y1: 82, y2: 97, color: 'rgba(255,226,122,.16)', stroke: 'rgba(255,226,122,.75)', label: L('3+ na área (1.º / centro / 2.º)', '3+ in box (near / centre / far)') });
    return wrap(M, ['build', 'prog', 'final'], [L('1. F9 baixa e arrasta', '1. F9 drops & drags'), L('2. Corredores atacam espaço', '2. Runners attack space'), L('3. Área ocupada', '3. Box occupied')], { phaseByStep: ['IP', 'IP', 'IP'] });
  }
  if (key === 'press_coh') {
    const Mbad = mk('4-3-3 DM Wide', '4-4-2', {}, { loe: 2, dline: 0, trigger: 3 });
    const Mgood = mk('4-3-3 DM Wide', '4-4-2', { GK:['bpgk','swk'], DL:['wb_ip','pfb'] }, { loe: 2, dline: 4, trigger: 4, press_trap: 2 });
    const P = Mgood.P.map((p, i) => ({ ...p, pos: { s0: Mbad.P[i].pos.oop, s1: Mgood.P[i].pos.oop, s2: Mgood.P[i].pos.oop } }));
    const ob0 = Mbad.opp.oop, ob1 = Mgood.opp.oop;
    const trapPt = Mgood.ball.oop[Mgood.ball.oop.length - 1];
    const M = {
      P, opp: { s0: ob0, s1: ob1, s2: ob1 },
      ball: {
        s0: [{ ...ob0[2], kind: 'opp' }, { ...ob0[5], kind: 'opp' }],
        s1: [{ ...ob1[5], kind: 'opp' }, { ...ob1[7], kind: 'opp' }],
        s2: [{ ...ob1[7], kind: 'opp' }, trapPt],
      },
      overlays: [],
      lines: { s0: Mbad.lines.oop, s1: Mgood.lines.oop, s2: Mgood.lines.oop },
      zones: {
        s0: [{ kind: 'trap', x1: 10, x2: 90, y1: Mbad.dLine + 2, y2: Mbad.front - 4, color: 'rgba(255,107,90,.22)', stroke: 'rgba(255,107,90,.8)', label: L('Buraco entre linhas!', 'Huge gap between lines!') }],
        s1: Mgood.zones.oop,
        s2: Mgood.zones.oop,
      },
      arrows: { s0: [], s1: Mgood.P.filter(p => p.b.group === 'DC').map((p, i) => ({ from: Mbad.P[p.i].pos.oop, to: p.pos.oop, color: '#ffe27a', kind: 'run' })), s2: Mgood.arrows.oop },
      dLine: Mgood.dLine, front: Mgood.front, bs: Mgood.bs,
    };
    return wrap(M, ['s0', 's1', 's2'], [L('1. Desligado: buraco entre linhas', '1. Disconnected: huge gap'), L('2. Defesa sobe junta', '2. Line steps up together'), L('3. Bloco compacto + armadilha', '3. Compact press + trap')], { phaseByStep: ['OOP', 'OOP', 'OOP'] });
  }
  if (key === 'outlets') {
    const M = mk('4-3-3 DM Wide', '4-1-4-1 DM Wide', { AML:['if_ip','wow'], ST:['chf','socf'] }, { loe: 0, dline: 1, att_trans: 2 });
    M.zones.oop.push({ kind: 'trap', x1: 6, x2: 64, y1: 50, y2: 68, color: 'rgba(255,226,122,.16)', stroke: 'rgba(255,226,122,.72)', label: L('2 Outlets altos', '2 high outlets') });
    return wrap(M, ['oop', 'win'], [L('1. 9 defendem · 2 ficam altos', '1. 9 defend · 2 stay high'), L('2. Recuperação → contra-ataque', '2. Ball won → counter break')], { phaseByStep: ['OOP', 'IP'] });
  }
  if (key === 'creativity') {
    const M = mk('4-3-3 DM Wide', '4-1-4-1 DM Wide', { DM:['dlp','sdm'], MCL:['ap','cm_oop'], AML:['if_ip','w_oop'], AMR:['wf_ip','w_oop'], ST:['poa','cf_oop'] });
    const dlp = M.P.find(p => p.ip && p.ip.id === 'dlp'), ap = M.P.find(p => p.ip && p.ip.id === 'ap');
    if (dlp) M.zones.prog.push({ kind: 'circle', x: dlp.pos.prog.x, y: dlp.pos.prog.y, r: 10, color: 'rgba(255,226,122,.22)', stroke: 'rgba(255,226,122,.8)', label: L('Criador 1 (DLP)', 'Creator 1 (DLP)') });
    if (ap) M.zones.prog.push({ kind: 'circle', x: ap.pos.prog.x, y: ap.pos.prog.y, r: 10, color: 'rgba(255,226,122,.22)', stroke: 'rgba(255,226,122,.8)', label: L('Criador 2 (AP)', 'Creator 2 (AP)') });
    return wrap(M, ['build', 'prog', 'final'], [L('1. Construção segura', '1. Safe build-up'), L('2. 2 criadores pedem bola', '2. 2 creators find pockets'), L('3. Servem os corredores', '3. Feed the runners')], { phaseByStep: ['IP', 'IP', 'IP'] });
  }
  // asymmetry
  const M = mk('4-3-3 DM Wide', '4-1-4-1 DM Wide', { DL:['wb_ip','pfb'], DR:['ifb','hfb'], AML:['if_ip','w_oop'], AMR:['w_ip','tw'], DM:['dm_ip','sdm'] });
  M.zones.final.push({ kind: 'lane', x1: 2, x2: 24, y1: 42, y2: 92, color: 'rgba(255,226,122,.14)', stroke: 'rgba(255,226,122,.68)', label: L('Flanco ofensivo (WB+IF)', 'Attacking flank (WB+IF)') });
  M.zones.final.push({ kind: 'trap', x1: 68, x2: 95, y1: 26, y2: 52, color: 'rgba(125,211,252,.16)', stroke: 'rgba(125,211,252,.72)', label: L('Lado seguro (IFB)', 'Secure side (IFB)') });
  return wrap(M, ['build', 'prog', 'final'], [L('1. Saída assimétrica', '1. Asymmetric build-up'), L('2. Esq. sobe · Dir. protege', '2. Left attacks · Right locks'), L('3. Ataque + equilíbrio', '3. Attack + balance')], { phaseByStep: ['IP', 'IP', 'IP'] });
}

function oppClipModel(key) {
  const mk = (ipF, oopF, roles, ti = {}) => {
    const t = newTactic(ipF, oopF); t.ment = 'Positive'; t.ti = ti;
    Object.entries(roles || {}).forEach(([c, [ip, oop]]) => { const s = t.slots.find(x => x.ipCode === c); if (s) { if (ip) s.ip = ip; if (oop) s.oop = oop; } });
    return simModel(t);
  };
  const setOppForm = (M, form) => {
    const cfg = { build: [64, 30], prog: [74, 44], final: [86, 60], loss: [80, 46], oop: [Math.max(82, M.front + 14), M.dLine + 8], win: [70, M.dLine + 4] };
    Object.entries(cfg).forEach(([ph, [D, F]]) => { M.opp[ph] = oppCoordsFor(form, D, F); });
  };

  if (key === '442') {
    const M = mk('4-3-3 DM Wide', '4-1-4-1 DM Wide', { DM:['dlp','sdm'], MCL:['chm','cm_oop'], MCR:['mp','cm_oop'], AML:['if_ip','w_oop'] });
    setOppForm(M, '4-4-2');
    M.zones.prog.push({ kind: 'trap', x1: 26, x2: 74, y1: 35, y2: 64, color: 'rgba(255,226,122,.16)', stroke: 'rgba(255,226,122,.72)', label: L('3×2 no meio-campo', '3v2 central overload') });
    return { M, phases: ['build', 'prog', 'final'], labels: [L('1. Contra 4-4-2', '1. Vs 4-4-2'), L('2. 3×2 no meio + meios-espaços', '2. 3v2 midfield + half-spaces'), L('3. Ruptura nas costas', '3. Break in behind')], focus: null, isConceptClip: true, phaseByStep: ['IP','IP','IP'] };
  }
  if (key === '4231') {
    const M = mk('4-3-3 DM Wide', '4-4-2', { DCL:['bpcb','scb'], DCR:['cb_ip','ccb'], DM:['dm_ip','sdm'], ST:['cf_ip','tcf'] }, { loe: 1, dline: 2, trigger: 3 });
    setOppForm(M, '4-2-3-1 DM Wide');
    M.zones.oop.push({ kind: 'circle', x: 50, y: M.dLine + 16, r: 12, color: 'rgba(255,107,90,.2)', stroke: 'rgba(255,107,90,.8)', label: L('Anular o #10 (SCB+SDM)', 'Shut down #10 (SCB+SDM)') });
    return { M, phases: ['loss', 'oop'], labels: [L('1. Fechar o centro', '1. Close the middle'), L('2. Anular o AMC + pressionar pivots', '2. Trap #10 + press double pivot')], focus: null, isConceptClip: true, phaseByStep: ['OOP','OOP'] };
  }
  if (key === '433') {
    const M = mk('4-3-3 DM Wide', '4-1-4-1 DM Wide', { AML:['if_ip','wow'], ST:['chf','socf'] }, { loe: 1, dline: 1, att_trans: 2 });
    setOppForm(M, '4-3-3 DM Wide');
    M.zones.win.push({ kind: 'lane', x1: 4, x2: 28, y1: 58, y2: 92, color: 'rgba(255,226,122,.18)', stroke: 'rgba(255,226,122,.78)', label: L('Espaço nas costas do lateral', 'Space behind opponent FB') });
    return { M, phases: ['oop', 'win'], labels: [L('1. Laterais adversários sobem', '1. Opponent FBs push up'), L('2. Outlets atacam as costas', '2. Outlets sprint into space')], focus: null, isConceptClip: true, phaseByStep: ['OOP','IP'] };
  }
  if (key === '352') {
    const M = mk('4-3-3 DM Wide', '4-1-4-1 DM Wide', { DL:['wb_ip','pfb'], DR:['wb_ip','pfb'], AML:['w_ip','tw'], AMR:['w_ip','tw'] }, { loe: 2, dline: 3, trigger: 3, press_trap: 2 });
    setOppForm(M, '3-5-2');
    M.zones.final.push({ kind: 'lane', x1: 4, x2: 26, y1: 60, y2: 92, color: 'rgba(255,226,122,.16)', stroke: 'rgba(255,226,122,.75)', label: L('Prender os alas atrás', 'Pin wing-backs back') });
    return { M, phases: ['final', 'loss', 'oop'], labels: [L('1. Extremos prendem os alas', '1. Wingers pin wing-backs'), L('2. Transição na perda', '2. Transition on loss'), L('3. Armadilha fora + PFB', '3. Trap outside + PFB')], focus: null, isConceptClip: true, phaseByStep: ['IP','OOP','OOP'] };
  }
  if (key === '541') {
    const M = mk('4-3-3 DM Wide', '4-1-4-1 DM Wide', { DL:['wb_ip','fb_oop'], DR:['wb_ip','fb_oop'], AML:['if_ip','w_oop'], AMR:['wf_ip','w_oop'], MCL:['ap','cm_oop'] }, { width: 4, patience: 2, def_trans: 2, crossing: 0 });
    setOppForm(M, '5-4-1');
    M.zones.final.push({ kind: 'lane', x1: 4, x2: 96, y1: 68, y2: 94, color: 'rgba(255,226,122,.12)', stroke: 'rgba(255,226,122,.6)', label: L('Paciência + largura + 2.º poste', 'Patience + width + far post') });
    return { M, phases: ['prog', 'final', 'loss'], labels: [L('1. Esticar bloco de 5', '1. Stretch back 5'), L('2. Paciência e cruzamento baixo', '2. Patient low cutback'), L('3. Contra-pressão imediata', '3. Instant counter-press')], focus: null, isConceptClip: true, phaseByStep: ['IP','IP','OOP'] };
  }
  // diamond (4-4-2 Diamond)
  const M = mk('4-3-3 DM Wide', '4-1-4-1 DM Wide', { DL:['wb_ip','fb_oop'], DR:['wb_ip','fb_oop'], AML:['w_ip','w_oop'], AMR:['w_ip','w_oop'] }, { width: 4, support_runs: 2 });
  setOppForm(M, '4-4-2 Diamond');
  M.zones.prog.push({ kind: 'lane', x1: 2, x2: 22, y1: 36, y2: 90, color: 'rgba(255,226,122,.18)', stroke: 'rgba(255,226,122,.78)', label: L('Explorar largura!', 'Exploit wide flanks!') });
  M.zones.prog.push({ kind: 'lane', x1: 78, x2: 98, y1: 36, y2: 90, color: 'rgba(255,226,122,.18)', stroke: 'rgba(255,226,122,.78)', label: L('Losango estreito', 'Narrow diamond') });
  return { M, phases: ['build', 'prog', 'final'], labels: [L('1. Adversário estreito no meio', '1. Opponent narrow inside'), L('2. Abrir pelos corredores', '2. Exploit both flanks'), L('3. Sobreposição e cruzamento', '3. Overlap & cross')], focus: null, isConceptClip: true, phaseByStep: ['IP','IP','IP'] };
}
