// ============================================================
// Vistas da equipa: plantel, XI & sugestões, adversário
// ============================================================
function vSquad() {
  const ps = players();
  return `<div class="top"><div><h2>Plantel${ST.myClub ? ' — ' + esc(ST.myClub) : ''}</h2><p class="muted small" style="margin:0">${ps.length ? `${ps.length} jogadores · ${esc(ST.squad.source)}${ST.squad.at ? ' · ' + new Date(ST.squad.at).toLocaleString('pt-PT') : ''}` : 'Ainda sem plantel carregado.'}</p></div>
    <div class="row"><button class="btn" onclick="A.sampleSquad()">Usar exemplo</button>${ps.length ? '<button class="btn" onclick="A.clearSquad()">Limpar</button>' : ''}</div></div>
  <div class="grid g2">
    <div class="panel"><h3>Como trazer o plantel do FM27</h3>
      ${DESKTOP ? `<div class="notice"><b>Do save (recomendado):</b> grave o jogo no FM — a app lê o save sozinha${ST.auto.save ? '' : ' (automático desligado)'}.
        ${ST.lastSave ? `<br><span class="small">Último save: <b>${esc(ST.lastSave.name)}</b>${ST.auto.saveLast ? ' · lido ' + esc(ST.auto.saveLast.split(' · ')[1] || '') : ''}</span>` : ''}
        ${ST.auto.saveErr ? `<div class="small" style="color:var(--crit);margin-top:4px">${esc(ST.auto.saveErr)}</div><div class="small">Instale o leitor uma vez: <code>winget install --id=astral-sh.uv -e</code> (depois reinicie a app).</div>` : ''}
        <div class="row" style="margin-top:8px"><button class="btn primary sm" onclick="A.saveNow()">Ler o save agora</button></div></div>`
      : `<ol class="clean small"><li><b>Do save:</b> abra o <a href="https://rhiever.github.io/fmsave/app/" target="_blank" rel="noopener">fmsave no browser</a>, largue lá o seu <code>.fm</code> (Documentos › Sports Interactive › Football Manager 27 › games) e descarregue o CSV do plantel. Nada é enviado para a internet.</li><li>Arraste esse CSV para esta janela.</li></ol>`}
      <details style="margin-top:8px"><summary>Alternativa: F9 no jogo (FM View Exporter)</summary>
      <ol class="clean small">
        <li>Instale o plugin gratuito <b>FM View Exporter</b> (github.com/ElectroHugin/FM-View-Exporter → Releases → <code>setup.exe</code>).</li>
        <li>No ecrã do plantel: <b>Import View…</b> → <code>FM27-PlayerAnalyzer-Squad-Standard.fmf</code>, depois <b>F9</b>. ${DESKTOP ? 'A app apanha o ficheiro sozinha.' : 'Arraste o <code>squad_….csv</code> para aqui.'}</li>
        <li>F9 no plantel de outro clube carrega-o como <b>adversário</b>.</li>
      </ol></details>
      <p class="small muted">Também aceita exportações do fmsave (CSV/JSON), FMPT Chief Analyst, FMLibreEditor, HTML com tabela e texto colado.</p>
    </div>
    <div class="panel"><textarea id="sqtext" placeholder="Arraste para aqui o CSV do plantel (fmsave ou F9 do FM View Exporter), ou cole o conteúdo…" ondragover="event.preventDefault()" ondrop="event.preventDefault();A.squadFile({files:event.dataTransfer.files})"></textarea>
      <div class="row" style="margin-top:8px"><button class="btn primary" onclick="A.squadText()">Ler texto</button>
      <label class="btn">Escolher ficheiro<input type="file" accept=".html,.htm,.txt,.csv,.tsv,.json" hidden onchange="A.squadFile(this)"></label>
      <label class="btn">Carregar como adversário<input type="file" accept=".html,.htm,.txt,.csv,.tsv" hidden onchange="A.squadFile(this,'opp')"></label></div>
      ${(ST.squad.warnings || []).map(w => `<div class="notice warn">${esc(w)}</div>`).join('')}
      ${ST.squad.mapped && ST.squad.mapped.length ? `<details><summary>Colunas reconhecidas (${ST.squad.mapped.filter(m => m.k).length}/${ST.squad.mapped.length})</summary><p class="small">${ST.squad.mapped.map(m => `<span class="chip" style="${m.k ? '' : 'opacity:.5'}">${esc(m.h)}${m.k ? ' → ' + esc(m.k === 'name' || m.k === 'position' || m.k === 'age' ? m.k : attrName(m.k)) : ''}</span>`).join('')}</p></details>` : ''}
    </div>
  </div>
  ${ps.length ? `<div class="panel" style="margin-top:16px"><div class="row" style="justify-content:space-between"><h3>Melhores papéis de cada jogador</h3><span class="small muted">Melhores papéis por pontuação de atributos (0–100). Posições naturais são respeitadas.</span></div>
    <div class="table-wrap"><table><thead><tr><th>Jogador</th><th>Pos</th><th>Idade</th>${ps.some(p => p.meta && p.meta.ability) ? '<th>Adjunto (CA/PA ★)</th>' : ''}<th>Melhores papéis com bola</th><th>Melhores papéis sem bola</th></tr></thead><tbody>
    ${ps.map(p => {
      const groups = p.pos.length ? [...new Set(p.pos.filter(x => !x.includes(':')))] : Object.keys(GROUPS);
      const best = (phase) => groups.flatMap(g => rolesFor(g, phase, g === 'DC')).filter((r, i, a) => a.indexOf(r) === i)
        .map(r => ({ r, s: (roleScore(p, r) || { score: 0 }).score })).sort((x, y) => y.s - x.s).slice(0, 3);
      const adj = ps.some(q => q.meta && q.meta.ability) ? `<td class="small">${esc((p.meta && p.meta.ability) || '—')} / ${esc((p.meta && p.meta.potential) || '—')}${p.meta && p.meta.bestRole ? `<br><span class="muted">${esc(p.meta.bestRole)}</span>` : ''}</td>` : '';
      return `<tr><td><b>${esc(p.name)}</b></td><td class="small">${esc(p.posRaw || '—')}</td><td>${p.age ?? '—'}</td>${adj}
        <td>${best('IP').map(x => `${fitBadge(x.s)} <span class="small">${esc(x.r.name)}</span>`).join('<br>')}</td>
        <td>${best('OOP').map(x => `${fitBadge(x.s)} <span class="small">${esc(x.r.name)}</span>`).join('<br>')}</td></tr>`;
    }).join('')}</tbody></table></div></div>` : ''}`;
}

