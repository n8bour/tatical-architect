// ============================================================
// Bíblia
// ============================================================
function vRoles() {
  const q = ST.rq.toLowerCase();
  const list = ROLES.filter(r => (!ST.rg || r.groups.includes(ST.rg)) && (!ST.rp || r.phase === ST.rp) &&
    (!q || [r.name, PT_TERMS[r.name] || '', PT_ROLE_ID[r.id] || '', r.desc, r.tip, r.ab, T(r.desc), T(r.tip)].join(' ').toLowerCase().includes(q)));
  const tagNames = { W:'dá largura', IN:'entra por dentro', RD:'fica atrás', BOX:'ataca a área', CR:'criativo', LINK:'liga jogo', RUN:'profundidade', DROP:'baixa', AIR:'aéreo', ADV:'sobe', TOCB:'vira central', TODM:'vira médio def.', HP:'pressiona alto', LOW:'cauteloso', OUT:'outlet', CW:'cobre flanco', TRK:'acompanha recuo' };
  return `<div class="top"><div><h2>Papéis — com bola e sem bola</h2>
    <p class="muted small">${ROLES.length} papéis. Desde o FM26 não há duties: cada jogador tem um papel IP e um OOP. <span class="tag v">CONF</span> atributos confirmados · <span class="tag e">EST</span> estimados.</p></div></div>
  <div class="panel row" style="margin-bottom:14px">
    <input id="rq" type="text" placeholder="Pesquisar papel…" value="${esc(ST.rq)}" oninput="A.rq(this.value)" style="min-width:220px">
    ${selectHTML([['', 'Todas as posições']].concat(Object.entries(GROUPS)), ST.rg, 'A.rg(this.value)')}
    ${selectHTML([['', 'IP + OOP'], ['IP', 'Com bola (IP)'], ['OOP', 'Sem bola (OOP)']], ST.rp, 'A.rp(this.value)')}
    <span class="small muted">${list.length} resultado(s)</span></div>
  <div class="grid g2">${list.map(r => `<div class="role-card">
    <div class="row" style="justify-content:space-between"><div><b>${esc(roleName(r))}</b> <span class="muted small">(${esc(r.ab)})</span></div><div><span class="chip ${r.phase === 'IP' ? 'ip' : 'oop'}">${r.phase}</span>${srcTag(r)}</div></div>
    <div class="small muted">${r.groups.map(g => `<span>${GROUPS[g]}</span>`).join(' · ')}${r.wideCB ? ` · <span>só central do lado numa linha de 3</span>` : ''}</div>
    <p class="small">${esc(r.desc)}</p><p class="small">💡 ${esc(r.tip)}</p>
    <div class="attrs"><b>Chave:</b> ${r.key.map(k => `<span class="chip">${esc(attrName(k))}</span>`).join('')}</div>
    <div class="attrs"><b>Preferidos:</b> ${r.pref.map(k => `<span class="chip">${esc(attrName(k))}</span>`).join('')}</div>
    ${r.tags.length ? `<div class="attrs muted"><span>Comportamento:</span> ${r.tags.map(x => `<span>${tagNames[x] || x}</span>`).join(', ')}</div>` : ''}
    <div class="row" style="margin-top:8px"><button class="btn sm" onclick="A.clipToggle('role:${r.id}')" aria-expanded="${!!(ST.openClips || {})['role:' + r.id]}">${(ST.openClips || {})['role:' + r.id] ? '■ Fechar clip' : '▶ Clip do papel'}</button><button class="btn sm ghost" onclick="A.roleDemo('${r.id}')" title="Abrir no campo animado completo">Campo completo</button></div>
    ${(ST.openClips || {})['role:' + r.id] ? clipHTML('role:' + r.id, (() => { const n = MOVE_NOTES[r.id], ph = `<span>${r.phase === 'IP' ? L('posição → movimento → acção com bola', 'shape → movement → ball action') : r.tags.includes('OUT') ? L('bloco → fica alto → contra-ataque', 'block → stays high → counter') : L('bloco → ataque adversário → reacção', 'block → opponent attack → reaction')}</span>`; return n ? `<b>${esc(r.ab)}</b> <span>${esc(n[1])}</span> · ${ph}` : ph; })()) : ''}
  </div>`).join('')}</div>`;
}

