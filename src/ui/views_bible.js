// ============================================================
// Bíblia
// ============================================================
function vRoles() {
  const q = ST.rq.toLowerCase();
  const list = ROLES.filter(r => (!ST.rg || r.groups.includes(ST.rg)) && (!ST.rp || r.phase === ST.rp) &&
    (!q || (r.name + ' ' + r.desc + ' ' + r.tip + ' ' + r.ab).toLowerCase().includes(q)));
  const tagNames = { W:'dá largura', IN:'entra por dentro', RD:'fica atrás', BOX:'ataca a área', CR:'criativo', LINK:'liga jogo', RUN:'profundidade', DROP:'baixa', AIR:'aéreo', ADV:'sobe', TOCB:'vira central', TODM:'vira médio def.', HP:'pressiona alto', LOW:'cauteloso', OUT:'outlet', CW:'cobre flanco', TRK:'acompanha recuo' };
  return `<div class="top"><div><h2>Papéis — com bola e sem bola</h2>
    <p class="muted small">${ROLES.length} papéis. Desde o FM26 não há duties: cada jogador tem um papel IP e um OOP. <span class="tag v">CONF</span> atributos confirmados · <span class="tag e">EST</span> estimados.</p></div></div>
  <div class="panel row" style="margin-bottom:14px">
    <input id="rq" type="text" placeholder="Pesquisar papel…" value="${esc(ST.rq)}" oninput="A.rq(this.value)" style="min-width:220px">
    ${selectHTML([['', 'Todas as posições']].concat(Object.entries(GROUPS)), ST.rg, 'A.rg(this.value)')}
    ${selectHTML([['', 'IP + OOP'], ['IP', 'Com bola (IP)'], ['OOP', 'Sem bola (OOP)']], ST.rp, 'A.rp(this.value)')}
    <span class="small muted">${list.length} resultado(s)</span></div>
  <div class="grid g2">${list.map(r => `<div class="role-card">
    <div class="row" style="justify-content:space-between"><div><b>${esc(r.name)}</b> <span class="muted small">(${esc(r.ab)})</span></div><div><span class="chip ${r.phase === 'IP' ? 'ip' : 'oop'}">${r.phase}</span>${srcTag(r)}</div></div>
    <div class="small muted">${r.groups.map(g => GROUPS[g]).join(' · ')}${r.wideCB ? ' · só central do lado numa linha de 3' : ''}</div>
    <p class="small">${esc(r.desc)}</p><p class="small">💡 ${esc(r.tip)}</p>
    <div class="attrs"><b>Chave:</b> ${r.key.map(k => `<span class="chip">${esc(attrName(k))}</span>`).join('')}</div>
    <div class="attrs"><b>Preferidos:</b> ${r.pref.map(k => `<span class="chip">${esc(attrName(k))}</span>`).join('')}</div>
    ${r.tags.length ? `<div class="attrs muted">Comportamento: ${r.tags.map(x => tagNames[x] || x).join(', ')}</div>` : ''}
    <button class="btn sm" style="margin-top:8px" onclick="A.roleDemo('${r.id}')">▶ Ver em campo</button>
  </div>`).join('')}</div>`;
}