function vSuggest() {
  const ps = players();
  if (ps.length < 11) return `<h3 style="margin-top:20px">Que sistema serve este plantel?</h3>
    <div class="panel" style="margin-top:16px"><h3>Modelos disponíveis</h3><div class="grid g2">${TEMPLATES.map(tp => `<div class="role-card"><b>${esc(tp.n)}</b><p class="small muted">${esc(tp.ip)} → ${esc(tp.oop)} · ${esc(tp.ment)}</p><p class="small">${esc(tp.desc)}</p><button class="btn sm" onclick="A.useTemplate('${tp.id}')">Abrir no analisador</button></div>`).join('')}</div></div>`;
  if (!ST.sugg) ST.sugg = suggestTactics(ps);
  return `<div class="top" style="margin-top:22px"><div><h2>Sistemas ordenados para o teu plantel</h2>
    <p class="muted small">Pontuação = adequação do XI (55%) + perfil do plantel ao estilo (20%) + profundidade (15%) + coerência da tática (10%).</p></div>
    <button class="btn" onclick="A.runSuggest()">Recalcular</button></div>
  <div class="grid">${ST.sugg.map((x, i) => `<div class="panel"><div class="grid" style="grid-template-columns:200px 1fr;gap:16px">
      <div>${pitchSVG(x.t, 'IP', null, { static: true })}</div>
      <div><div class="row" style="justify-content:space-between"><div><span class="eyebrow">#${i + 1}</span><h3>${esc(x.tp.n)}</h3><div class="small muted">${esc(x.tp.ip)} → ${esc(x.tp.oop)} · ${esc(x.tp.ment)}</div></div>${ring(x.total, 72)}</div>
      <p class="small">${esc(x.tp.desc)}</p>
      <div class="bars" style="max-width:420px">
        ${[['XI', x.xiAvg], ['Estilo', x.fit], ['Profundidade', x.depth], ['Coerência', x.audit.overall]].map(([n, v]) => `<div class="bar"><span>${n}</span><div class="t"><i style="width:${v}%;background:${fitColor(v)}"></i></div><b>${v}</b></div>`).join('')}
      </div>
      <p class="small" style="margin-top:8px"><b>Pontos fortes:</b> ${x.strongest.map(y => `${esc(y.p ? y.p.name : '?')} como ${esc(ROLE_BY_ID[y.s.ip].name)} (${y.sc})`).join(' · ')}</p>
      <p class="small"><b>Pontos fracos:</b> ${x.weakest.map(y => `${esc(y.p ? y.p.name : '?')} em ${esc(y.s.ipCode)} (${y.sc})`).join(' · ')} — alvo de recrutamento/treino.</p>
      <button class="btn primary sm" onclick="A.useSuggestion(${i})">Abrir no analisador</button></div></div></div>`).join('')}</div>`;
}