function vTI() {
  return `<div class="top"><div><h2>Instruções</h2></div></div>
  <div class="panel"><h3>Mentalidade</h3><div class="grid g2">${MENTALITIES.map(m => `<div><b>${m}</b><p class="small muted">${esc(MENTALITY_INFO[m])}</p></div>`).join('')}</div></div>
  ${['IP','OOP'].map(ph => `<div class="panel" style="margin-top:16px"><h3>Instruções de equipa — ${ph === 'IP' ? 'com bola' : 'sem bola'}</h3>
    <div class="table-wrap" style="max-height:none"><table><thead><tr><th>Instrução</th><th>Opções</th><th>O que faz</th><th>Prós</th><th>Contras</th></tr></thead><tbody>
    ${TEAM_INSTRUCTIONS.filter(d => d.ph === ph).map(d => `<tr><td><b>${esc(d.n)}</b>${d.fm27 ? ' <span class="chip new">FM27</span>' : ''}<div class="small muted"><span>${esc(d.area)}</span>${d.unconf ? ` · <span>opções a confirmar no jogo</span>` : ''}</div>${SIM_TI.has(d.k) ? `<button class="btn sm" style="margin-top:4px" onclick="A.clipToggle('ti:${d.k}')" aria-expanded="${!!(ST.openClips || {})['ti:' + d.k]}">${(ST.openClips || {})['ti:' + d.k] ? '■ Fechar' : '▶ Ver efeito'}</button>` : ''}</td><td class="small">${d.opts.map((o, i) => i === d.def ? `<u>${esc(o)}</u>` : esc(o)).join(' · ')}</td><td class="small">${esc(d.d)}</td><td class="small">${esc(d.pro)}</td><td class="small">${esc(d.con)}</td></tr>${(ST.openClips || {})['ti:' + d.k] ? `<tr class="cliprow"><td colspan="5"><div class="clip-pair">${[0, d.opts.length - 1].map(v => clipHTML('ti:' + d.k + ':' + v, `<b>${esc(d.opts[v])}</b>${v === d.def ? ' (default)' : ''}`)).join('')}</div><p class="small muted" style="margin:6px 0 0"><span>${L('Mesma equipa e mesma fase', 'Same team, same phase')}</span> (<span>${esc(PHASES[TI_PHASE[d.k] ?? 0].n)}</span>)<span>${L('; só muda', '; only')}</span> <b>${esc(d.n)}</b> ${L('', '<span>changes.</span>')} <span>${d.ph === 'IP' ? 'Instrução com bola.' : 'Instrução sem bola.'}</span></p></td></tr>` : ''}`).join('')}
    </tbody></table></div></div>`).join('')}
  <div class="grid g2" style="margin-top:16px">
    <div class="panel"><h3>Instruções individuais</h3>${PLAYER_INSTRUCTIONS.map(p => `<p class="small"><b>${esc(p.n)}</b>${p.fm27 ? ' <span class="chip new">FM27</span>' : ''}<br>${esc(p.d)} <span class="muted">${esc(p.tip)}</span></p>`).join('')}</div>
    <div class="grid">
      <div class="panel"><h3>Instruções de adversário</h3>${OPPOSITION_INSTRUCTIONS.map(p => `<p class="small"><b>${esc(p.n)}</b><br>${esc(p.d)} <span class="muted">${esc(p.tip)}</span></p>`).join('')}</div>
      <div class="panel"><h3>Tactical Tweaks <span class="chip new">FM27</span></h3><p class="small muted">Atalhos no ecrã de jogo que alteram várias instruções de uma vez. Passe o rato no jogo para ver exactamente quais.</p>
        ${TACTICAL_TWEAKS.map(p => `<p class="small"><b>${esc(p.n)}</b> — ${esc(p.d)}<br><span class="muted"><span>Quando:</span> <span>${esc(p.when)}</span></span></p>`).join('')}</div>
    </div></div>`;
}

