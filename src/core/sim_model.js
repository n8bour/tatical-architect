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
