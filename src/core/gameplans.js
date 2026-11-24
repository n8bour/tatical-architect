// ============================================================
// Situações de jogo: a partir da tática base e do plantel, o que mudar quando
// é preciso marcar, segurar o resultado, jogar fora contra mais fortes, etc.
// Cada mudança traz o porquê, com os atributos dos jogadores. Devolve a tática
// ajustada (para guardar como tática B/C no FM27), as substituições sugeridas
// e o impacto na sinergia (o que se ganha e o que se arrisca).
// ============================================================
const SCENARIOS = () => [
  { k: 'chase', ic: '⚡', n: L('Precisamos de golos', 'We need goals'), d: L('A perder ou empatado quando só a vitória serve: mais gente na frente, mais ritmo, mais risco.', 'Losing, or drawing when only a win will do: more bodies forward, more tempo, more risk.') },
  { k: 'protect', ic: '🛡', n: L('Segurar o resultado', 'Protect the lead'), d: L('A ganhar na parte final: menos risco, bloco compacto, gerir o tempo.', 'Winning late on: less risk, a compact block, manage the clock.') },
  { k: 'away_strong', ic: '⛰', n: L('Fora contra mais fortes', 'Away to a stronger side'), d: L('Prudência primeiro: bloco baixo, transições rápidas, anular os perigosos deles.', 'Caution first: a low block, quick transitions, shut down their dangerous players.') },
  { k: 'away_equal', ic: '⚖', n: L('Fora contra equipa do mesmo nível', 'Away to an evenly matched side'), d: L('Equilíbrio: bloco médio, não dar espaço nas costas, aproveitar as transições.', 'Balance: a mid block, no space in behind, make the most of transitions.') },
  { k: 'home_weak', ic: '🏟', n: L('Em casa contra mais fracos (bloco baixo)', 'At home to a weaker side (low block)'), d: L('O adversário fecha-se: paciência, largura, gente na área e pressão logo após a perda.', 'The opponent sits deep: patience, width, bodies in the box and pressure straight after losing the ball.') },
];