function conceptDiagram(k) {
  if (k === 'complexity') {
    const tiers = [
      [L('Baixa', 'Low'), L('0–5 desvios · adaptação rápida', '0–5 tweaks · fast learning'), 'var(--good)', 32],
      [L('Média', 'Medium'), L('6–12 desvios · equilíbrio', '6–12 tweaks · balanced'), 'var(--warn)', 66],
      [L('Alta', 'High'), L('13+ desvios · exige elite e treino', '13+ tweaks · needs elite IQ & training'), 'var(--crit)', 96],
    ];
    return `<div class="concept-viz">${tiers.map(([l, d, col, w]) => `<div class="cv-row"><span class="cv-lbl">${l}</span><div class="cv-bar"><i style="width:${w}%;background:${col}"></i></div><span class="small muted">${d}</span></div>`).join('')}</div>`;
  }
  if (k === 'effectiveness') {
    const pillars = [
      [L('Compreensão tática', 'Tactical Understanding'), L('Treino + jogos + simplicidade', 'Training + matches + simplicity'), 'var(--good)'],
      [L('Capacidade do jogador', 'Player Ability'), L('Adequação ao papel + CA', 'Role suitability + CA'), '#3b82f6'],
      [L('Adesão do plantel', 'Player Buy-In'), L('Apoio + coesão + promessas', 'Support + cohesion + promises'), 'var(--warn)'],
      [L('Info. próximo jogo', 'Next Match Info'), L('Analistas + preparação', 'Analysts + match prep'), '#a855f7'],
    ];
    return `<div class="concept-viz cv-pillars">${pillars.map(([l, d, col]) => `<div class="cv-pill" style="border-top-color:${col}"><b>${l}</b><span class="small muted">${d}</span></div>`).join('')}</div>`;
  }
  return '';
}

function vConcepts() {
  const open = ST.openClips || {};
  return `<div class="top"><div><h2>Conceitos, princípios e adversários</h2></div></div>
  <div class="grid g2">${CONCEPTS.map(c => {
    const cid = 'concept:' + c.k, hasClip = CONCEPT_CLIPS.has(c.k), isOpen = !!open[cid];
    return `<div class="panel">
      <div class="row" style="justify-content:space-between;align-items:center">
        <h3 style="margin:0">${esc(c.n)}</h3>
        ${hasClip ? `<ui-toggle class="clip-tg" data-clip-toggle="${cid}"${isOpen ? ' default-pressed' : ''}><button type="button" class="btn sm">${L('Ver no campo', 'Show on pitch')}</button></ui-toggle>` : ''}
      </div>
      <p class="small" style="margin-top:8px">${esc(c.t)}</p>
      ${conceptDiagram(c.k)}
      ${hasClip && isOpen ? clipHTML(cid, `<b>${esc(c.n)}</b> · <span>${L('animação tática', 'tactical animation')}</span>`) : ''}
    </div>`;
  }).join('')}</div>
  <div class="panel" style="margin-top:16px"><h3>Contra cada sistema</h3>
    <div class="grid g2" style="margin-top:10px">${OPPONENTS.map(o => {
      const cid = 'opp:' + o.k, isOpen = !!open[cid];
      return `<div class="opp-card">
        <div class="row" style="justify-content:space-between;align-items:center">
          <b>${esc(o.f)}</b>
          <ui-toggle class="clip-tg" data-clip-toggle="${cid}"${isOpen ? ' default-pressed' : ''}><button type="button" class="btn sm">${L('Ver plano', 'Show plan')}</button></ui-toggle>
        </div>
        <p class="small" style="margin:.45em 0 0">${esc(o.t)}</p>
        ${isOpen ? clipHTML(cid, `<span>${L('Como bater', 'How to beat')}</span> <b>${esc(o.f)}</b>`) : ''}
      </div>`;
    }).join('')}</div></div>
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