function vTI() {
  return `<div class="top"><div><h2>Instruções</h2></div></div>
  <div class="panel"><h3>Mentalidade</h3><div class="grid g2">${MENTALITIES.map(m => `<div><b>${m}</b><p class="small muted">${esc(MENTALITY_INFO[m])}</p></div>`).join('')}</div></div>
  ${['IP','OOP'].map(ph => `<div class="panel" style="margin-top:16px"><h3>Instruções de equipa — ${ph === 'IP' ? 'com bola' : 'sem bola'}</h3>
    <div class="table-wrap" style="max-height:none"><table><thead><tr><th>Instrução</th><th>Opções</th><th>O que faz</th><th>Prós</th><th>Contras</th></tr></thead><tbody>
    ${TEAM_INSTRUCTIONS.filter(d => d.ph === ph).map(d => `<tr><td><b>${esc(d.n)}</b>${d.fm27 ? ' <span class="chip new">FM27</span>' : ''}<div class="small muted">${esc(d.area)}${d.unconf ? ' · opções a confirmar no jogo' : ''}</div>${SIM_TI.has(d.k) ? `<button class="btn sm" style="margin-top:4px" onclick="A.tiDemo('${d.k}')">▶ Ver em campo</button>` : ''}</td><td class="small">${d.opts.map((o, i) => i === d.def ? `<u>${esc(o)}</u>` : esc(o)).join(' · ')}</td><td class="small">${esc(d.d)}</td><td class="small">${esc(d.pro)}</td><td class="small">${esc(d.con)}</td></tr>`).join('')}
    </tbody></table></div></div>`).join('')}
  <div class="grid g2" style="margin-top:16px">
    <div class="panel"><h3>Instruções individuais</h3>${PLAYER_INSTRUCTIONS.map(p => `<p class="small"><b>${esc(p.n)}</b>${p.fm27 ? ' <span class="chip new">FM27</span>' : ''}<br>${esc(p.d)} <span class="muted">${esc(p.tip)}</span></p>`).join('')}</div>
    <div class="grid">
      <div class="panel"><h3>Instruções de adversário</h3>${OPPOSITION_INSTRUCTIONS.map(p => `<p class="small"><b>${esc(p.n)}</b><br>${esc(p.d)} <span class="muted">${esc(p.tip)}</span></p>`).join('')}</div>
      <div class="panel"><h3>Tactical Tweaks <span class="chip new">FM27</span></h3><p class="small muted">Atalhos no ecrã de jogo que alteram várias instruções de uma vez. Passe o rato no jogo para ver exactamente quais.</p>
        ${TACTICAL_TWEAKS.map(p => `<p class="small"><b>${esc(p.n)}</b> — ${esc(p.d)}<br><span class="muted">Quando: ${esc(p.when)}</span></p>`).join('')}</div>
    </div></div>`;
}

function vConcepts() {
  return `<div class="top"><div><h2>Conceitos, princípios e adversários</h2></div></div>
  <div class="grid g2">${CONCEPTS.map(c => `<div class="panel"><h3>${esc(c.n)}</h3><p class="small">${esc(c.t)}</p></div>`).join('')}</div>
  <div class="panel" style="margin-top:16px"><h3>Contra cada sistema</h3><table><tbody>${OPPONENTS.map(o => `<tr><td style="width:170px"><b>${esc(o.f)}</b></td><td class="small">${esc(o.t)}</td></tr>`).join('')}</tbody></table></div>
  <div class="panel" style="margin-top:16px"><h3>Glossário</h3><table><tbody>${GLOSSARY.map(([a, b]) => `<tr><td style="width:170px"><b>${esc(a)}</b></td><td class="small">${esc(b)}</td></tr>`).join('')}</tbody></table></div>`;
}

function vBTraining() {
  return `<div class="top"><div><h2>Treino & Tactical Effectiveness (FM27)</h2></div><button class="btn" onclick="A.go('training')">Gerar plano para a minha tática →</button></div>
  <div class="grid g2">${TRAINING.effectiveness.map(e => `<div class="panel"><h3>${esc(e.n)}</h3><p class="small">${esc(e.d)}</p><p class="small"><b>Como melhorar:</b> ${esc(e.fix)}</p></div>`).join('')}</div>
  <div class="grid g2" style="margin-top:16px">
    <div class="panel"><h3>Princípios</h3><ul class="clean small">${TRAINING.principles.map(p => `<li>${esc(p)}</li>`).join('')}</ul></div>
    <div class="panel"><h3>Periodização</h3><table><tbody>${TRAINING.periodization.map(p => `<tr><td><b>${esc(p.p)}</b></td><td class="small">${esc(p.f)}<br><span class="muted">${esc(p.n)}</span></td></tr>`).join('')}</tbody></table>
      <h4 style="margin-top:12px">Áreas de Tactical Understanding</h4><p class="small">${Object.values(TRAINING.areas).map(a => `<span class="chip">${esc(a.n)}</span>`).join('')}</p></div>
  </div>`;
}

