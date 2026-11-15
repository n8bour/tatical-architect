// ============================================================
// Sinergia ofensiva: quem serve quem, com os atributos reais.
// Não basta cada jogador estar no seu melhor papel — as oportunidades
// nascem de ligações: criador → finalizador, cruzador → alvo,
// avançado que baixa → quem ataca o espaço, extremo por dentro ↔ lateral por fora.
// Devolve 0–100 por componente, as ligações principais e os problemas com a correcção.
// ============================================================
const SYN_W = { chain: 0.30, cross: 0.15, front: 0.12, box: 0.15, flank: 0.16, load: 0.12 };
const SYN_PHASE_W = { build: 0.15, att: 0.40, def: 0.30, trans: 0.15 };

function synergyAttack(t, players) {
  const pm = Object.fromEntries((players || []).map(p => [p.id, p]));
  const a = (p, k) => (p && p.a && p.a[k] != null ? p.a[k] : 10);
  const q = (p, ks) => ks.reduce((s, k) => s + a(p, k), 0) / ks.length / 20;          // 0–1
  const Q = {
    create: (p) => q(p, ['pas', 'vis', 'tec', 'dec', 'fir']),
    run: (p) => q(p, ['otb', 'acc', 'pac', 'ant']),
    finish: (p) => q(p, ['fin', 'cmp', 'otb', 'ant']),
    air: (p) => q(p, ['hea', 'jum', 'str', 'bra']),
    cross: (p) => q(p, ['cro', 'tec', 'dec']),
    dribble: (p) => q(p, ['dri', 'agi', 'acc', 'fla']),
  };
  // jogadores com o papel com bola e a posição
  const X = t.slots.map(s => {
    const r = ROLE_BY_ID[s.ip], si = slotInfo(t.ipForm, s.ipCode), p = pm[s.pid];
    return r && si ? { s, r, si, p, tag: (k) => r.tags.includes(k), code: s.ipCode } : null;
  }).filter(Boolean);
  const out = X.filter(x => x.si.group !== 'GK');
  const lane = (x) => (x.si.x < 38 ? 'L' : x.si.x > 62 ? 'R' : 'C');
  const sameLane = (c, r) => lane(c) === 'C' || lane(r) === 'C' || lane(c) === lane(r);
  const label = (x) => `${x.r.ab} ${x.p ? surname(x.p.name) : x.code}`;
  const links = [], issues = [];
  const pct = (v) => Math.max(0, Math.min(100, Math.round(v * 100)));

  // ---- 1. criador → finalizador (criativos de meio-campo e ataque; construtores de trás não contam aqui)
  const creators = out.filter(x => x.tag('CR') && !['DC', 'GK'].includes(x.si.group));
  const receivers = out.filter(x => x.tag('RUN') || x.tag('BOX'));
  const chains = [];
  creators.forEach(c => {
    const opts = receivers.filter(r => r !== c && r.si.y >= c.si.y - 4 && sameLane(c, r))
      .map(r => ({ r, v: Math.sqrt(Q.create(c.p) * Math.max(Q.run(r.p), Q.finish(r.p))) }))
      .sort((m, n) => n.v - m.v);
    if (opts[0]) { chains.push(opts[0].v); links.push({ type: 'chain', from: c, to: opts[0].r, v: opts[0].v,
      why: L(`${label(c)} serve ${label(opts[0].r)}`, `${label(c)} feeds ${label(opts[0].r)}`) }); }
    else issues.push({ k: 'chain', t: L(`${label(c)} não tem ninguém à frente a atacar o espaço no seu corredor.`, `${label(c)} has nobody ahead attacking the space in his channel.`),
      fix: L('Ponha um Avançado Interior, Channel Midfielder ou Segundo Avançado à frente dele.', 'Put an Inside Forward, Channel Midfielder or Second Striker ahead of him.') });
  });
  chains.sort((m, n) => n - m);
  let chain = chains.length ? chains.slice(0, 3).reduce((s, v) => s + v, 0) / Math.min(3, chains.length) : 0;
  if (chains.length === 1) chain *= 0.8;
  // todos a servir o mesmo jogador é previsível: o adversário anula-o e a criação pára
  const chainLinks = links.filter(l => l.type === 'chain'), distinct = new Set(chainLinks.map(l => l.to)).size;
  if (chainLinks.length >= 2 && distinct === 1) {
    chain *= 0.8;
    issues.push({ k: 'chain', t: L(`Todos os criativos servem o mesmo jogador (${label(chainLinks[0].to)}): previsível e fácil de anular.`, `Every creator feeds the same player (${label(chainLinks[0].to)}): predictable and easy to shut down.`),
      fix: L('Dê um segundo alvo em movimento: Avançado Interior no outro flanco, Channel Midfielder ou Segundo Avançado.', 'Give them a second moving target: an Inside Forward on the other flank, a Channel Midfielder or a Second Striker.') });
  }
  if (!creators.length) issues.push({ k: 'chain', t: L('Ninguém cria no meio-campo ou no ataque.', 'Nobody creates in midfield or attack.'),
    fix: L('Um Construtor de Jogo (DLP, AP, MP) ou um Extremo Construtor dá a primeira peça da ligação.', 'A playmaker (DLP, AP, MP) or a Playmaking Winger provides the first link.') });
  if (!receivers.length) issues.push({ k: 'chain', t: L('Ninguém ataca o espaço nem a área.', 'Nobody attacks the space or the box.'),
    fix: L('Avançado Interior, Avançado de Corredor, Ponta-de-Lança Fixo ou Médio Área-a-Área.', 'Inside Forward, Channel Forward, Poacher or Box-to-Box Midfielder.') });

  // ---- 2. cruzamentos → alvos
  const wides = out.filter(x => x.tag('W') && x.si.side !== 'C' && x.si.group !== 'DC');
  const targets = out.filter(x => x.tag('BOX') && ['ST', 'AMC', 'MC'].includes(x.si.group));
  const bestTarget = targets.map(x => ({ x, v: Math.max(Q.air(x.p), Q.finish(x.p) * 0.9) })).sort((m, n) => n.v - m.v)[0];
  let cross = 0;
  if (wides.length && bestTarget) {
    const bestW = wides.map(x => ({ x, v: Q.cross(x.p) })).sort((m, n) => n.v - m.v)[0];
    cross = Math.sqrt(bestW.v * bestTarget.v) * Math.min(1, 0.55 + 0.25 * targets.length);
    links.push({ type: 'cross', from: bestW.x, to: bestTarget.x, v: cross, why: L(`${label(bestW.x)} cruza para ${label(bestTarget.x)}`, `${label(bestW.x)} crosses for ${label(bestTarget.x)}`) });
    if (Q.cross(bestW.x.p) < 0.55) issues.push({ k: 'cross', t: L(`Quem dá largura cruza mal (${label(bestW.x)}).`, `The width providers cross poorly (${label(bestW.x)}).`), fix: L('Low Crosses, ou um ala/extremo com melhores Cruzamentos.', 'Low Crosses, or a wide player with better Crossing.') });
  } else if (wides.length && !bestTarget) {
    issues.push({ k: 'cross', t: L('Há largura mas ninguém ataca a área para os cruzamentos.', 'There is width but nobody attacks the box for crosses.'), fix: L('Ponta-de-Lança, Avançado de Referência ou um médio que chegue à área.', 'A Centre Forward, Target Forward or a midfielder who arrives in the box.') });
  }

  // ---- 3. dupla / referência da frente
  const sts = out.filter(x => x.si.group === 'ST');
  let front = 0.6;
  if (sts.length >= 2) {
    const drop = sts.filter(x => x.tag('DROP') || x.tag('AIR')), run = sts.filter(x => x.tag('RUN') && !x.tag('DROP'));
    const complete = sts.filter(x => x.tag('LINK') && x.tag('BOX') && !x.tag('DROP') && !x.tag('AIR'));
    if (!drop.length && complete.length && run.length && complete[0] !== run[0]) { front = 0.82; links.push({ type: 'front', from: complete[0], to: run[0], v: front, why: L(`${label(complete[0])} segura e ${label(run[0])} ataca a profundidade`, `${label(complete[0])} holds it up and ${label(run[0])} runs in behind`) }); }
    else if (drop.length && run.length && drop[0] !== run[0]) { front = 0.9 + 0.1 * Math.min(Q.run(run[0].p), 1); links.push({ type: 'front', from: drop[0], to: run[0], v: front, why: L(`${label(drop[0])} baixa e abre espaço para ${label(run[0])}`, `${label(drop[0])} drops and opens space for ${label(run[0])}`) }); }
    else { front = 0.45; issues.push({ k: 'front', t: L('Os dois avançados têm o mesmo perfil e ocupam o mesmo espaço.', 'Both strikers have the same profile and occupy the same space.'), fix: L('Um que baixe (Avançado Recuado, Avançado de Referência) e outro que ataque o espaço (Avançado de Corredor, Ponta-de-Lança Fixo).', 'One who drops (Deep-Lying Forward, Target Forward) and one who runs in behind (Channel Forward, Poacher).') }); }
  } else if (sts.length === 1) {
    const st = sts[0];
    if (st.tag('DROP')) {
      const beyond = out.filter(x => x !== st && (x.tag('RUN') || (x.tag('BOX') && x.si.group !== 'ST')) && x.si.y >= 55);
      front = Math.min(1, 0.4 + 0.25 * beyond.length);
      if (beyond.length < 2) issues.push({ k: 'front', t: L(`${label(st)} baixa, mas quase ninguém ataca o espaço que ele abre.`, `${label(st)} drops, but hardly anyone attacks the space he opens.`), fix: L('Dois corredores por trás dele: Avançado Interior, Channel Midfielder ou Segundo Avançado.', 'Two runners behind him: Inside Forward, Channel Midfielder or Second Striker.') });
      else links.push({ type: 'front', from: st, to: beyond[0], v: front, why: L(`${label(st)} baixa e ${label(beyond[0])} ataca o espaço`, `${label(st)} drops and ${label(beyond[0])} attacks the space`) });
    } else front = 0.55 + 0.35 * Math.max(Q.finish(st.p), Q.run(st.p));
  } else front = 0.35;

  // ---- 4. presença na área (3–4 é o ideal)
  const boxN = out.filter(x => x.tag('BOX') || (x.tag('IN') && x.tag('RUN'))).length;
  const box = { 0: 0, 1: 0.15, 2: 0.45, 3: 0.85, 4: 1, 5: 0.85 }[boxN] ?? 0.6;
  if (boxN <= 2) issues.push({ k: 'box', t: L(`Só ${boxN} jogador(es) chegam à área: poucas finalizações.`, `Only ${boxN} player(s) arrive in the box: few shots.`), fix: L('Troque um papel de apoio por Avançado Interior, Channel Midfielder ou Médio Área-a-Área.', 'Swap a support role for an Inside Forward, Channel Midfielder or Box-to-Box Midfielder.') });

  // ---- 5. combinações de flanco
  let flank = 0, fl = 0;
  ['L', 'R'].forEach(side => {
    const att = out.filter(x => x.si.side === side && ['AM_LR', 'M_LR'].includes(x.si.group));
    const back = out.filter(x => x.si.side === side && ['D_LR', 'WB_LR'].includes(x.si.group));
    if (!att.length && !back.length) return;
    fl++;
    const aIn = att.some(x => x.tag('IN')), aW = att.some(x => x.tag('W')), bW = back.some(x => x.tag('W') || x.tag('ADV')), bIn = back.some(x => x.tag('IN') || x.tag('RD'));
    const nm = side === 'L' ? L('esquerdo', 'left') : L('direito', 'right');
    if (aIn && bW) { flank += 1; const f = back.find(x => x.tag('W') || x.tag('ADV')), g = att.find(x => x.tag('IN')); links.push({ type: 'flank', from: f, to: g, v: 1, why: L(`Flanco ${nm}: ${label(f)} por fora, ${label(g)} por dentro`, `${side === 'L' ? 'Left' : 'Right'} flank: ${label(f)} outside, ${label(g)} inside`) }); }
    else if (aW && bIn) flank += 0.9;
    else if (aW && bW) { flank += 0.45; issues.push({ k: 'flank', t: L(`Flanco ${nm}: dois jogadores abertos no mesmo corredor.`, `${side === 'L' ? 'Left' : 'Right'} flank: two wide players in the same channel.`), fix: L('Um deles por dentro (Extremo de Apoio Interior / Ala Invertido) ou Supporting Runs em Underlap.', 'One of them inside (Inside Winger / Inside Wing-Back) or Supporting Runs set to Underlap.') }); }
    else if (aIn && bIn) { flank += 0.35; issues.push({ k: 'flank', t: L(`Flanco ${nm}: ninguém dá largura.`, `${side === 'L' ? 'Left' : 'Right'} flank: nobody provides width.`), fix: L('Lateral/ala por fora (Wing-Back, Ala Projetado) ou um Extremo aberto.', 'A full-back/wing-back outside (Wing-Back, Advanced Wing-Back) or a wide Winger.') }); }
    else flank += 0.7;
  });
  flank = fl ? flank / fl : 0.6;

  // ---- 6. carga criativa
  const nCr = creators.length;
  const load = { 0: 0, 1: 0.65, 2: 1, 3: 0.9 }[nCr] ?? 0.5;
  if (nCr === 1) issues.push({ k: 'load', t: L(`Só ${label(creators[0])} cria: se for anulado, a equipa deixa de criar.`, `Only ${label(creators[0])} creates: if he is shut down, the team stops creating.`), fix: L('Um segundo criativo noutra zona (Construtor de Jogo Recuado atrás, ou Extremo Construtor numa ala).', 'A second creator in another zone (a Deep-Lying Playmaker behind, or a Playmaking Winger out wide).') });
  if (nCr > 3) issues.push({ k: 'load', t: L(`${nCr} criativos pedem todos a bola; falta quem a receba em movimento.`, `${nCr} creators all want the ball; nobody receives it on the move.`), fix: L('Troque um criativo por um papel de movimento (corredor/finalizador).', 'Swap one creator for a movement role (runner/finisher).') });

  // ameaça individual (bónus pequeno): o melhor driblador da frente
  const drib = Math.max(0, ...out.filter(x => ['AM_LR', 'AMC', 'ST', 'M_LR'].includes(x.si.group)).map(x => Q.dribble(x.p)));
  const parts = { chain: pct(chain), cross: pct(cross), front: pct(front), box: pct(box), flank: pct(flank), load: pct(load) };
  let score = Object.entries(SYN_W).reduce((s, [k, w]) => s + parts[k] * w, 0);
  score = Math.round(Math.min(100, score * 0.95 + drib * 100 * 0.05));
  links.sort((m, n) => n.v - m.v);
  links.forEach(l => l.phase = 'att'); issues.forEach(i => i.phase = 'att');
  return { score, parts, links: links.slice(0, 8), issues, creators: nCr, boxN };
}
const SYN_NAMES_ATT = () => ({ chain: L('Criador → finalizador', 'Creator → finisher'), cross: L('Cruzamentos → alvos', 'Crosses → targets'), front: L('Dupla / referência da frente', 'Front line partnership'),
  box: L('Presença na área', 'Box presence'), flank: L('Combinações de flanco', 'Flank combinations'), load: L('Carga criativa', 'Creative load') });