// intensidade da tática (o que a sinergia não mede): quanto vai para a frente e quanto fica a proteger
function gameIntent(t) {
  const MI = Math.max(0, MENTALITIES.indexOf(t.ment)), ti = (k) => tiVal(t, k);
  const ips = t.slots.map(s => ({ r: ROLE_BY_ID[s.ip], a: slotInfo(t.ipForm, s.ipCode), o: ROLE_BY_ID[s.oop], pis: s.pis || [] })).filter(x => x.r && x.a);
  const box = ips.filter(x => x.r.tags.includes('BOX') || (x.r.tags.includes('IN') && x.r.tags.includes('RUN'))).length;
  const advFB = ips.filter(x => ['D_LR', 'WB_LR'].includes(x.a.group) && (x.r.tags.includes('ADV') || x.r.tags.includes('W') || x.pis.includes('Get further forward'))).length;
  const rd = ips.filter(x => x.r.tags.includes('RD') && x.a.group !== 'GK').length;
  const hold = ips.filter(x => x.o && (x.o.tags.includes('LOW') || x.o.tags.includes('CW') || x.o.tags.includes('TRK'))).length;
  const outs = ips.filter(x => x.o && x.o.tags.includes('OUT')).length;
  const att = MI / 6 * 28 + Math.min(box, 5) / 5 * 20 + advFB / 2 * 12 + ti('tempo') / 4 * 10 + ti('width') / 4 * 6 + ti('loe') / 2 * 8 + ti('dline') / 4 * 8 + (ti('def_trans') === 2 ? 4 : 0) + (ti('timewaste') === 0 ? 4 : 0);
  const sec = Math.min(rd, 4) / 4 * 26 + (1 - ti('dline') / 4) * 14 + (2 - ti('loe')) / 2 * 10 + (ti('def_trans') === 0 ? 8 : ti('def_trans') === 1 ? 4 : 0) + Math.min(hold, 5) / 5 * 16 + ({ 0: 8, 1: 6, 2: 3 }[outs] ?? 0) + (6 - MI) / 6 * 12 + (ti('tackling') === 0 ? 3 : 0) + (ti('timewaste') === 2 ? 3 : 0);
  return { att: Math.round(Math.min(100, att)), sec: Math.round(Math.min(100, sec)) };
}
const gpRoleName = (r) => (r ? (typeof roleName === 'function' ? roleName(r) : r.name) : '');
function gamePlan(base, players, k, opts = {}) {
  const t = JSON.parse(JSON.stringify(base)); t.ti = { ...(base.ti || {}) };
  const sc = SCENARIOS().find(s => s.k === k) || SCENARIOS()[0];
  t.name = `${typeof tName === 'function' ? tName(base) : base.name} — ${sc.n}`; t.source = 'gameplan';
  const pm = Object.fromEntries((players || []).map(p => [p.id, p]));
  const a = (p, key) => (p && p.a && p.a[key] != null ? p.a[key] : 10);
  const q = (p, ks) => ks.reduce((s, key) => s + a(p, key), 0) / ks.length;
  const f1 = (x) => x.toFixed(1);
  const changes = [], tweaks = [], subs = [], ois = [], notes = [];
  const X = () => t.slots.map((s, i) => ({ s, i, p: pm[s.pid], a: slotInfo(t.ipForm, s.ipCode), b: slotInfo(t.oopForm, s.oopCode), ip: ROLE_BY_ID[s.ip], oop: ROLE_BY_ID[s.oop] }));
  const xi = X(), inXI = new Set(t.slots.map(s => s.pid)), bench = (players || []).filter(p => !inXI.has(p.id));
  const grp = (gs) => xi.filter(x => gs.includes(x.a.group));
  const avgP = (list, ks) => list.length ? list.reduce((s, x) => s + q(x.p, ks), 0) / list.length : 10;

  // ---- perfil do plantel em campo
  const sts = grp(['ST']), att = grp(['ST', 'AMC', 'AM_LR', 'M_LR']), cbs = grp(['DC']);
  const stAir = sts.length ? Math.max(...sts.map(x => q(x.p, ['hea', 'jum', 'str']))) : 8;
  const attPace = avgP(att, ['pac', 'acc']), cbPace = avgP(cbs, ['pac', 'acc']), cbAir = avgP(cbs, ['hea', 'jum', 'str']);
  const tech = avgP(grp(['DC', 'DM', 'MC', 'D_LR', 'WB_LR']), ['pas', 'fir', 'tec', 'cmp']);
  const press = avgP(xi.filter(x => x.a.group !== 'GK'), ['wor', 'sta', 'acc', 'agg']);
  const lonTop = xi.map(x => a(x.p, 'lon')).sort((m, n) => n - m).slice(0, 3).reduce((s, v) => s + v, 0) / 3;
  const airTop = xi.filter(x => x.a.group !== 'GK').map(x => q(x.p, ['hea', 'jum', 'str'])).sort((m, n) => n - m).slice(0, 4).reduce((s, v) => s + v, 0) / 4;
  const bestCross = Math.max(0, ...xi.filter(x => ['D_LR', 'WB_LR', 'M_LR', 'AM_LR'].includes(x.a.group)).map(x => a(x.p, 'cro')));
  const firstLine = avgP(grp(['GK', 'DC']), ['pas', 'fir', 'cmp']);

  // ---- ferramentas
  const cur = (key) => tiVal(t, key);
  const setTI = (key, v, why) => { const d = TI_BY_KEY[key]; if (!d || v == null) return; v = Math.max(0, Math.min(d.opts.length - 1, v)); const was = cur(key); if (was === v) return;
    if (v === d.def) delete t.ti[key]; else t.ti[key] = v; changes.push({ kind: 'ti', key, label: d.n, from: d.opts[was], to: d.opts[v], why }); };
  const MI = (m) => MENTALITIES.indexOf(m);
  const setMent = (m, why) => { if (t.ment === m) return; changes.push({ kind: 'ment', label: L('Mentalidade', 'Mentality'), from: t.ment, to: m, why }); t.ment = m; };
  const stepMent = (d, lo, hi) => { const i = Math.max(MI(lo), Math.min(MI(hi), MI(t.ment) + d)); return MENTALITIES[i]; };
  // a troca de papel é testada contra a sinergia: em planos para atacar só entra se não piorar o ataque,
  // em planos para defender só entra se não piorar a defesa; entre as opções válidas escolhe a que mais ajuda
  const GUARD = { chase: 'att', home_weak: 'att', protect: 'def', away_strong: 'def', away_equal: 'def' }[k];
  const fitOf = (x, r) => (x.p ? (roleScore(x.p, r) || { score: 0 }).score : 50);
  const setRole = (x, phase, cands, why) => {
    const si = phase === 'IP' ? x.a : x.b, key = phase === 'IP' ? 'ip' : 'oop';
    const ok = cands.map(id => ROLE_BY_ID[id]).filter(r => r && validRole(r.id, si.group, phase, si));
    if (!ok.length) return false;
    const orig = x.s[key];
    const metric = () => { const rep = synergyReport(t, players); return rep.phases[GUARD].score + (GUARD === 'att' ? rep.phases.att.parts.box * 0.05 : 0); };
    const m0 = metric() + fitOf(x, ROLE_BY_ID[orig]) * 0.1;
    let best = null, bestM = -1;
    for (const r of ok) { x.s[key] = r.id; const m = metric() + fitOf(x, r) * 0.1; if (m > bestM) { bestM = m; best = r; } }
    x.s[key] = orig;
    if (!best || best.id === orig || bestM < m0 + 0.5) return false;   // não melhora o que o plano procura: fica como estava
    changes.push({ kind: 'role', phase, code: phase === 'IP' ? x.s.ipCode : x.s.oopCode, player: x.p ? x.p.name : '', from: gpRoleName(ROLE_BY_ID[x.s[key]]), to: gpRoleName(best), why });
    x.s[key] = best.id; return true;
  };
  const OPP_PI = { 'Hold position': 'Get further forward', 'Get further forward': 'Hold position', 'Take more risks': 'Take fewer risks', 'Take fewer risks': 'Take more risks', 'Shoot more': 'Shoot less', 'Press more': 'Press less', 'Press less': 'Press more', 'Dribble more': 'Dribble less' };
  const addPI = (x, pi, why) => { if (!x.p || !PLAYER_INSTRUCTIONS.some(z => z.n === pi)) return; x.s.pis = (x.s.pis || []).filter(z => z !== OPP_PI[pi]);
    if (x.s.pis.includes(pi) || x.s.pis.length >= 3) return; x.s.pis.push(pi); changes.push({ kind: 'pi', code: x.s.ipCode, player: x.p.name, to: pi, why }); };
  const dropPI = (x, pi) => { if (x.s.pis && x.s.pis.includes(pi)) { x.s.pis = x.s.pis.filter(z => z !== pi); changes.push({ kind: 'pi', code: x.s.ipCode, player: x.p ? x.p.name : '', to: pi, removed: true, why: L('Deixa de fazer sentido neste cenário.', 'No longer makes sense in this scenario.') }); } };
  const tweak = (n, why) => { if (TACTICAL_TWEAKS.some(z => z.n === n)) tweaks.push({ n, why }); };
  // substituições: melhor jogador do banco para a casa, pelo critério do cenário
  const proposeSubs = (score, slotsOk, n, why) => {
    if (!bench.length) return;
    const used = new Set();
    const opts = xi.filter(x => slotsOk(x) && x.p).map(x => {
      const cand = bench.filter(b => !used.has(b.id)).map(b => ({ b, v: score(b, x) * Math.min(posFactor(b, x.a.group, x.a.side), 1) })).sort((m, n) => n.v - m.v)[0];
      return cand ? { x, b: cand.b, gain: cand.v - score(x.p, x) } : null;
    }).filter(o => o && o.gain > 0.4).sort((m, n) => n.gain - m.gain);
    for (const o of opts) { if (subs.length >= n || used.has(o.b.id)) continue; used.add(o.b.id);
      subs.push({ out: o.x.p.name, in: o.b.name, code: o.x.s.ipCode, role: gpRoleName(ROLE_BY_ID[o.x.s.ip]), why: why(o.b, o.x), gain: o.gain }); }
  };
  const attackQ = (p) => q(p, ['fin', 'otb', 'acc', 'pac', 'dri', 'cmp', 'ant']);
  const defendQ = (p) => q(p, ['tck', 'pos', 'mar', 'wor', 'sta', 'ant', 'cnt']);

  if (k === 'chase') {
    setMent(stepMent(+1, 'Balanced', 'Very Attacking'), L('Mais jogadores entram no ataque e arriscam mais.', 'More players join the attack and take more risks.'));
    setTI('tempo', Math.max(cur('tempo'), 3), L('Menos tempo para o adversário se organizar.', 'Less time for the opponent to get organised.'));
    setTI('width', Math.max(cur('width'), 3), L('Esticar o bloco deles abre espaços por dentro.', 'Stretching their block opens space inside.'));
    if (stAir >= 14) { setTI('passing', Math.max(cur('passing'), 3), L(`Referência forte no ar na frente (${f1(stAir)}): a bola chega mais depressa à área.`, `A strong aerial target up front (${f1(stAir)}): the ball reaches the box faster.`)); setTI('crossing', 3, L(`Cruzamentos altos para a referência (${f1(stAir)}).`, `High crosses for the target (${f1(stAir)}).`)); tweak('Play Longer', L('Bolas para a referência aérea.', 'Balls up to the aerial target.')); setTI('goal_kicks', 2, L('Pontapés de baliza para a referência aérea.', 'Goal kicks to the aerial target.')); }
    else if (attPace >= 13) { setTI('reception', 0, L(`Atacantes rápidos (${f1(attPace)}): bola para o espaço nas costas.`, `Quick attackers (${f1(attPace)}): balls into the space behind.`)); setTI('crossing', 0, L('Sem referência aérea: cruzamentos rasteiros.', 'No aerial target: low crosses.')); }
    if (lonTop >= 13) setTI('long_shots', 2, L(`Há quem remate bem de fora (top 3: ${f1(lonTop)}).`, `You have good long-range shooters (top 3: ${f1(lonTop)}).`));
    if (airTop >= 13.5) setTI('set_pieces', 2, L(`Vantagem nas bolas paradas (${f1(airTop)}).`, `An edge at set pieces (${f1(airTop)}).`));
    setTI('timewaste', 0, L('Cada segundo conta.', 'Every second counts.'));
    setTI('gk_speed', 2, L('O GR repõe rápido.', 'The keeper distributes quickly.'));
    setTI('def_trans', 2, L('Recuperar a bola logo após a perda, perto da área deles.', 'Win the ball back straight after losing it, near their box.'));
    if (press >= 12) { setTI('loe', Math.max(cur('loe'), 2), L(`O XI aguenta pressão alta (${f1(press)}).`, `The XI can sustain a high press (${f1(press)}).`)); setTI('trigger', Math.max(cur('trigger'), 3), L('Pressionar mais vezes para recuperar mais à frente.', 'Press more often to win the ball higher.')); }
    setTI('dline', Math.max(cur('dline'), 3), L('Linha subida para encurtar o campo.', 'A higher line to compress the pitch.'));
    tweak('More Urgency', L('Sobe ritmo e verticalidade de uma vez.', 'Raises tempo and verticality in one go.'));
    if (bestCross >= 14) tweak('Exploit Width', L(`Cruzadores fortes (${bestCross}).`, `Strong crossers (${bestCross}).`));
    // papéis: laterais sobem, um médio passa a chegar à área, o AM passa a segundo avançado, avançado que baixa passa a finalizador
    grp(['D_LR']).forEach(x => setRole(x, 'IP', ['wb_ip', 'pwb', 'iwb'], L('Mais um no ataque: por fora se o extremo entra por dentro, por dentro se o extremo dá largura.', 'One more in attack: outside if the winger comes inside, inside if the winger gives width.')));
    grp(['WB_LR']).forEach(x => setRole(x, 'IP', ['awb', 'wb_ip'], L('Ala projetado: quase um extremo.', 'Advanced wing-back: almost a winger.')));
    const mid = grp(['MC']).filter(x => x.ip && !x.ip.tags.includes('BOX') && !x.ip.tags.includes('CR')).sort((m, n) => attackQ(n.p) - attackQ(m.p))[0];
    if (mid) setRole(mid, 'IP', ['chm', 'am_ip'], L('Mais um a chegar à área.', 'One more arriving in the box.'));
    const dmx = grp(['DM']).filter(x => x.ip && x.ip.id === 'dm_ip')[0];
    if (dmx) setRole(dmx, 'IP', ['b2b', 'b2bp', 'dlp'], L('O médio defensivo passa a ligar e a chegar à frente.', 'The defensive midfielder starts linking and getting forward.'));
    grp(['AMC']).forEach(x => setRole(x, 'IP', ['ss', 'am_ip'], L('Junta-se ao avançado na área.', 'Joins the striker in the box.')));
    sts.forEach(x => { if (x.ip && x.ip.tags.includes('DROP')) setRole(x, 'IP', ['cf_ip', 'poa', 'chf', 'tf'], L('Precisamos dele na área, não a baixar.', 'We need him in the box, not dropping deep.')); });
    // sem bola: um extremo e o avançado ficam na frente para a transição
    const fastW = grp(['AM_LR']).sort((m, n) => q(n.p, ['pac', 'acc']) - q(m.p, ['pac', 'acc']))[0];
    if (fastW) setRole(fastW, 'OOP', ['wow', 'iow'], L('Fica na frente para atacar logo que se recupera a bola.', 'Stays forward to attack as soon as the ball is won.'));
    sts.forEach(x => setRole(x, 'OOP', ['cf_oop', 'socf'], L('Poupa-se a recuar: fica para finalizar.', 'Saves his legs: stays up to finish.')));
    // instruções individuais
    X().forEach(x => {
      if (!x.p || x.a.group === 'GK') return;
      if (['D_LR', 'WB_LR', 'DM'].includes(x.a.group)) dropPI(x, 'Hold position');
      if (a(x.p, 'lon') >= 14 && x.a.group !== 'DC') addPI(x, 'Shoot more', L(`Remates de Longe ${a(x.p, 'lon')}: arriscar de fora.`, `Long Shots ${a(x.p, 'lon')}: try from distance.`));
      if (ROLE_BY_ID[x.s.ip].tags.includes('CR') && a(x.p, 'vis') >= 13) addPI(x, 'Take more risks', L(`Visão ${a(x.p, 'vis')}: tentar o passe de ruptura.`, `Vision ${a(x.p, 'vis')}: try the killer pass.`));
      if (['AM_LR', 'M_LR'].includes(x.a.group) && a(x.p, 'dri') >= 14) addPI(x, 'Dribble more', L(`Drible ${a(x.p, 'dri')}: procurar o um-para-um.`, `Dribbling ${a(x.p, 'dri')}: take players on.`));
      if (['ST', 'AMC'].includes(x.a.group) && q(x.p, ['pac', 'acc']) >= 14 && a(x.p, 'otb') >= 13) addPI(x, 'Attack the Defensive Line', L('Rápido e com bom movimento: atacar a profundidade.', 'Quick with good movement: attack depth.'));
    });
    proposeSubs((p, x) => attackQ(p) + (roleScore(p, ROLE_BY_ID[x.s.ip]) || { score: 0 }).score / 20, x => ['ST', 'AMC', 'AM_LR', 'M_LR', 'MC'].includes(x.a.group), 2,
      (b, x) => L(`${b.name} é mais perigoso aqui (Finalização ${a(b, 'fin')}, Sem Bola ${a(b, 'otb')}, Aceleração ${a(b, 'acc')}).`, `${b.name} is more dangerous here (Finishing ${a(b, 'fin')}, Off the Ball ${a(b, 'otb')}, Acceleration ${a(b, 'acc')}).`));
    if (sts.length === 1) notes.push(L('Nos últimos 10–15 minutos, se ainda faltar o golo: troque o médio mais defensivo por um segundo avançado (por exemplo, passar a duas pontas).', 'In the last 10–15 minutes, if the goal still has not come: swap your most defensive midfielder for a second striker (e.g. switch to two up front).'));
  }

  if (k === 'protect') {
    setMent(stepMent(-1, 'Defensive', 'Balanced'), L('Menos gente no ataque, mais atrás da bola.', 'Fewer bodies in attack, more behind the ball.'));
    const keepBall = tech >= 13 && !(attPace >= 14);
    if (keepBall) { setTI('passing', Math.min(cur('passing'), 1), L(`Equipa técnica (${f1(tech)}): guardar a bola é a melhor defesa.`, `A technical side (${f1(tech)}): keeping the ball is the best defence.`)); setTI('patience', 2, L('Sem pressa: só finalizar com a jogada certa.', 'No rush: only shoot from the right move.')); setTI('att_trans', 0, L('Ao recuperar, reorganizar com a bola.', 'On winning the ball, reset in possession.')); }
    else { setTI('att_trans', attPace >= 13 ? 2 : 0, attPace >= 13 ? L(`Avançados rápidos (${f1(attPace)}): o golo da tranquilidade vem em contra-ataque.`, `Quick forwards (${f1(attPace)}): the clinching goal comes on the counter.`) : L('Ao recuperar, reorganizar com a bola.', 'On winning the ball, reset in possession.')); if (firstLine < 11.5) setTI('buildup', 2, L(`Primeira linha fraca com bola (${f1(firstLine)}): não arriscar perto da baliza.`, `First line weak on the ball (${f1(firstLine)}): no risks near your goal.`)); }
    setTI('tempo', Math.min(cur('tempo'), 1), L('Ritmo baixo: o tempo corre a teu favor.', 'A lower tempo: the clock is on your side.'));
    setTI('timewaste', 2, L('Gerir o relógio.', 'Run down the clock.'));
    setTI('gk_speed', 0, L('O GR abranda a reposição.', 'The keeper slows down distribution.'));
    setTI('loe', cbPace < 12.5 || press < 11.5 ? 0 : Math.min(cur('loe'), 1), L('Bloco compacto, sem espaço entre linhas.', 'A compact block, no space between the lines.'));
    setTI('dline', Math.min(cur('dline'), cbPace < 12.5 ? 1 : 2), cbPace < 12.5 ? L(`Centrais pouco rápidos (${f1(cbPace)}): linha mais baixa, sem espaço nas costas.`, `Centre-backs short of pace (${f1(cbPace)}): a deeper line, no space in behind.`) : L('Linha normal: proteger a profundidade.', 'A standard line: protect depth.'));
    setTI('trigger', Math.min(cur('trigger'), 1), L('Pressionar menos poupa energia e evita deixar buracos.', 'Pressing less saves energy and avoids leaving gaps.'));
    setTI('def_trans', 0, L('Ao perder, reagrupar primeiro.', 'On losing it, regroup first.'));
    setTI('tackling', 0, L('Evitar faltas e penáltis no fim.', 'Avoid fouls and penalties late on.'));
    setTI('cross_eng', cbAir >= 14 ? 0 : 2, cbAir >= 14 ? L(`Centrais dominantes no ar (${f1(cbAir)}): deixa-os cruzar.`, `Centre-backs dominant in the air (${f1(cbAir)}): let them cross.`) : L(`Centrais menos fortes no ar (${f1(cbAir)}): impedir os cruzamentos.`, `Centre-backs weaker in the air (${f1(cbAir)}): stop the crosses.`));
    setTI('set_pieces', 0, L('Manter a bola em jogo: menos bolas paradas a favor = menos contra-ataques contra.', 'Keep the ball in play: fewer set pieces for you = fewer counters against.'));
    if (cur('loe') >= 1) tweak('Press Less', L('Baixa a intensidade da pressão de uma vez.', 'Lowers the pressing intensity in one go.'));
    grp(['D_LR']).forEach(x => { setRole(x, 'OOP', ['hfb', 'fb_oop'], L('Segura a posição atrás.', 'Holds his position at the back.')); setRole(x, 'IP', ['fb_ip', 'ifb'], L('Não sobe: fica a proteger o corredor.', 'Does not push on: stays to protect the flank.')); });
    grp(['WB_LR']).forEach(x => { setRole(x, 'OOP', ['hwb', 'wb_oop'], L('Fecha a linha de 5.', 'Completes the back five.')); setRole(x, 'IP', ['wb_ip', 'iwb'], L('Sobe menos.', 'Pushes on less.')); });
    grp(['DM']).forEach(x => setRole(x, 'OOP', ['sdm', 'dm_oop'], L('Protege a zona à frente dos centrais.', 'Protects the zone in front of the centre-backs.')));
    grp(['MC']).forEach(x => setRole(x, 'OOP', ['scm', 'wccm', 'cm_oop'], L('Fecha os espaços centrais.', 'Closes the central spaces.')));
    grp(['AM_LR']).forEach(x => setRole(x, 'OOP', ['tw', 'w_oop'], L('Acompanha o lateral deles até atrás.', 'Tracks their full-back all the way back.')));
    grp(['M_LR']).forEach(x => setRole(x, 'OOP', ['twm', 'wm_oop'], L('Acompanha o recuo.', 'Tracks back.')));
    grp(['AMC']).forEach(x => setRole(x, 'OOP', ['tam', 'am_oop'], L('Recua para ajudar o meio-campo.', 'Drops back to help the midfield.')));
    const fastS = sts.sort((m, n) => q(n.p, ['pac', 'acc']) - q(m.p, ['pac', 'acc']));
    fastS.forEach((x, i) => setRole(x, 'OOP', i === 0 && attPace >= 13 && !keepBall ? ['socf', 'cf_oop'] : ['tcf', 'cf_oop'], i === 0 && attPace >= 13 && !keepBall ? L('Fica como saída para o contra-ataque.', 'Stays up as the outlet for the counter.') : L('Pressiona a saída de bola deles.', 'Presses their build-up.')));
    X().forEach(x => {
      if (!x.p) return;
      ['Get further forward', 'Take more risks', 'Shoot more', 'Dribble more'].forEach(pi => dropPI(x, pi));
      if (['D_LR', 'WB_LR', 'DM'].includes(x.a.group)) addPI(x, 'Hold position', L('Não deixar espaço nas costas.', 'Leave no space in behind.'));
      if (['DC', 'DM'].includes(x.a.group) && a(x.p, 'pas') <= 12) addPI(x, 'Take fewer risks', L(`Passe ${a(x.p, 'pas')}: nada de perdas perto da área.`, `Passing ${a(x.p, 'pas')}: no turnovers near your box.`));
    });
    proposeSubs((p, x) => defendQ(p) + (roleScore(p, ROLE_BY_ID[x.s.oop]) || { score: 0 }).score / 20, x => ['AM_LR', 'M_LR', 'AMC', 'MC', 'ST'].includes(x.a.group), 2,
      (b, x) => L(`${b.name} defende melhor aqui (Desarme ${a(b, 'tck')}, Posicionamento ${a(b, 'pos')}, Índice de Trabalho ${a(b, 'wor')}) e tem as pernas frescas.`, `${b.name} defends better here (Tackling ${a(b, 'tck')}, Positioning ${a(b, 'pos')}, Work Rate ${a(b, 'wor')}) and has fresh legs.`));
  }

  if (k === 'away_strong') {
    setMent(MI(t.ment) > MI('Cautious') ? 'Cautious' : t.ment, L('Contra equipas mais fortes, primeiro não sofrer.', 'Against stronger sides, first do not concede.'));
    setTI('loe', press >= 13.5 ? 1 : 0, press >= 13.5 ? L(`Bloco médio: o XI tem energia (${f1(press)}) para fechar e sair.`, `Mid block: the XI has the energy (${f1(press)}) to close down and break.`) : L('Bloco baixo: menos espaço para a qualidade deles.', 'Low block: less space for their quality.'));
    setTI('dline', cbPace >= 14 ? 2 : 1, cbPace >= 14 ? L(`Centrais rápidos (${f1(cbPace)}): linha normal.`, `Quick centre-backs (${f1(cbPace)}): a standard line.`) : L(`Linha mais baixa: os avançados deles não podem ter espaço nas costas (centrais ${f1(cbPace)}).`, `A deeper line: their forwards must not get space in behind (centre-backs ${f1(cbPace)}).`));
    setTI('trigger', Math.min(cur('trigger'), 1), L('Escolher os momentos de pressão.', 'Pick your pressing moments.'));
    setTI('def_trans', 0, L('Ao perder, reagrupar: não deixar espaço para a transição deles.', 'On losing it, regroup: no space for their transition.'));
    setTI('att_trans', attPace >= 12.5 ? 2 : 1, attPace >= 12.5 ? L(`Atacantes rápidos (${f1(attPace)}): o contra-ataque é a arma.`, `Quick attackers (${f1(attPace)}): the counter is your weapon.`) : L('Transição equilibrada: sem velocidade para contra-atacar sempre.', 'A balanced transition: not enough pace to always counter.'));
    setTI('passing', Math.max(cur('passing'), 3), L('Chegar à frente sem dar tempo para pressionarem.', 'Get forward without giving them time to press.'));
    if (attPace >= 13) setTI('reception', 0, L('Bola para o espaço nas costas deles.', 'Balls into the space behind them.'));
    if (firstLine < 12.5) setTI('buildup', 2, L(`Primeira linha pouco segura com bola (${f1(firstLine)}): saltar a pressão deles.`, `First line not secure on the ball (${f1(firstLine)}): bypass their press.`));
    setTI('patience', 0, L('Cruzar cedo para aproveitar a defesa desorganizada.', 'Hit early crosses to catch the defence disorganised.'));
    setTI('cross_eng', cbAir >= 14 ? 0 : 2, cbAir >= 14 ? L(`Centrais fortes no ar (${f1(cbAir)}).`, `Centre-backs strong in the air (${f1(cbAir)}).`) : L(`Centrais menos fortes no ar (${f1(cbAir)}): impedir cruzamentos.`, `Centre-backs weaker in the air (${f1(cbAir)}): stop crosses.`));
    // uma saída para o contra-ataque, o resto a defender
    const outs = [...grp(['ST']), ...grp(['AM_LR'])].sort((m, n) => q(n.p, ['pac', 'acc', 'otb']) - q(m.p, ['pac', 'acc', 'otb']));
    outs.forEach((x, i) => {
      if (i === 0) setRole(x, 'OOP', x.a.group === 'ST' ? ['socf', 'cocf'] : ['wow', 'iow'], L('A saída para o contra-ataque: o mais rápido fica na frente.', 'The counter-attack outlet: the quickest stays forward.'));
      else setRole(x, 'OOP', x.a.group === 'ST' ? ['tcf', 'cf_oop'] : ['tw', 'w_oop'], L('Ajuda a defender.', 'Helps to defend.'));
    });
    grp(['DM']).forEach(x => setRole(x, 'OOP', ['sdm', 'dm_oop'], L('Protege à frente dos centrais.', 'Screens in front of the centre-backs.')));
    grp(['D_LR']).forEach(x => setRole(x, 'OOP', ['hfb', 'fb_oop'], L('Segura o corredor atrás do extremo.', 'Holds the channel behind the winger.')));
    grp(['M_LR']).forEach(x => setRole(x, 'OOP', ['twm', 'wm_oop'], L('Acompanha o recuo.', 'Tracks back.')));
    if (opts.opp && opts.opp.length >= 11) {
      const rep = opponentReport(t, players, opts.opp);
      rep.dangers.forEach(d => ois.push({ p: d.p.name, code: d.code, ois: d.ois, why: L('Um dos três mais perigosos do adversário.', 'One of the opponent’s three most dangerous players.') }));
    } else notes.push(L('Carregue o plantel do adversário para saber a quem aplicar marcação apertada.', 'Load the opponent’s squad to know whom to tight-mark.'));
    notes.push(L('Se aos 60’ estiver empatado e precisar de ganhar, passe para “Precisamos de golos”.', 'If it is level at 60’ and you need the win, switch to “We need goals”.'));
  }

  if (k === 'away_equal') {
    setMent(MI(t.ment) > MI('Balanced') ? 'Balanced' : t.ment === 'Very Defensive' || t.ment === 'Defensive' ? 'Cautious' : t.ment, L('Fora e de igual para igual: equilíbrio.', 'Away and evenly matched: balance.'));
    setTI('loe', 1, L('Bloco médio: nem convidar, nem expor.', 'Mid block: neither invite pressure nor get exposed.'));
    setTI('dline', cbPace >= 13.5 ? 2 : 1, L('Linha ajustada à velocidade dos centrais.', 'The line set to the centre-backs’ pace.'));
    setTI('def_trans', press >= 13 ? 2 : 1, press >= 13 ? L(`Energia para contra-pressionar (${f1(press)}).`, `Energy to counter-press (${f1(press)}).`) : L('Reação equilibrada à perda.', 'A balanced reaction to losing the ball.'));
    setTI('att_trans', attPace >= 13.5 ? 2 : 1, attPace >= 13.5 ? L(`Atacantes rápidos (${f1(attPace)}): aproveitar as transições.`, `Quick attackers (${f1(attPace)}): exploit transitions.`) : L('Transição equilibrada.', 'A balanced transition.'));
    setTI('timewaste', 1, L('Sem queimar tempo cedo.', 'No time-wasting early on.'));
    const fastW = [...grp(['AM_LR']), ...grp(['ST'])].sort((m, n) => q(n.p, ['pac', 'acc']) - q(m.p, ['pac', 'acc']))[0];
    if (fastW && attPace >= 13.5) setRole(fastW, 'OOP', fastW.a.group === 'ST' ? ['socf', 'cf_oop'] : ['wow', 'w_oop'], L('Uma saída rápida para a transição.', 'One quick outlet for transitions.'));
    notes.push(L('Mantenha o plano base e ajuste aos 60’ conforme o resultado: “Precisamos de golos” ou “Segurar o resultado”.', 'Keep the base plan and adjust at 60’ depending on the score: “We need goals” or “Protect the lead”.'));
  }

  if (k === 'home_weak') {
    setMent(stepMent(+1, 'Positive', 'Attacking'), L('Assumir o jogo: o adversário vai fechar-se.', 'Take control: the opponent will sit deep.'));
    setTI('patience', 2, L('Contra bloco baixo, trabalhar a bola até à melhor oportunidade.', 'Against a low block, work the ball until the best chance.'));
    setTI('width', Math.max(cur('width'), 3), L('Esticar o bloco para abrir espaços por dentro.', 'Stretch the block to open space inside.'));
    setTI('reception', 2, L('Não há espaço nas costas: passe ao pé.', 'There is no space in behind: pass to feet.'));
    setTI('def_trans', 2, L('Recuperar logo após a perda, antes que saiam.', 'Win it back straight after losing it, before they break.'));
    if (press >= 12) setTI('trigger', Math.max(cur('trigger'), 3), L(`Pressão constante (${f1(press)}).`, `Constant pressure (${f1(press)}).`));
    setTI('dline', Math.max(cur('dline'), cbPace >= 13 ? 3 : 2), L('Linha subida para empurrar o adversário para trás.', 'A higher line to push the opponent back.'));
    setTI('crossing', stAir >= 14 ? 3 : 0, stAir >= 14 ? L(`Referência aérea (${f1(stAir)}): cruzamentos altos.`, `Aerial target (${f1(stAir)}): high crosses.`) : L('Cruzamentos rasteiros e atrasados para quem chega.', 'Low cut-backs for the late runners.'));
    if (lonTop >= 13) setTI('long_shots', 2, L(`Bloco baixo deixa a entrada da área livre (remate de longe top 3: ${f1(lonTop)}).`, `A low block leaves the edge of the box free (long shots top 3: ${f1(lonTop)}).`));
    if (airTop >= 13.5) setTI('set_pieces', 2, L(`Bolas paradas desbloqueiam jogos fechados (${f1(airTop)}).`, `Set pieces unlock tight games (${f1(airTop)}).`));
    setTI('att_trans', 1, L('Sem necessidade de contra-ataque: o jogo é teu.', 'No need to counter: the game is yours.'));
    // assimetria: um lateral por dentro, o outro por fora
    const fbs = grp(['D_LR', 'WB_LR']);
    if (fbs.length === 2) { const [l, r] = fbs.sort((m, n) => q(n.p, ['pas', 'fir', 'dec']) - q(m.p, ['pas', 'fir', 'dec']));
      setRole(l, 'IP', ['iwb', 'ifb', 'pwb'], L('Entra por dentro para sobrecarregar o meio contra o bloco baixo.', 'Comes inside to overload the middle against the low block.'));
      setRole(r, 'IP', ['wb_ip', 'awb'], L('Dá toda a largura do outro lado.', 'Provides all the width on the other side.')); }
    grp(['AMC']).forEach(x => setRole(x, 'IP', ['ap', 'fr', 'am_ip'], L('Criatividade entre linhas para abrir o bloco.', 'Creativity between the lines to unlock the block.')));
    sts.forEach(x => { if (x.ip && x.ip.tags.includes('DROP') && grp(['AMC']).length) setRole(x, 'IP', ['cf_ip', 'poa'], L('Fica na área: o espaço entre linhas já está ocupado.', 'Stays in the box: the space between the lines is already occupied.')); });
    X().forEach(x => {
      if (!x.p || x.a.group === 'GK') return;
      if (a(x.p, 'lon') >= 15 && x.a.group !== 'DC') addPI(x, 'Shoot more', L(`Remates de Longe ${a(x.p, 'lon')}.`, `Long Shots ${a(x.p, 'lon')}.`));
      if (['AM_LR', 'M_LR'].includes(x.a.group) && a(x.p, 'dri') >= 14) addPI(x, 'Dribble more', L(`Drible ${a(x.p, 'dri')}: desequilibrar contra defesas fechadas.`, `Dribbling ${a(x.p, 'dri')}: unbalance a packed defence.`));
    });
    tweak('Exploit Width', L('Contra bloco baixo, a largura abre o meio.', 'Against a low block, width opens up the middle.'));
  }

  const before = synergyReport(base, players), after = synergyReport(t, players);
  return { sc, t, changes, tweaks, subs, ois, notes, before, after, intentBefore: gameIntent(base), intentAfter: gameIntent(t), auditBefore: auditTactic(base, players), auditAfter: auditTactic(t, players) };
}
