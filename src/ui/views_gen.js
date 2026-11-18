// ============================================================
// Gerador de tática — relatório na ordem dos ecrãs do FM27
// ============================================================
function genOpts() { if (!ST.gen) ST.gen = { expect: 'mid', must: [], star: '', youth: [], res: null, pick: 0 }; return ST.gen; }
function vGen() {
  const ps = players(), g = genOpts();
  if (ps.length < 11) return `<div class="panel empty"><h3>Carregue o plantel primeiro</h3><p>O gerador monta uma tática completa à volta do teu plantel: formas com e sem bola, XI com papel para cada fase, as instruções todas com o porquê, bolas paradas, profundidade, reforços e alternativas.</p>
    <div class="row" style="justify-content:center">${dataButtons()}<button class="btn" onclick="A.sampleSquad();A.tab('team','gen')">Usar exemplo</button></div></div>`;
  const byName = [...ps].sort((a, b) => a.name.localeCompare(b.name));
  const young = byName.filter(p => p.age != null && p.age <= 21);
  const opts = `<div class="panel grid"><h3>O teu contexto</h3>
    <label class="f">Objectivo na liga${selectHTML(Object.entries(EXPECT).map(([k, e]) => [k, e.n]), g.expect, "A.genOpt('expect',this.value)")}</label>
    <label class="f">Jogador-estrela (constrói à volta dele)${selectHTML([['', '— nenhum —']].concat(byName.map(p => [p.id, p.name + (p.posRaw ? ' · ' + p.posRaw : '')])), g.star, "A.genOpt('star',this.value)")}</label>
    <div class="f"><span>Titulares obrigatórios</span><div class="chips">${byName.map(p => `<button class="chip pick ${g.must.includes(p.id) ? 'on' : ''}" onclick="A.genToggle('must','${esc(p.id)}')">${esc(p.name)}</button>`).join('')}</div></div>
    ${young.length ? `<div class="f"><span>Jovens a desenvolver (≤ 21) — ganham prioridade no XI</span><div class="chips">${young.map(p => `<button class="chip pick ${g.youth.includes(p.id) ? 'on' : ''}" onclick="A.genToggle('youth','${esc(p.id)}')">${esc(p.name)} <span class="muted">${p.age}</span></button>`).join('')}</div></div>` : ''}
    <button class="btn primary" data-busy="gen" onclick="A.genRun()">${g.res ? 'Gerar de novo' : 'Gerar tática'}</button>
    <p class="small muted" style="margin:0">Testa todos os pares de formações, escolhe o papel de cada posição com regras de equilíbrio (rest-defence, largura, área, criadores, outlets) e o XI exacto pelo algoritmo húngaro.</p></div>`;
  if (!g.res) return `<div class="top"><div><h2>Gerar tática</h2><p class="muted small" style="margin:0">Uma tática completa feita à volta do teu plantel, pronta a copiar para o FM27.</p></div></div><div class="grid" style="grid-template-columns:minmax(280px,380px) 1fr;gap:16px;align-items:start">${opts}<div class="panel empty">Escolha o contexto e carregue em <b>Gerar tática</b>.</div></div>`;
  const all = [g.res.main, ...g.res.alts], r = all[g.pick] || all[0], t = r.t;
  const pm = Object.fromEntries(ps.map(p => [p.id, p]));
  const sp = setPiecePlan(t, ps, oppPlayers()), tp = trainingPlan(t, ps, r.audit), dc = depthChart(t, ps), weak = weakSpots(t, ps);
  const pis = t.slots.map(s => { const a = slotInfo(t.ipForm, s.ipCode); const rec = recommendPIs(pm[s.pid], ROLE_BY_ID[s.ip], ROLE_BY_ID[s.oop], a.group).slice(0, 2); return { s, p: pm[s.pid], rec }; }).filter(x => x.rec.length);
  const sec = (n, title, body) => `<div class="panel"><h3><span class="secn">${n}</span>${title}</h3>${body}</div>`;
  return `<div class="top"><div><h2>Gerar tática</h2><p class="muted small" style="margin:0">${g.res.considered} pares de formações avaliados, ${g.res.optimised} optimizados a fundo.</p></div>
    <div class="row"><button class="btn primary" onclick="A.genUse()">Usar esta tática</button><button class="btn" onclick="A.genUse(true)">▶ Ver em campo</button><button class="btn" onclick="A.genCopy()">Copiar como texto</button></div></div>
  <div class="grid" style="grid-template-columns:minmax(280px,380px) 1fr;gap:16px;align-items:start">
    <div class="grid">${opts}
      <div class="panel"><h3>Opções geradas</h3>${all.map((x, i) => `<button class="genopt ${i === g.pick ? 'on' : ''}" onclick="A.genPick(${i})"><b>${i === 0 ? 'Recomendada' : 'Alternativa ' + i}</b><span>${esc(x.t.ipForm)} → ${esc(x.t.oopForm)}</span><span class="small muted">${L(`adequação ${x.fitAvg} · sinergia ${x.syn ?? '—'} · diagnóstico ${x.audit.overall}`, `fit ${x.fitAvg} · synergy ${x.syn ?? '—'} · diagnosis ${x.audit.overall}`)}</span>${ring(x.total, 46)}</button>`).join('')}</div>
    </div>
    <div class="grid">
      ${sec(1, 'Estilo e mentalidade', `<p><b>Style:</b> Clean Slate · <b>Mentality:</b> ${esc(t.ment)}</p><p class="small muted">${esc(r.mentWhy)}</p>`)}
      ${sec(2, 'Formas e XI', `<div class="grid g2"><div><p class="small"><b>Com bola:</b> ${esc(t.ipForm)}</p>${pitchSVG(t, 'IP', null, { static: true })}</div><div><p class="small"><b>Sem bola:</b> ${esc(t.oopForm)}</p>${pitchSVG(t, 'OOP', null, { static: true })}</div></div>
        <div class="table-wrap" style="margin-top:10px;max-height:none"><table><thead><tr><th>Pos</th><th>Jogador</th><th>In Possession</th><th>Out of Possession</th><th>Adequação</th></tr></thead><tbody>
        ${t.slots.map(s => { const p = pm[s.pid]; return `<tr><td><b>${esc(s.ipCode)}</b>${s.oopCode !== s.ipCode ? ` <span class="muted small">→ ${esc(s.oopCode)}</span>` : ''}</td><td>${p ? esc(p.name) : '—'}${p && g.youth.includes(p.id) ? ' <span class="chip new">jovem</span>' : ''}${p && g.star === p.id ? ' <span class="chip new">estrela</span>' : ''}</td><td>${esc(roleName(ROLE_BY_ID[s.ip]))}</td><td>${esc(roleName(ROLE_BY_ID[s.oop]))}</td><td>${fitBadge(p ? slotScore(p, t, s) : null)}</td></tr>`; }).join('')}
        </tbody></table></div>`)}
      ${sec(3, L('Sinergia entre papéis — todas as fases', 'Role synergy — every phase'), synergyPanel(t, ps, { labHint: true }))}
      ${sec(4, 'Instruções de equipa', FM_SECTIONS.map(([n, keys]) => `<h4 style="margin:10px 0 4px">${esc(n)}</h4><table><tbody>${keys.map(k => { const d = TI_BY_KEY[k], v = tiVal(t, k), ch = v !== d.def;
          return `<tr class="${ch ? '' : 'dim'}"><td style="width:34%">${esc(d.n)}</td><td style="width:22%">${ch ? `<b>${esc(d.opts[v])}</b>` : esc(d.opts[v])}</td><td class="small muted">${esc(r.reasons[k] || '')}</td></tr>`; }).join('')}</tbody></table>`).join(''))}
      ${sec(5, 'Instruções individuais', pis.length ? `<table><tbody>${pis.map(x => `<tr><td><b>${esc(x.s.ipCode)}</b> ${esc(x.p.name)}</td><td>${x.rec.map(r2 => `<span class="chip" title="${esc(r2.why)}">${esc(r2.pi)}</span>`).join('')}</td><td class="small muted">${esc(x.rec[0].why)}</td></tr>`).join('')}</tbody></table>` : '<p class="muted small">Nenhum atributo pede instruções individuais.</p>')}
      ${sp ? sec(6, 'Bolas paradas e capitães', `<div class="grid g2"><table><tbody>
          <tr><td>Capitão</td><td><b>${esc(sp.caps[0].p.name)}</b></td></tr><tr><td>Vice-capitão</td><td><b>${esc(sp.caps[1].p.name)}</b></td></tr>
          <tr><td>Cantos (dir./esq.)</td><td><b>${esc(sp.corners.right.x.p.name)}</b> / <b>${esc(sp.corners.left.x.p.name)}</b></td></tr>
          <tr><td>Livres</td><td><b>${esc((sp.fk.direct || sp.fk.cross).p.name)}</b>${sp.fk.direct ? ' (directos)' : ' (cruzar)'}</td></tr>
          <tr><td>Penáltis</td><td>${sp.pens.map(x => esc(x.p.name)).join(' → ')}</td></tr></tbody></table>
          <table><tbody>${sp.attack.filter(x => ['far', 'centre', 'near'].includes(x.k)).map(x => `<tr><td><span>Alvo aéreo</span> — ${esc(x.n)}</td><td><b>${esc(x.x.p.name)}</b></td></tr>`).join('')}</tbody></table></div>
          <button class="btn sm" onclick="A.genUse(false,'setpieces')">Ver diagramas dos cantos</button>`) : ''}
      ${sec(7, 'Profundidade por posição', `<div class="depth">${dc.map(d => `<div><b>${esc(d.code)}</b>${d.list.map((x, i) => `<div class="small">${i + 1}. ${esc(x.p.name)} ${fitBadge(x.sc)}</div>`).join('')}</div>`).join('')}</div>`)}
      ${sec(8, 'Onde reforçar', weak.length ? `<table><tbody>${weak.map(w => `<tr><td><b>${esc(w.code)}</b> · ${esc(roleName(w.role))}</td><td>titular ${fitBadge(w.sc)} · alternativa ${fitBadge(w.alt)}</td><td class="small muted">${w.lack.length ? 'procurar: ' + w.lack.map(attrName).join(', ') : ''}</td></tr>`).join('')}</tbody></table>` : '<p class="small">Todas as posições têm titular ≥ 55 e alternativa ≥ 45. 👍</p>')}
      ${sec(9, 'Treino', `<ol class="clean small">${tp.areas.slice(0, 4).map(a => `<li><b>${esc(TRAINING.areas[a.k].n)}</b> — ${esc([...new Set(a.why)][0])}</li>`).join('')}</ol><button class="btn sm" onclick="A.genUse(false,'training')">Plano de treino completo</button>`)}
    </div>
  </div>`;
}