// ============================================================
// Construção, defesa e transições — as sinergias das outras fases
// ============================================================
function synergyReport(t, players) {
  const pm = Object.fromEntries((players || []).map(p => [p.id, p]));
  const a = (p, k) => (p && p.a && p.a[k] != null ? p.a[k] : 10);
  const q = (p, ks) => ks.reduce((s, k) => s + a(p, k), 0) / ks.length / 20;
  const pct = (v) => Math.max(0, Math.min(100, Math.round(v * 100)));
  const IPX = t.slots.map(s => { const r = ROLE_BY_ID[s.ip], si = slotInfo(t.ipForm, s.ipCode); return r && si ? { s, r, si, p: pm[s.pid], code: s.ipCode, tag: (k) => r.tags.includes(k) } : null; }).filter(Boolean);
  const OOX = t.slots.map(s => { const r = ROLE_BY_ID[s.oop], si = slotInfo(t.oopForm, s.oopCode); return r && si ? { s, r, si, p: pm[s.pid], code: s.oopCode, tag: (k) => r.tags.includes(k) } : null; }).filter(Boolean);
  const label = (x) => `${x.r.ab} ${x.p ? surname(x.p.name) : x.code}`;
  const sideName = (side) => side === 'L' ? L('esquerdo', 'left') : L('direito', 'right');
  const ti = (k) => tiVal(t, k);

  // ---------------- ATAQUE (modelo anterior)
  const att = synergyAttack(t, players);

  // ---------------- CONSTRUÇÃO
  const bIss = [], bLinks = [];
  const back = IPX.filter(x => ['GK', 'DC'].includes(x.si.group) || x.tag('TOCB'));
  const recv = IPX.filter(x => ['DM', 'MC'].includes(x.si.group) || x.tag('TODM') || x.tag('IN') && ['D_LR', 'WB_LR'].includes(x.si.group));
  const passQ = (p) => q(p, ['pas', 'fir', 'cmp', 'dec']);
  const bestBack = back.map(x => ({ x, v: passQ(x.p) * (x.tag('CR') ? 1.05 : 1) })).sort((m, n) => n.v - m.v)[0];
  const bestRecv = recv.map(x => ({ x, v: q(x.p, ['fir', 'pas', 'cmp', 'ant', 'dec']) })).sort((m, n) => n.v - m.v)[0];
  let bShort = bestBack && bestRecv ? Math.sqrt(bestBack.v * bestRecv.v) : 0.3;
  if (bestBack && bestRecv) bLinks.push({ type: 'build', from: bestBack.x, to: bestRecv.x, v: bShort, phase: 'build', why: L(`${label(bestBack.x)} sai a jogar para ${label(bestRecv.x)}`, `${label(bestBack.x)} plays out to ${label(bestRecv.x)}`) });
  const supportN = IPX.filter(x => x.tag('TODM') || x.tag('TOCB') || x.tag('DROP') && x.si.group === 'DM' || x.si.group === 'DM').length;
  const bSupport = { 0: 0.3, 1: 0.75, 2: 1, 3: 1 }[supportN] ?? 0.85;
  if (supportN === 0) bIss.push({ k: 'build', phase: 'build', t: L('Ninguém se oferece entre linhas na saída de bola: os centrais ficam sem linha de passe.', 'Nobody offers himself between the lines in the build-up: the centre-backs have no passing option.'), fix: L('Um Médio Defensivo, Construtor de Jogo Recuado ou um lateral que feche por dentro (Lateral Invertido / Ala Invertido).', 'A Defensive Midfielder, Deep-Lying Playmaker or a full-back who tucks inside (Inside Full-Back / Inside Wing-Back).') });
  const firstLine = IPX.filter(x => ['GK', 'DC', 'DM'].includes(x.si.group));
  const resist = firstLine.length ? firstLine.reduce((s, x) => s + q(x.p, ['fir', 'cmp', 'pas', 'dec']), 0) / firstLine.length : 0.5;
  const through = ti('buildup') === 0, bypass = ti('buildup') === 2 || ti('goal_kicks') === 2;
  let bLong = 0.7;
  if (bypass) {
    const tgt = IPX.filter(x => x.si.group === 'ST').map(x => ({ x, v: q(x.p, ['hea', 'jum', 'str', 'bra']) * (x.tag('AIR') ? 1.08 : 1) })).sort((m, n) => n.v - m.v)[0];
    bLong = tgt ? Math.min(1, tgt.v) : 0.2;
    if (!tgt || tgt.v < 0.6) bIss.push({ k: 'build', phase: 'build', t: L('Saltar a pressão com bola longa, mas sem uma referência forte no ar.', 'Bypassing the press with long balls, but without a strong aerial target.'), fix: L('Avançado de Referência na frente, ou sair a jogar curto.', 'A Target Forward up front, or play out short.') });
  }
  if (through && resist < 0.62) bIss.push({ k: 'build', phase: 'build', t: L(`Sair a jogar contra a pressão com uma primeira linha pouco resistente (${Math.round(resist * 20 * 10) / 10}).`, `Playing through the press with a first line that is not press-resistant (${Math.round(resist * 20 * 10) / 10}).`), fix: L('Balanced ou Contornar a Pressão, ou centrais/GR melhores com bola.', 'Balanced or Bypass Press, or centre-backs/keeper better on the ball.') });
  const build = { parts: { short: pct(bShort), support: pct(bSupport), resist: pct(through ? resist * 1.05 : Math.max(resist, 0.6)), long: pct(bLong) } };
  build.score = Math.round(build.parts.short * 0.35 + build.parts.support * 0.25 + build.parts.resist * 0.25 + build.parts.long * 0.15);

  // ---------------- DEFESA (sem bola, na forma sem bola)
  const dIss = [], dLinks = [];
  const outf = OOX.filter(x => x.si.group !== 'GK');
  const lane = (x) => (x.si.x < 38 ? 'L' : x.si.x > 62 ? 'R' : 'C');
  const pressQ = (p) => q(p, ['wor', 'sta', 'acc', 'agg', 'ant']), coverQ = (p) => q(p, ['pos', 'ant', 'pac', 'cnt']);
  // 1. quem sai a pressionar tem cobertura atrás
  const pressers = outf.filter(x => x.tag('HP'));
  let pCover = 0.65;
  if (pressers.length) {
    const vals = pressers.map(pr => {
      const cov = outf.filter(c => c !== pr && !c.tag('HP') && !c.tag('OUT') && (
          // um central que sai é coberto pelo central ao lado (Central de Cobertura); os outros por alguém atrás, no mesmo corredor
          (pr.si.group === 'DC' && c.si.group === 'DC' && Math.abs(c.si.y - pr.si.y) <= 6) ||
          (c.si.y < pr.si.y && (lane(c) === lane(pr) || lane(c) === 'C' || lane(pr) === 'C') && pr.si.y - c.si.y <= 30)))
        .map(c => ({ c, v: coverQ(c.p) * (c.tag('LOW') || c.tag('CW') || c.tag('DROP') ? 1.08 : 1) })).sort((m, n) => n.v - m.v)[0];
      if (cov) { dLinks.push({ type: 'press', from: pr, to: cov.c, v: Math.sqrt(pressQ(pr.p) * Math.min(1, cov.v)), phase: 'def', why: L(`${label(pr)} sai a pressionar, ${label(cov.c)} cobre`, `${label(pr)} steps out to press, ${label(cov.c)} covers`) }); return Math.sqrt(pressQ(pr.p) * Math.min(1, cov.v)); }
      dIss.push({ k: 'press', phase: 'def', t: L(`${label(pr)} sai a pressionar sem ninguém a cobrir atrás dele.`, `${label(pr)} steps out to press with nobody covering behind him.`), fix: L('Um papel de cobertura atrás (Central de Cobertura, Médio Defensivo de Proteção, Médio Centro de Cobertura) ou troque-o por um papel normal.', 'A covering role behind (Covering Centre-Back, Screening Defensive Midfielder, Wide Covering Central Midfielder) or switch him to a standard role.') });
      return 0.25;
    });
    pCover = vals.reduce((s, v) => s + v, 0) / vals.length;
  } else if (ti('loe') === 2) dIss.push({ k: 'press', phase: 'def', t: L('Linha de pressão alta sem nenhum papel que saia a pressionar.', 'A high line of engagement with no role that steps out to press.'), fix: L('Médio Trabalhador, Lateral Trabalhador ou Avançado Esforçado.', 'Pressing Central Midfielder, Pressing Full-Back or Tracking Centre Forward.') });
  // 2. dupla de centrais: um sai, outro cobre
  const cbs = outf.filter(x => x.si.group === 'DC');
  let cbPair = 0.8;
  if (cbs.length >= 2) {
    const st = cbs.filter(x => x.tag('HP')).length, cv = cbs.filter(x => x.tag('LOW') || x.tag('CW')).length;
    if (st >= 1 && cv >= 1) cbPair = 1; else if (st >= 2) { cbPair = 0.35; dIss.push({ k: 'cb', phase: 'def', t: L('Todos os centrais saem da linha a pressionar: o espaço nas costas fica aberto.', 'Every centre-back steps out of the line: the space behind is left open.'), fix: L('Combine um Central de Antecipação com um Central de Cobertura.', 'Pair a Stopping Centre-Back with a Covering Centre-Back.') }); }
  }
  // 3. protecção da zona à frente dos centrais
  const screens = outf.filter(x => ['DM', 'MC'].includes(x.si.group) && !x.tag('HP') && !x.tag('OUT'));
  const screen = screens.length ? Math.max(...screens.map(x => q(x.p, ['pos', 'ant', 'tck', 'cnt', 'tea']))) * (screens.some(x => x.tag('LOW')) ? 1.06 : 1) : 0.3;
  if (!screens.length) dIss.push({ k: 'screen', phase: 'def', t: L('Ninguém protege a zona à frente dos centrais: os médios saem todos a pressionar.', 'Nobody protects the zone in front of the centre-backs: every midfielder steps out to press.'), fix: L('Um Médio Defensivo de Proteção ou um Médio Centro de Proteção.', 'A Screening Defensive Midfielder or a Screening Central Midfielder.') });
  // 4. defesa dos flancos
  let flk = 0, nf = 0;
  ['L', 'R'].forEach(side => {
    const wa = outf.filter(x => x.si.side === side && ['AM_LR', 'M_LR'].includes(x.si.group));
    const fb = outf.filter(x => x.si.side === side && ['D_LR', 'WB_LR'].includes(x.si.group));
    if (!wa.length && !fb.length) return;
    nf++;
    const cwHelp = outf.some(x => x.tag('CW') && (x.si.side === side || lane(x) === 'C'));
    if (wa.some(x => x.tag('OUT'))) {
      const fbHold = fb.some(x => x.tag('LOW')), fbPress = fb.some(x => x.tag('HP'));
      if (fbPress && !cwHelp) { flk += 0.25; dIss.push({ k: 'flank', phase: 'def', t: L(`Flanco ${sideName(side)}: o extremo fica na frente e o lateral sai a pressionar — o corredor fica vazio.`, `${side === 'L' ? 'Left' : 'Right'} flank: the winger stays forward and the full-back steps out to press — the channel is left empty.`), fix: L('Lateral Fixo nesse lado, ou um Médio de Cobertura que feche o corredor.', 'A Holding Full-Back on that side, or a Wide Covering midfielder to close the channel.') }); }
      else if (fbHold || cwHelp) { flk += 0.8; const o = wa.find(x => x.tag('OUT')), h = fb.find(x => x.tag('LOW')) || outf.find(x => x.tag('CW')); if (h) dLinks.push({ type: 'flankdef', from: o, to: h, v: 0.8, phase: 'def', why: L(`Flanco ${sideName(side)}: ${label(o)} fica na frente, ${label(h)} segura o corredor`, `${side === 'L' ? 'Left' : 'Right'} flank: ${label(o)} stays forward, ${label(h)} holds the channel`) }); }
      else { flk += 0.5; dIss.push({ k: 'flank', phase: 'def', t: L(`Flanco ${sideName(side)}: o extremo fica na frente e o lateral fica sozinho contra dois.`, `${side === 'L' ? 'Left' : 'Right'} flank: the winger stays forward and the full-back is left alone against two.`), fix: L('Lateral Fixo, ou um Médio de Cobertura que dobre nesse lado.', 'A Holding Full-Back, or a Wide Covering midfielder doubling up on that side.') }); }
    } else if (wa.some(x => x.tag('TRK'))) { flk += 1; const tw = wa.find(x => x.tag('TRK')), f = fb[0]; if (f) dLinks.push({ type: 'flankdef', from: tw, to: f, v: 1, phase: 'def', why: L(`Flanco ${sideName(side)}: ${label(tw)} acompanha e dobra com ${label(f)}`, `${side === 'L' ? 'Left' : 'Right'} flank: ${label(tw)} tracks back and doubles up with ${label(f)}`) }); }
    else flk += 0.8;
  });
  const flankDef = nf ? flk / nf : 0.7;
  // 5. linha defensiva compatível com os centrais e o GR
  const dl = ti('dline'), gkSK = OOX.some(x => x.si.group === 'GK' && x.r.id === 'swk');
  const cbPace = cbs.length ? cbs.reduce((s, x) => s + q(x.p, ['pac', 'acc']), 0) / cbs.length : 0.6;
  const line = dl >= 3 ? Math.min(1, cbPace / 0.72) * (gkSK ? 1 : 0.72) : dl <= 1 ? 0.9 : Math.min(1, 0.75 + cbPace * 0.3);
  if (dl >= 3 && (cbPace < 0.62 || !gkSK)) dIss.push({ k: 'line', phase: 'def', t: L('Linha alta sem os perfis para a suportar (centrais rápidos e Guarda-Redes Líbero).', 'A high line without the profiles to support it (quick centre-backs and a Sweeper Keeper).'), fix: L('Baixe a linha, ou Guarda-Redes Líbero e centrais com Velocidade/Aceleração.', 'Drop the line, or a Sweeper Keeper and centre-backs with Pace/Acceleration.') });
  // 6. jogo aéreo defensivo
  const airD = cbs.length ? cbs.map(x => q(x.p, ['hea', 'jum', 'str', 'pos'])).sort((m, n) => n - m).slice(0, 2).reduce((s, v, i, arr) => s + v / arr.length, 0) : 0.5;
  if (airD < 0.6 && ti('cross_eng') === 0) dIss.push({ k: 'air', phase: 'def', t: L('Convida cruzamentos com centrais fracos no ar.', 'Inviting crosses with centre-backs weak in the air.'), fix: L('Impedir Cruzamentos, ou centrais mais fortes no ar.', 'Stop Crosses, or centre-backs stronger in the air.') });
  const outN = outf.filter(x => x.tag('OUT')).length;
  const def = { parts: { press: pct(pCover), cb: pct(cbPair), screen: pct(Math.min(1, screen)), flankd: pct(flankDef), line: pct(line), air: pct(airD) } };
  def.score = Math.round(def.parts.press * 0.22 + def.parts.cb * 0.12 + def.parts.screen * 0.2 + def.parts.flankd * 0.22 + def.parts.line * 0.14 + def.parts.air * 0.10);

  // ---------------- TRANSIÇÕES
  const tIss = [], tLinks = [];
  const rd = IPX.filter(x => x.tag('RD') && x.si.group !== 'GK');
  const rdQ = rd.length ? rd.reduce((s, x) => s + q(x.p, ['pac', 'acc', 'pos', 'ant']), 0) / rd.length : 0.4;
  const rdN = rd.length, rdCount = { 0: 0.1, 1: 0.3, 2: 0.65, 3: 1, 4: 1, 5: 0.8 }[rdN] ?? 0.7;
  const loss = rdCount * (0.55 + 0.45 * rdQ);
  if (rdN >= 3 && rdQ < 0.6) tIss.push({ k: 'loss', phase: 'trans', t: L('Quem fica atrás é lento: os contra-ataques adversários chegam em vantagem.', 'Those who stay back are slow: opposition counter-attacks arrive with the advantage.'), fix: L('Um Médio Defensivo / lateral mais rápido na rest-defence, ou linha mais baixa.', 'A quicker Defensive Midfielder / full-back in the rest-defence, or a deeper line.') });
  const breakers = OOX.filter(x => x.tag('OUT')).concat(IPX.filter(x => x.tag('RUN') && ['ST', 'AM_LR'].includes(x.si.group)));
  const bestRun = breakers.map(x => ({ x, v: q(x.p, ['pac', 'acc', 'otb', 'dri']) })).sort((m, n) => n.v - m.v)[0];
  const releaser = IPX.filter(x => ['DM', 'MC', 'AMC'].includes(x.si.group) || x.tag('CR')).map(x => ({ x, v: q(x.p, ['vis', 'pas', 'dec', 'tec']) })).sort((m, n) => n.v - m.v)[0];
  const win = bestRun && releaser ? Math.sqrt(bestRun.v * releaser.v) : 0.35;
  if (bestRun && releaser && bestRun.x !== releaser.x) tLinks.push({ type: 'counter', from: releaser.x, to: bestRun.x, v: win, phase: 'trans', why: L(`${label(releaser.x)} lança ${label(bestRun.x)} no contra-ataque`, `${label(releaser.x)} releases ${label(bestRun.x)} on the counter`) });
  if (ti('att_trans') === 2 && win < 0.62) tIss.push({ k: 'win', phase: 'trans', t: L('Contra-Ataque escolhido, mas sem velocidade na frente ou sem quem lance a bola.', 'Counter-Attack chosen, but without pace up front or anyone to release the ball.'), fix: L('Um Avançado de Desmarcação / Extremo de Transição rápido e um médio com Visão.', 'A quick Splitting Outlet / Wide Outlet and a midfielder with Vision.') });
  if (outN >= 3) tIss.push({ k: 'out', phase: 'trans', t: L(`${outN} jogadores ficam na frente sem defender: o bloco fica em inferioridade.`, `${outN} players stay forward without defending: the block is outnumbered.`), fix: L('No máximo 1–2 papéis de saída.', 'At most 1–2 outlet roles.') });
  const outBal = outN === 0 ? 0.7 : outN <= 2 ? 1 : 0.4;
  const trans = { parts: { loss: pct(loss), win: pct(win), out: pct(outBal) } };
  trans.score = Math.round(trans.parts.loss * 0.5 + trans.parts.win * 0.35 + trans.parts.out * 0.15);

  const phases = { build, att: { score: att.score, parts: att.parts }, def, trans };
  const score = Math.round(Object.entries(SYN_PHASE_W).reduce((s, [k, w]) => s + phases[k].score * w, 0));
  const links = [...bLinks, ...att.links, ...dLinks.sort((m, n) => n.v - m.v).slice(0, 5), ...tLinks];
  const issues = [...bIss, ...att.issues, ...dIss, ...tIss];
  return { score, phases, parts: att.parts, attScore: att.score, links, issues, creators: att.creators, boxN: att.boxN, outN, rdN };
}
const SYN_PHASES = () => ({ build: L('Construção', 'Build-up'), att: L('Ataque', 'Attack'), def: L('Defesa', 'Defence'), trans: L('Transições', 'Transitions') });
const SYN_NAMES = () => ({ ...SYN_NAMES_ATT(),
  short: L('Saída curta: quem passa → quem recebe', 'Short build-up: passer → receiver'), support: L('Apoios entre linhas', 'Support between the lines'), resist: L('Resistência à pressão', 'Press resistance'), long: L('Bola longa → referência', 'Long ball → target'),
  press: L('Pressão com cobertura', 'Pressing with cover'), cb: L('Dupla de centrais (sai + cobre)', 'Centre-back pair (step + cover)'), screen: L('Proteção à frente dos centrais', 'Screen in front of the centre-backs'),
  flankd: L('Defesa dos flancos', 'Flank defence'), line: L('Linha vs. perfis', 'Line vs. profiles'), air: L('Jogo aéreo defensivo', 'Aerial defence'),
  loss: L('Reação à perda (rest-defence)', 'Reaction to losing the ball (rest-defence)'), win: L('Contra-ataque: quem lança → quem arranca', 'Counter: releaser → runner'), out: L('Saídas vs. bloco', 'Outlets vs. block') });
