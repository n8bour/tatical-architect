// ============================================================
// Início: fluxo guiado em 3 passos + o que a app viu
// ============================================================
function vHome() {
  const t = ST.tactic, ps = players(), op = oppPlayers(), au = auditTactic(t, ps);
  const top = au.findings.filter(f => f.sev === 'crit' || f.sev === 'warn').slice(0, 3);
  const step = (n, done, title, body, cta) => `<div class="panel step-card ${done ? 'done' : ''}"><div class="row" style="gap:10px"><div class="n">${done ? '✓' : n}</div><h3 style="margin:0">${title}</h3></div><div class="small" style="margin:8px 0 12px">${body}</div><div class="row">${cta}</div></div>`;
  let sugg = '';
  if (ps.length >= 11) { if (!ST.sugg) ST.sugg = suggestTactics(ps); const s0 = ST.sugg[0]; sugg = `<div class="panel"><h3>Sistema que melhor serve o plantel</h3><p><b>${esc(s0.tp.n)}</b> — pontuação ${s0.total} (XI ${s0.xiAvg}, estilo ${s0.fit}, profundidade ${s0.depth}).</p><div class="row"><button class="btn" onclick="A.tab('team','xi')">Ver ranking completo</button><button class="btn" onclick="A.useSuggestion(0)">Abrir no analisador</button></div></div>`; }
  let oppBox = '';
  if (op.length) { const rep = opponentReport(t, ps, op); ST._oppRep = rep; oppBox = `<div class="panel"><h3>Próximo adversário: ${esc(rep.club || '')}</h3>${rep.plan.slice(0, 2).map(x => `<p class="small">• <b>${esc(TI_BY_KEY[x.ti].n)} → ${esc(TI_BY_KEY[x.ti].opts[x.v])}</b>: ${esc(x.why)}</p>`).join('') || '<p class="small muted">Sem ajustes necessários.</p>'}<button class="btn" onclick="A.tab('team','opp')">Ver relatório</button></div>`; }
  return `<div class="top"><div><h1>Olá! Vamos preparar o próximo jogo.</h1><p class="muted">Três passos. Tudo corre neste computador${DESKTOP ? ' e o plantel actualiza sozinho quando carrega F9 no FM' : ''}.</p></div></div>
  <div class="grid g3">
    ${step(1, ps.length > 0, 'Plantel', ps.length ? `<b>${ps.length} jogadores</b>${ST.myClub ? ' do ' + esc(ST.myClub) : ''}<br><span class="muted">${esc(ST.squad.source)}</span>` : (DESKTOP ? (ST.auto.saveErr ? 'Falta instalar o leitor de saves — veja Definições.' : 'Grave o jogo no FM27: a app lê o save sozinha.') : 'Leia o save com o fmsave online e arraste o CSV para aqui — ou experimente com o exemplo.'),
      ps.length ? `<button class="btn" onclick="A.tab('team','xi')">Ver melhor XI</button>` : (DESKTOP ? `<button class="btn primary" onclick="A.saveNow()">Ler o save agora</button>` : `<button class="btn primary" onclick="A.tab('team','squad')">Como carregar</button>`) + `<button class="btn" onclick="A.sampleSquad()">Usar exemplo</button>`)}
    ${step(2, true, 'Tática', `<div class="score-ring" style="gap:12px">${ring(au.overall, 64)}<div><b>${esc(t.name)}</b><br><span class="muted">${esc(t.ipForm)} → ${esc(t.oopForm)} · ${esc(t.ment)}</span></div></div>`,
      `<button class="btn primary" onclick="A.tab('tactic','analyze')">Analisar</button><button class="btn" onclick="A.simOpen()">▶ Ver em campo</button><button class="btn" onclick="A.tab('tactic','sheet')">Ficha FM27</button>`)}
    ${step(3, op.length > 0, 'Adversário', op.length ? `<b>${esc(ST._oppRep && ST._oppRep.club || ST.opp.source)}</b> carregado` : 'No FM27, abra o plantel do adversário e carregue <b>F9</b>. A app reconhece que é outro clube.',
      `<button class="btn ${op.length ? '' : 'primary'}" onclick="A.tab('team','opp')">${op.length ? 'Ver relatório' : 'Como carregar'}</button>`)}
  </div>
  <div class="grid g2" style="margin-top:16px">
    <div class="panel"><h3>O que corrigir primeiro</h3>${top.length ? top.map(f => `<div class="finding"><div class="sev ${f.sev}">${f.sev === 'crit' ? '!' : '▲'}</div><div><b>${esc(f.title)}</b><span class="small">${esc(f.text)}</span>${f.fix ? `<div class="fix">→ ${esc(f.fix)}</div>` : ''}</div></div>`).join('') : '<p class="muted">Nada grave na tática. 👍</p>'}
      <button class="btn" onclick="A.tab('tactic','analyze')">Ver diagnóstico completo</button></div>
    <div class="grid">${sugg}${oppBox}${!sugg && !oppBox ? `<div class="panel"><h3>Bíblia tática</h3><p class="small muted">${ROLES.length} papéis com animação, ${TEAM_INSTRUCTIONS.length} instruções, conceitos e treino FM27.</p><button class="btn" onclick="A.go('bible')">Abrir</button></div>` : ''}</div>
  </div>
  ${ST.saved.length ? `<div class="panel" style="margin-top:16px"><h3>Táticas guardadas</h3><div class="row">${ST.saved.slice(0, 8).map((s, i) => `<button class="btn sm" onclick="A.loadSaved(${i})">${esc(s.name)}</button>`).join('')}</div></div>` : ''}`;
}