// ---------- Melhor XI: adequação, instruções recomendadas, suplentes, sucessores ----------
function vXI() {
  const ps = players(), t = ST.tactic;
  if (ps.length < 11) return `<div class="panel empty"><h3>Carregue o plantel primeiro</h3><p>Com o plantel, a app escolhe o melhor XI para a tua tática, recomenda instruções individuais pelos atributos e mostra suplentes e sucessores para cada posição.</p>
    <button class="btn primary" onclick="A.tab('team','squad')">Ir para Plantel</button> <button class="btn" onclick="A.sampleSquad();A.tab('team','xi')">Usar exemplo</button></div>`;
  const pm = Object.fromEntries(ps.map(p => [p.id, p]));
  const bs = benchAndSuccessors(t, ps);
  const rows = t.slots.map((s, i) => {
    const a = slotInfo(t.ipForm, s.ipCode), p = pm[s.pid], ri = ROLE_BY_ID[s.ip], ro = ROLE_BY_ID[s.oop];
    const sc = p ? slotScore(p, t, s) : null;
    const alt = p ? bestRolesForPlayer(p, a.group, 'IP', a.wideCB, 1)[0] : null;
    const cur = p ? (roleScore(p, ri) || { score: 0 }).score : 0;
    const recs = p ? recommendPIs(p, ri, ro, a.group) : [];
    const b = bs[i];
    return `<tr><td><b>${esc(s.ipCode)}</b></td>
      <td>${p ? `${fitBadge(sc)} <b>${esc(p.name)}</b>${p.age ? ` <span class="muted small">${p.age}a</span>` : ''}` : '<span class="muted">vazio</span>'}
        <div class="small muted">${esc(ri.name)} / ${esc(ro.name)}${alt && alt.r.id !== ri.id && alt.s >= cur + 5 ? ` · <span style="color:var(--warn)">melhor como ${esc(alt.r.name)} (${alt.s})</span>` : ''}</div></td>
      <td>${recs.map(r => (s.pis || []).includes(r.pi) ? `<span class="chip ip" title="${esc(r.why)}">✓ ${esc(r.pi)}</span>` : `<button class="chip btnchip" title="${esc(r.why)}" onclick="A.applyPI(${i}, '${r.pi.replace(/'/g, "\\'")}')">+ ${esc(r.pi)}</button>`).join('') || '<span class="muted small">—</span>'}</td>
      <td class="small">${b.bench ? `${esc(b.bench.p.name)} ${fitBadge(b.bench.sc)}` : '—'}</td>
      <td class="small">${b.succ ? `${esc(b.succ.p.name)} <span class="muted">${b.succ.p.age}a</span> ${fitBadge(b.succ.sc)}` : '—'}</td></tr>`;
  }).join('');
  return `<div class="top"><div><h2>Melhor XI — ${esc(t.name)}</h2><p class="muted small" style="margin:0">Passe o rato numa instrução para ver porquê. As instruções aplicadas entram na tática e na Ficha para o FM27.</p></div>
    <div class="row"><button class="btn" onclick="A.autoXI()">Escolher XI automaticamente</button><button class="btn primary" onclick="A.applyAllPIs()">Aplicar a principal de cada jogador</button></div></div>
  <div class="grid" style="grid-template-columns:minmax(220px,300px) 1fr;gap:16px;align-items:start">
    <div class="panel">${pitchSVG(t, 'IP', null, { static: true })}<p class="small muted">Cor = adequação ao par de papéis (verde ≥ 70).</p></div>
    <div class="table-wrap" style="max-height:none"><table><thead><tr><th>Pos</th><th>Jogador e papéis</th><th>Instruções recomendadas</th><th>Suplente</th><th>Sucessor (≤ 21)</th></tr></thead><tbody>${rows}</tbody></table></div>
  </div>
  ${vSuggest()}`;
}

// ---------- Relatório do adversário ----------
function vOpp() {
  const op = oppPlayers();
  if (!op.length) return `<div class="grid g2"><div class="panel"><h2>Relatório do adversário</h2>
      <p>Carrega o plantel do próximo adversário e a app compara setor a setor com o teu XI, diz quem são os perigosos e propõe ajustes à tua tática.</p>
      <ol class="clean small"><li>No FM27, abre o plantel do adversário (com a mesma vista de atributos).</li><li>Carrega em <b>F9</b>. ${DESKTOP && ST.auto.on ? 'A app reconhece que é outro clube e carrega-o aqui sozinha.' : 'Arrasta o CSV para esta janela — se o clube for diferente do teu, entra como adversário.'}</li></ol>
      <label class="btn primary">Escolher ficheiro do adversário<input type="file" accept=".csv,.html,.htm,.txt,.tsv" hidden onchange="A.squadFile(this,'opp')"></label></div>
      <div class="panel"><h3>O que vais ver</h3><ul class="clean small"><li>Velocidade e jogo aéreo dos centrais deles contra os teus avançados.</li><li>Laterais, criação no meio-campo, pressão, extremos, finalização e ataque aéreo.</li><li>Os 3 jogadores mais perigosos e as instruções de adversário a usar.</li><li>Um plano de ajustes que aplicas à tua tática com um clique e vês no campo animado.</li></ul></div></div>`;
  const rep = opponentReport(ST.tactic, players(), op); ST._oppRep = rep;
  const pct = (v) => v == null ? 0 : Math.max(0, Math.min(100, (v - 1) / 19 * 100));
  return `<div class="top"><div><h2>Relatório: ${esc(rep.club || ST.opp.source)}</h2><p class="muted small" style="margin:0">${op.length} jogadores · onze provável escolhido pelos atributos (4-2-3-1). ${players().length ? 'O traço vertical é o teu XI.' : 'Carrega o teu plantel para comparar.'}</p></div>
    <div class="row"><button class="btn" onclick="A.clearOpp()">Trocar adversário</button></div></div>
  <div class="grid g2">
    <div class="panel"><h3>Setor a setor</h3>
      ${rep.sectors.map(s => `<div class="oppbar"><span>${esc(s.n)}</span><div class="t"><i style="width:${pct(s.tv)}%;background:${s.tv >= 14 ? 'var(--crit)' : s.tv <= 10.5 ? 'var(--good)' : 'var(--warn)'}"></i>${s.uv != null ? `<em style="left:${pct(s.uv)}%" title="Teu XI: ${s.uv.toFixed(1)}"></em>` : ''}</div><b>${s.tv != null ? s.tv.toFixed(1) : '—'}</b></div>`).join('')}
      <p class="small muted">Vermelho = ponto forte deles · verde = ponto fraco deles.</p></div>
    <div class="panel"><div class="row" style="justify-content:space-between"><h3>Plano de jogo</h3>${rep.plan.length ? `<div class="row"><button class="btn primary sm" onclick="A.applyPlan('all')">Aplicar tudo</button><button class="btn sm" onclick="A.simOpen()">▶ Ver em campo</button></div>` : ''}</div>
      ${rep.plan.length ? rep.plan.map((x, i) => { const d = TI_BY_KEY[x.ti], on = tiVal(ST.tactic, x.ti) === x.v;
        return `<div class="finding"><div class="sev ${on ? 'good' : 'info'}">${on ? '✓' : i + 1}</div><div><b>${esc(d.n)} → ${esc(d.opts[x.v])}</b><span class="small">${esc(x.why)}</span>${on ? '' : `<div><button class="btn sm" style="margin-top:4px" onclick="A.applyPlan(${i})">Aplicar</button></div>`}</div></div>`; }).join('')
      : '<p class="muted">Sem diferenças marcadas entre as equipas: mantém o teu plano.</p>'}
      <h3 style="margin-top:14px">Mais perigosos</h3>
      ${rep.dangers.map(d => `<div class="finding"><div class="sev crit">!</div><div><b>${esc(d.p.name)}</b> <span class="muted small">${esc(d.code)}</span><div class="small">Instruções de adversário: ${d.ois.map(o => `<span class="chip">${esc(o)}</span>`).join('')}</div></div></div>`).join('')}
    </div>
  </div>
  <div class="panel" style="margin-top:16px"><h3>Onze provável</h3><p class="small">${rep.them.map(x => `<span class="chip"><b>${esc(x.code)}</b> ${esc(x.p.name)}</span>`).join('')}</p></div>`;
}
