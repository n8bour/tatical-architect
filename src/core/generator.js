// ============================================================
// Gerador de tática: formações, XI, papéis IP/OOP e as instruções
// de equipa (cada uma com razão), a partir do plantel.
//
// 1. Pré-selecção: todos os pares formação-com-bola × 3 formações-sem-bola
//    mais próximas, pontuados pelo melhor par de papéis de cada jogador.
// 2. Nos melhores pares: alterna (a) XI óptimo pelo algoritmo húngaro com
//    (b) pesquisa local do papel IP e OOP de cada posição, maximizando
//    adequação + regras de equilíbrio (auditTactic estrutural).
// 3. Instruções de equipa geradas comparando cada sector com a média do plantel.
// ============================================================

const EXPECT = {
  title: { n: 'Candidato ao título', ment: 'Positive', press: 1 },
  top: { n: 'Lugares de cima', ment: 'Positive', press: 0.5 },
  mid: { n: 'Meio da tabela', ment: 'Balanced', press: 0 },
  low: { n: 'Fuga à descida', ment: 'Cautious', press: -1 },
};

function pairCost(ip, oop) {
  return matchFormations(ip, oop).reduce((acc, m) => { const A = slotInfo(ip, m.ipCode), B = slotInfo(oop, m.oopCode); return acc + Math.hypot(A.x - B.x, A.y - B.y); }, 0);
}
// profundidade média dos jogadores de campo: a forma com bola não pode ser mais recuada do que a forma sem bola
const formDepth = (f) => { const sl = FORMATIONS[f].filter(x => x.code !== 'GK'); return sl.reduce((a, x) => a + x.y, 0) / sl.length; };
function formationPairs(perIP = 3) {
  const F = Object.keys(FORMATIONS), out = [];
  F.forEach(ip => F.filter(oop => formDepth(ip) >= formDepth(oop) - 0.5).map(oop => ({ ip, oop, c: pairCost(ip, oop) })).sort((a, b) => a.c - b.c).slice(0, perIP).forEach(x => out.push(x)));
  return out;
}

// cache de pontuação jogador × papel
function makeScorer(players) {
  const cache = new Map();
  const rs = (p, r) => { const k = p.id + '|' + r.id; let v = cache.get(k); if (v === undefined) { v = (roleScore(p, r) || { score: 0 }).score; cache.set(k, v); } return v; };
  const fit = (p, a, b, ri, ro) => (rs(p, ri) * 0.55 + rs(p, ro) * 0.45) * Math.min(posFactor(p, a.group, a.side), posFactor(p, b.group, b.side)) * footFactor(p, a, ri);
  return { rs, fit };
}

// Corre por passos: cada yield devolve o progresso; o valor final é o resultado.
function* generateTacticSteps(players, opts = {}) {
  const exp = EXPECT[opts.expect] || EXPECT.mid;
  const must = new Set(opts.must || []), youth = new Set(opts.youth || []), star = opts.star || null;
  if (players.length < 11) return null;
  yield { done: 0, total: 1, label: L('A avaliar todos os pares de formações', 'Evaluating every pair of formations') };
  const S = makeScorer(players);
  const bonus = (p) => (youth.has(p.id) ? 6 : 0);
  const weight = (p) => (p.id === star ? 1.6 : 1);

  // ---------- 1. pré-selecção rápida
  const bestPair = (p, a, b) => {
    let best = 0;
    const ipR = rolesFor(a.group, 'IP', a), opR = rolesFor(b.group, 'OOP', b);
    const bi = Math.max(...ipR.map(r => S.rs(p, r))), bo = Math.max(...opR.map(r => S.rs(p, r)));
    best = (bi * 0.55 + bo * 0.45) * Math.min(posFactor(p, a.group, a.side), posFactor(p, b.group, b.side));
    return best;
  };
  const quick = formationPairs(3).map(pr => {
    const pairs = matchFormations(pr.ip, pr.oop).map(m => ({ a: slotInfo(pr.ip, m.ipCode), b: slotInfo(pr.oop, m.oopCode), m }));
    const cost = pairs.map(x => players.map(p => -(bestPair(p, x.a, x.b) + bonus(p)) * weight(p) - (must.has(p.id) ? 1000 : 0)));
    while (cost[0].length < cost.length) cost.forEach(r => r.push(1000));
    const asg = hungarian(cost);
    const v = pairs.reduce((acc, x, i) => acc + (players[asg[i]] ? bestPair(players[asg[i]], x.a, x.b) : 0), 0) / 11;
    return { ...pr, v };
  }).sort((a, b) => b.v - a.v);
  // os melhores pares, sem repetir demasiado a mesma formação com bola
  const shortlist = [], perIP = {};
  for (const q of quick) { if ((perIP[q.ip] || 0) >= 2) continue; perIP[q.ip] = (perIP[q.ip] || 0) + 1; shortlist.push(q); if (shortlist.length >= (opts.depth || 10)) break; }

  // ---------- 2. optimização conjunta papéis × XI
  const objective = (t, pmap) => {
    let sum = 0, wsum = 0;
    t.slots.forEach(s => { const p = pmap[s.pid]; if (!p) return; const a = slotInfo(t.ipForm, s.ipCode), b = slotInfo(t.oopForm, s.oopCode);
      const w = weight(p); sum += (S.fit(p, a, b, ROLE_BY_ID[s.ip], ROLE_BY_ID[s.oop]) + bonus(p)) * w; wsum += w; });
    const fitAvg = wsum ? sum / wsum : 0;
    const au = auditTactic(t, []);
    const ipS = au.scores.IP, oopS = au.scores.OOP;
    return { v: fitAvg * 0.72 + ((ipS + oopS) / 2) * 0.28, fitAvg, au };
  };
  const assign = (t) => {
    const cost = t.slots.map(s => { const a = slotInfo(t.ipForm, s.ipCode), b = slotInfo(t.oopForm, s.oopCode), ri = ROLE_BY_ID[s.ip], ro = ROLE_BY_ID[s.oop];
      return players.map(p => -(S.fit(p, a, b, ri, ro) + bonus(p)) * weight(p) - (must.has(p.id) ? 1000 : 0)); });
    while (cost[0].length < cost.length) cost.forEach(r => r.push(1000));
    const asg = hungarian(cost);
    t.slots.forEach((s, i) => { s.pid = players[asg[i]] ? players[asg[i]].id : null; });
  };
  const pmap = Object.fromEntries(players.map(p => [p.id, p]));
  const optimise = (pr) => {
    const t = newTactic(pr.ip, pr.oop); t.ment = exp.ment; t.name = `${pr.ip} (gerada)`;
    // papéis iniciais: o melhor par para quem melhor serve cada posição
    assign(t);
    for (let pass = 0; pass < 2; pass++) {
      t.slots.forEach((s, i) => {
        const p = pmap[s.pid]; if (!p) return;
        const a = slotInfo(t.ipForm, s.ipCode), b = slotInfo(t.oopForm, s.oopCode);
        for (const phase of ['IP', 'OOP']) {
          const key = phase === 'IP' ? 'ip' : 'oop', grp = phase === 'IP' ? a : b;
          let best = s[key], bestV = -1;
          for (const r of rolesFor(grp.group, phase, grp)) { s[key] = r.id; const o = objective(t, pmap).v; if (o > bestV + 1e-9) { bestV = o; best = r.id; } }
          s[key] = best;
        }
      });
      assign(t);
    }
    const o = objective(t, pmap);
    return { t, pair: pr, fitAvg: Math.round(o.fitAvg), balance: Math.round((o.au.scores.IP + o.au.scores.OOP) / 2), v: o.v };
  };
  const results = [];
  for (let k = 0; k < shortlist.length; k++) {
    yield { done: k, total: shortlist.length, label: `A optimizar ${shortlist[k].ip} → ${shortlist[k].oop}` };
    results.push(optimise(shortlist[k]));
  }
  results.sort((a, b) => b.v - a.v);
  yield { done: shortlist.length, total: shortlist.length, label: L('A gerar instruções, bolas paradas e alternativas', 'Building instructions, set pieces and alternatives') };

  // ---------- 3. instruções de equipa com razão, para cada resultado
  results.forEach(r => { const g = generateTIs(r.t, players, exp); r.t.ti = g.ti; r.reasons = g.reasons; r.t.ment = g.ment; r.mentWhy = g.mentWhy;
    r.audit = auditTactic(r.t, players); r.total = Math.round(r.fitAvg * 0.6 + r.audit.overall * 0.4); });
  results.sort((a, b) => b.total - a.total);
  // alternativas com formação com bola diferente da recomendada
  const main = results[0], alts = [], seen = new Set([main.t.ipForm]);
  for (const r of results.slice(1)) { if (seen.has(r.t.ipForm)) continue; seen.add(r.t.ipForm); alts.push(r); if (alts.length === 3) break; }
  return { main, alts, considered: quick.length, optimised: shortlist.length, expect: exp };
}
// versão síncrona (testes, Node)
function generateTactic(players, opts = {}) { const it = generateTacticSteps(players, opts); let r = it.next(); while (!r.done) r = it.next(); return r.value; }
// versão assíncrona: devolve o controlo à janela entre passos e reporta o progresso
async function generateTacticAsync(players, opts = {}, onProgress = () => {}) {
  const it = generateTacticSteps(players, opts); let r = it.next();
  while (!r.done) { onProgress(r.value); await new Promise(res => setTimeout(res, 0)); r = it.next(); }
  return r.value;
}

// ---------- instruções de equipa a partir do perfil do plantel ----------
function generateTIs(t, players, exp) {
  const pm = Object.fromEntries(players.map(p => [p.id, p]));
  const xi = t.slots.map(s => ({ p: pm[s.pid], a: slotInfo(t.ipForm, s.ipCode), ip: ROLE_BY_ID[s.ip] })).filter(x => x.p);
  const grp = (gs) => xi.filter(x => gs.includes(x.a.group)).map(x => x.p);
  const top = [...players].sort((a, b) => (avgOf(b, Object.keys(ATTRS)) || 0) - (avgOf(a, Object.keys(ATTRS)) || 0)).slice(0, 18);
  const sq = (ks) => avgPlayers(top.filter(p => !(p.pos || []).includes('GK') || (p.pos || []).length > 1), ks) ?? 10;
  const v = (ps, ks) => avgPlayers(ps, ks);
  const f1 = (x) => x == null ? '—' : x.toFixed(1);
  const ti = {}, reasons = {};
  const set = (k, val, why) => { const d = TI_BY_KEY[k]; if (val !== d.def) ti[k] = val; reasons[k] = why; };
  const keep = (k, why) => { reasons[k] = why; };
  const out = xi.filter(x => x.a.group !== 'GK').map(x => x.p);
  const DC = grp(['DC']), FB = grp(['D_LR', 'WB_LR']), MID = grp(['DM', 'MC', 'AMC']), WIDE = grp(['M_LR', 'AM_LR']), ST = grp(['ST']), ATT = grp(['ST', 'AM_LR', 'AMC', 'M_LR']);
  const GK = grp(['GK']);
  const tech = v([...DC, ...MID, ...FB], ['pas', 'tec', 'fir', 'cmp']), phys = v(out, ['pac', 'acc', 'str']);
  const press = v(out, ['wor', 'sta', 'acc', 'agg']), sqPress = sq(['wor', 'sta', 'acc', 'agg']);
  const attPace = v(ATT, ['pac', 'acc']), sqPace = sq(['pac', 'acc']), cbPace = v(DC, ['pac', 'acc']);
  const stAir = v(ST, ['hea', 'jum', 'str']), cbAir = v(DC, ['hea', 'jum', 'str']);
  const buildQ = v([...GK, ...DC], ['pas', 'cmp', 'fir']);
  const flair = v(ATT, ['fla', 'dec', 'vis']), dri = v(ATT, ['dri', 'agi']), lon = (() => { const l = out.map(p => p.a.lon ?? 0).sort((a, b) => b - a).slice(0, 3); return l.reduce((a, b) => a + b, 0) / Math.max(1, l.length); })();
  const wideQ = v(WIDE.concat(FB), ['cro', 'dri', 'acc']), centreQ = v(MID, ['pas', 'vis', 'dec']);
  const defT = v([...DC, ...FB, ...grp(['DM'])], ['tck', 'agg']);
  const inside = xi.filter(x => ['M_LR', 'AM_LR'].includes(x.a.group) && x.ip && x.ip.tags.includes('IN')).length;
  const wideW = xi.filter(x => ['M_LR', 'AM_LR'].includes(x.a.group) && x.ip && x.ip.tags.includes('W')).length;
  const sideQ = (side) => v(xi.filter(x => x.a.side === side && x.a.group !== 'GK' && x.a.group !== 'DC').map(x => x.p), ['dri', 'cro', 'pas', 'acc', 'otb']);

  // mentalidade
  const ment = exp.ment, mentWhy = L(`${exp.n}: ${ment} dá iniciativa sem expor a equipa.`, `${T(exp.n)}: ${ment} takes the initiative without exposing the team.`) + (exp.press < 0 ? L(' Contra equipas mais fortes, prudência primeiro.', ' Against stronger sides, caution first.') : '');
  // ---- com bola
  if (tech != null && phys != null && tech - phys >= 1) set('passing', 1, L(`Os teus defesas e médios são mais técnicos (${f1(tech)}) do que físicos (${f1(phys)}): passe curto.`, `Your defenders and midfielders are more technical (${f1(tech)}) than physical (${f1(phys)}): short passing.`));
  else if (phys != null && tech != null && phys - tech >= 1.2 && stAir >= 13) set('passing', 3, L(`Equipa mais física (${f1(phys)}) do que técnica (${f1(tech)}) e avançados fortes no ar: jogo mais directo.`, `A more physical (${f1(phys)}) than technical (${f1(tech)}) side with strikers strong in the air: play more directly.`));
  else keep('passing', L(`Técnica (${f1(tech)}) e físico (${f1(phys)}) equilibrados.`, `Technique (${f1(tech)}) and physique (${f1(phys)}) are balanced.`));
  const midDec = v(MID, ['dec', 'tec']);
  if (midDec != null && midDec >= sq(['dec', 'tec']) + 1) set('tempo', 3, L(`O meio-campo decide e executa bem (${f1(midDec)}): podes acelerar.`, `Your midfield decides and executes well (${f1(midDec)}): you can speed up.`));
  else if (exp.press < 0) set('tempo', 1, L('Como underdog, ritmo mais baixo controla o jogo e poupa energia.', 'As the underdog, a lower tempo controls the game and saves energy.'));
  else keep('tempo', L('Ritmo normal para o nível do meio-campo.', 'Normal tempo for your midfield’s level.'));
  if (wideQ != null && centreQ != null && wideQ - centreQ >= 1.2) set('width', 3, L(`Corredores mais fortes (${f1(wideQ)}) do que o meio (${f1(centreQ)}): abre o jogo.`, `Your flanks are stronger (${f1(wideQ)}) than the centre (${f1(centreQ)}): stretch the game.`));
  else if (centreQ != null && wideQ != null && centreQ - wideQ >= 1.2) set('width', 1, L(`Meio-campo mais forte (${f1(centreQ)}) do que os corredores (${f1(wideQ)}): joga mais por dentro.`, `Your midfield is stronger (${f1(centreQ)}) than the flanks (${f1(wideQ)}): play more through the middle.`));
  else keep('width', L('Corredores e meio-campo ao mesmo nível.', 'Flanks and midfield at the same level.'));
  if (flair != null && flair >= 14) set('creative', 2, L(`Atacantes criativos (Talento/Decisões/Visão ${f1(flair)}): mais liberdade.`, `Creative attackers (Flair/Decisions/Vision ${f1(flair)}): more freedom.`));
  else if (flair != null && flair <= 10.5) set('creative', 0, L(`Talento/Decisões baixos na frente (${f1(flair)}): mais disciplina.`, `Low Flair/Decisions up front (${f1(flair)}): more discipline.`));
  else keep('creative', L('Criatividade média: equilíbrio.', 'Average creativity: balance.'));
  if ((attPace != null && attPace >= Math.max(14, sqPace + 1)) || exp.press < 0) set('att_trans', 2, attPace != null && attPace >= 14 ? L(`Atacantes rápidos (${f1(attPace)}): ataca a desorganização adversária.`, `Quick attackers (${f1(attPace)}): exploit the opponent’s disorganisation.`) : L('Como underdog, o contra-ataque é a melhor arma.', 'As the underdog, the counter-attack is your best weapon.'));
  else if (exp.press > 0 && tech != null && tech >= 13) set('att_trans', 0, L('Favorito e técnico: depois de recuperar, assume a posse.', 'Favourites and technical: after winning the ball, take control of possession.'));
  else keep('att_trans', L('Transição equilibrada.', 'A balanced transition.'));
  if (exp.press > 0) set('timewaste', 0, L('Favorito: queres ritmo, não queimar tempo.', 'Favourites: you want tempo, not to run down the clock.')); else keep('timewaste', L('Ajusta durante o jogo conforme o resultado.', 'Adjust during the match depending on the score.'));
  const air = xi.filter(x => x.a.group !== 'GK').map(x => SP.aerial(x.p)).sort((a, b) => b - a).slice(0, 4); const airTop = air.reduce((a, b) => a + b, 0) / Math.max(1, air.length);
  if (airTop >= 14) set('set_pieces', 2, L(`Quatro jogadores fortes no ar (média ${f1(airTop)}): procura cantos e livres.`, `Four players strong in the air (average ${f1(airTop)}): go for corners and free kicks.`));
  else if (airTop <= 11) set('set_pieces', 0, L(`Pouca presença aérea (${f1(airTop)}): mantém a bola em jogo.`, `Little aerial presence (${f1(airTop)}): keep the ball in play.`));
  else keep('set_pieces', L('Presença aérea média.', 'Average aerial presence.'));
  const attOtb = v(ATT, ['otb']);
  if (attPace != null && attPace >= 14 && attOtb >= 12.5) set('reception', 0, L(`Avançados rápidos e com bom movimento (${f1(attPace)} / ${f1(attOtb)}): passe para o espaço.`, `Quick forwards with good movement (${f1(attPace)} / ${f1(attOtb)}): pass into space.`));
  else if (v(ATT, ['fir', 'tec']) >= (attPace || 0) + 1.5) set('reception', 2, L('Avançados mais técnicos do que rápidos: passe ao pé.', 'Forwards more technical than quick: pass to feet.'));
  else keep('reception', L('Mistura de passe ao pé e para o espaço.', 'A mix of passes to feet and into space.'));
  if (buildQ != null && buildQ >= 13.5) set('buildup', 0, L(`GR e centrais bons com bola (${f1(buildQ)}): sai a jogar contra a pressão.`, `Keeper and centre-backs good on the ball (${f1(buildQ)}): play through the press.`));
  else if (buildQ != null && buildQ <= 10.5) set('buildup', 2, L(`GR e centrais fracos com bola (${f1(buildQ)}): salta a pressão.`, `Keeper and centre-backs weak on the ball (${f1(buildQ)}): bypass the press.`));
  else keep('buildup', L(`Qualidade de construção média (${f1(buildQ)}).`, `Average build-up quality (${f1(buildQ)}).`));
  if (buildQ != null && buildQ >= 13.5) set('goal_kicks', 0, L('Construção curta a partir do GR.', 'Short build-up from the keeper.'));
  else if (buildQ != null && buildQ <= 10.5 && stAir >= 13.5) set('goal_kicks', 2, L(`Pontapés longos para o teu avançado forte no ar (${f1(stAir)}).`, `Long kicks to your striker who is strong in the air (${f1(stAir)}).`));
  else keep('goal_kicks', L('Mistura curto e longo.', 'A mix of short and long.'));
  if (ti.att_trans === 2) set('gk_speed', 2, L('Contra-ataque: o GR repõe rápido.', 'Counter-attack: the keeper distributes quickly.')); else keep('gk_speed', L('Ritmo normal de reposição.', 'Normal distribution speed.'));
  if (ti.buildup === 2 && stAir >= 14) set('gk_target', 4, L('Bola longa para a referência aérea.', 'Long ball to the aerial target.'));
  else if (ti.buildup === 0 && v(FB, ['pas', 'fir']) >= 13) set('gk_target', 1, L('Laterais bons com bola: começa por eles.', 'Full-backs good on the ball: start through them.'));
  else keep('gk_target', L('Distribuição variada.', 'Varied distribution.'));
  if (inside >= 1 && wideW === 0) set('support_runs', 2, L('Os teus alas entram por dentro: os laterais sobrepõem por fora.', 'Your wide players come inside: the full-backs overlap outside.'));
  else if (wideW >= 2) set('support_runs', 0, L('Alas abertos: os laterais apoiam por dentro.', 'Wide wingers: the full-backs support inside.'));
  else keep('support_runs', L('Apoios mistos conforme o flanco.', 'Mixed support depending on the flank.'));
  if (dri != null && dri >= 14) set('dribbling', 2, L(`Atacantes com Drible/Agilidade altos (${f1(dri)}).`, `Attackers with high Dribbling/Agility (${f1(dri)}).`));
  else if (dri != null && dri <= 10) set('dribbling', 0, L(`Drible fraco na frente (${f1(dri)}): solta a bola.`, `Weak dribbling up front (${f1(dri)}): release the ball early.`));
  else keep('dribbling', L('Drible na média.', 'Average dribbling.'));
  const QL = sideQ('L'), QR = sideQ('R');
  if (QL != null && QR != null && Math.abs(QL - QR) >= 1.3) set('progress', QL > QR ? 0 : 3, L(`O teu flanco ${QL > QR ? 'esquerdo' : 'direito'} é mais forte (${f1(Math.max(QL, QR))} vs ${f1(Math.min(QL, QR))}).`, `Your ${QL > QR ? 'left' : 'right'} flank is stronger (${f1(Math.max(QL, QR))} vs ${f1(Math.min(QL, QR))}).`));
  else keep('progress', L('Flancos equilibrados.', 'Balanced flanks.'));
  if (exp.press > 0 && tech != null && tech >= 13) set('patience', 2, L('Favorito e técnico: trabalha a bola até à área.', 'Favourites and technical: work the ball into the box.'));
  else if (exp.press < 0) set('patience', 0, L('Como underdog, aproveita cada chegada.', 'As the underdog, make every attack count.'));
  else keep('patience', L('Paciência normal.', 'Normal patience.'));
  if (lon >= 14.5) set('long_shots', 2, L(`Há quem remate bem de longe (top 3: ${f1(lon)}).`, `You have good long-range shooters (top 3: ${f1(lon)}).`)); else if (lon <= 10) set('long_shots', 0, L(`Remates de longe fracos (${f1(lon)}).`, `Weak long-range shooting (${f1(lon)}).`)); else keep('long_shots', L('Remates de longe na média.', 'Average long-range shooting.'));
  if (stAir >= 15) set('crossing', 3, L(`Avançados dominantes no ar (${f1(stAir)}): cruzamentos altos.`, `Strikers dominant in the air (${f1(stAir)}): high crosses.`));
  else if (stAir != null && stAir <= 11 && attPace >= 13) set('crossing', 0, L(`Avançados baixos mas rápidos: cruzamentos rasteiros.`, `Short but quick strikers: low crosses.`));
  else keep('crossing', L('Cruzamentos variados.', 'Varied crosses.'));
  // ---- sem bola
  const pressRel = press != null ? press - sqPress : 0;
  if (press != null && (press >= 13.5 || pressRel >= 1) && exp.press >= 0) set('loe', 2, L(`Índice de Trabalho/Resistência/Aceleração do XI ${f1(press)}: aguenta pressão alta.`, `The XI’s Work Rate/Stamina/Acceleration ${f1(press)}: it can sustain a high press.`));
  else if ((press != null && press <= 10.5) || exp.press < 0) set('loe', 0, press != null && press <= 10.5 ? L(`Pouca capacidade de pressão (${f1(press)}): bloco baixo.`, `Little pressing capacity (${f1(press)}): low block.`) : L('Como underdog, bloco baixo e compacto.', 'As the underdog, a low and compact block.'));
  else keep('loe', L('Bloco médio.', 'Mid block.'));
  t.ti = ti; const cbRel = cbPace != null ? cbPace - sqPace : 0, loeV = tiVal(t, 'loe');
  if (cbPace != null && (cbRel <= -1.5 || cbPace <= 10.5)) set('dline', Math.max(1, loeV * 2 - 1), L(`Centrais mais lentos do que o resto da equipa (${f1(cbPace)} vs ${f1(sqPace)}): linha mais baixa.`, `Centre-backs slower than the rest of the team (${f1(cbPace)} vs ${f1(sqPace)}): a deeper line.`));
  else if (cbPace != null && cbPace >= 13.5 && loeV === 2) set('dline', 3, L(`Centrais rápidos (${f1(cbPace)}) e pressão alta: linha subida.`, `Quick centre-backs (${f1(cbPace)}) and a high press: a high line.`));
  else set('dline', [1, 2, 3][loeV], L('Linha alinhada com a altura da pressão.', 'The line matches the height of the press.'));
  if (press != null && press >= 14) set('trigger', 3, L(`Muita energia no XI (${f1(press)}).`, `Plenty of energy in the XI (${f1(press)}).`)); else if (press != null && press <= 10.5) set('trigger', 1, L(`Pouca energia (${f1(press)}): pressiona menos.`, `Little energy (${f1(press)}): press less.`)); else keep('trigger', L('Pressão normal.', 'Normal pressing.'));
  if (press != null && press >= 13 && exp.press >= 0) set('def_trans', 2, L('Recupera a bola logo após a perda.', 'Win the ball back straight after losing it.'));
  else if (exp.press < 0 || (press != null && press <= 10.5)) set('def_trans', 0, L('Reagrupa primeiro: protege a organização.', 'Regroup first: protect the shape.'));
  else keep('def_trans', L('Reacção equilibrada à perda.', 'A balanced reaction to losing the ball.'));
  if (defT != null && defT >= 14) set('tackling', 2, L(`Defesas fortes no desarme (${f1(defT)}).`, `Defenders strong in the tackle (${f1(defT)}).`)); else if (defT != null && defT <= 10.5) set('tackling', 0, L(`Desarme fraco (${f1(defT)}): evita faltas.`, `Weak tackling (${f1(defT)}): avoid fouls.`)); else keep('tackling', L('Desarme normal.', 'Normal tackling.'));
  const fbAgg = v(FB, ['acc', 'tck', 'agg']), mdDef = v(grp(['DM', 'MC']), ['tck', 'pos', 'ant']);
  if (fbAgg != null && mdDef != null && fbAgg - mdDef >= 1) set('press_trap', 2, L(`Laterais agressivos (${f1(fbAgg)}): força o jogo para fora.`, `Aggressive full-backs (${f1(fbAgg)}): force play wide.`));
  else if (mdDef != null && fbAgg != null && mdDef - fbAgg >= 1) set('press_trap', 0, L(`Meio-campo forte a defender (${f1(mdDef)}): força o jogo para dentro.`, `A midfield strong defensively (${f1(mdDef)}): force play inside.`));
  else keep('press_trap', L('Sem armadilha específica.', 'No specific trap.'));
  if (tiVal(t, 'loe') === 2) set('short_gk', 1, L('Pressão alta: não deixes o GR adversário sair curto.', 'High press: do not let the opposing keeper play short.')); else keep('short_gk', L('Deixa o adversário sair curto.', 'Let the opponent play out short.'));
  keep('dl_behaviour', L('Comportamento normal da linha.', 'Normal line behaviour.'));
  if (cbAir != null && cbAir >= 14) set('cross_eng', 0, L(`Centrais dominantes no ar (${f1(cbAir)}): deixa-os cruzar.`, `Centre-backs dominant in the air (${f1(cbAir)}): let them cross.`));
  else if (cbAir != null && cbAir <= 11.5) set('cross_eng', 2, L(`Centrais fracos no ar (${f1(cbAir)}): impede os cruzamentos.`, `Centre-backs weak in the air (${f1(cbAir)}): stop the crosses.`));
  else keep('cross_eng', L('Cruzamentos adversários sem tratamento especial.', 'No special treatment for opposition crosses.'));
  return { ti, reasons, ment, mentWhy };
}

// ---------- profundidade e reforços ----------
function depthChart(t, players, n = 3) {
  return t.slots.map(s => ({ code: s.ipCode, list: players.map(p => ({ p, sc: slotScore(p, t, s) })).sort((a, b) => b.sc - a.sc).slice(0, n) }));
}
function weakSpots(t, players) {
  const pm = Object.fromEntries(players.map(p => [p.id, p]));
  return t.slots.map(s => {
    const p = pm[s.pid], sc = p ? slotScore(p, t, s) : 0;
    const alt = players.filter(q => q.id !== s.pid).map(q => slotScore(q, t, s)).sort((a, b) => b - a)[0] || 0;
    const ri = ROLE_BY_ID[s.ip], lack = p ? ri.key.filter(k => p.a[k] != null).sort((a, b) => p.a[a] - p.a[b]).slice(0, 3) : [];
    return { code: s.ipCode, role: ri, p, sc, alt, lack, urgent: sc < 55 || alt < 45 };
  }).filter(x => x.urgent).sort((a, b) => a.sc - b.sc);
}
