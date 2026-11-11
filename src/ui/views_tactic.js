// ============================================================
// Vistas da Tática: analisar, importar, ficha FM27
// ============================================================
function vTactic() {
  const t = ST.tactic, au = auditTactic(t, players()), sel = ST.sel;
  const forms = Object.keys(FORMATIONS).map(f => [f, f]);
  const cats = Object.entries(au.scores);
  return `<div class="top"><div><h2>${esc(ST.tactic.name)}</h2><p class="muted small" style="margin:0">Clique num jogador do campo para mudar papéis, jogador e instruções individuais.</p></div>
    <div class="row no-print">
      <button class="btn primary" onclick="A.simOpen()">▶ Ver em campo</button>
      <button class="btn" onclick="A.newTactic()">Nova</button>
      <button class="btn" onclick="A.saveTactic()">Guardar</button>
      <button class="btn" onclick="A.exportTactic()">Exportar JSON</button>
      <label class="btn">Importar JSON<input type="file" accept=".json" hidden onchange="A.importJSON(this)"></label>
      <button class="btn" onclick="A.print()">Imprimir/PDF</button>
    </div></div>
  <div class="split">
    <div class="grid">
      <div class="panel grid">
        <label class="f">Nome<input type="text" value="${esc(t.name)}" oninput="A.setName(this.value)"></label>
        <div class="grid g2">
          <label class="f">Formação com bola (IP)${selectHTML(forms, t.ipForm, "A.setForm('IP',this.value)")}</label>
          <label class="f">Formação sem bola (OOP)${selectHTML(forms, t.oopForm, "A.setForm('OOP',this.value)")}</label>
        </div>
        <label class="f">Mentalidade${selectHTML(MENTALITIES.map(m => [m, m]), t.ment, 'A.setMent(this.value)')}</label>
        <p class="small muted" style="margin:0">${esc(MENTALITY_INFO[t.ment])}</p>
      </div>
      <div class="panel">
        <div class="row" style="justify-content:space-between;margin-bottom:8px">
          <div class="seg">${['IP','OOP','BOTH'].map(m => `<button class="${ST.pitchMode === m ? 'on' : ''}" onclick="A.pitch('${m}')">${m === 'BOTH' ? 'Ambas' : m}</button>`).join('')}</div>
          <div class="row">${players().length ? `<button class="btn sm" onclick="A.autoXI()">Auto XI</button><button class="btn sm" onclick="A.clearXI()">Limpar XI</button>` : '<span class="small muted">Carregue o plantel para atribuir jogadores</span>'}</div>
        </div>
        ${pitchSVG(t, ST.pitchMode, sel)}
        <p class="small muted" style="margin:6px 0 0">Clique num jogador para editar papéis e instruções. ${ST.pitchMode === 'BOTH' ? 'Círculos azuis = posição sem bola.' : ''}</p>
      </div>
      ${sel != null ? vSlotEditor(sel) : ''}
    </div>
    <div class="grid">
      <div class="panel"><div class="score-ring">${ring(au.overall)}<div class="bars">
        ${cats.map(([c, v]) => `<div class="bar"><span>${c}</span><div class="t"><i style="width:${v}%;background:${fitColor(v)}"></i></div><b>${v}</b></div>`).join('')}
      </div></div>
      <p class="small muted" style="margin:10px 0 0">Complexidade: <b>${au.complexity.level}</b> (${au.complexity.cx})${au.complexity.parts.length ? ' — ' + au.complexity.parts.slice(0, 4).map(p => esc(p[0])).join('; ') + (au.complexity.parts.length > 4 ? '…' : '') : ''}</p></div>
      <div class="panel"><h3>Diagnóstico</h3>
        ${au.findings.map(f => `<div class="finding"><div class="sev ${f.sev}">${{ crit: '!', warn: '▲', info: 'i', good: '✓' }[f.sev]}</div><div><b>${esc(f.title)} <span class="chip">${esc(f.cat)}</span></b><span class="small">${esc(f.text)}</span>${f.fix ? `<div class="fix">→ ${esc(f.fix)}</div>` : ''}</div></div>`).join('')}
      </div>
    </div>
  </div>
  <div class="panel" style="margin-top:16px">
    <div class="row" style="justify-content:space-between"><h3>Instruções de equipa</h3><div class="row"><span class="small muted">Instruções de adversário activas:</span><input type="number" min="0" max="20" value="${t.ois || 0}" style="width:64px" onchange="A.setOIs(this.value)"><button class="btn sm" onclick="A.resetTI()">Repor default</button></div></div>
    <p class="small muted">Nomes do FM26. A borda verde marca instruções fora do default (cada uma soma complexidade no FM27).</p>
    ${['IP','OOP'].map(ph => `<h4 style="margin-top:12px">${ph === 'IP' ? 'Com bola' : 'Sem bola'}</h4><div class="ti-grid">${TEAM_INSTRUCTIONS.filter(d => d.ph === ph).map(d => {
      const v = tiVal(t, d.k);
      return `<div class="ti ${v !== d.def ? 'changed' : ''}"><div class="h"><span>${esc(d.n)}${d.fm27 ? ' <span class="chip new">FM27</span>' : ''}</span><span class="small muted">${esc(d.area)}</span></div>
        ${selectHTML(d.opts.map((o, i) => [i, o + (i === d.def ? ' (default)' : '')]), v, `A.setTI('${d.k}',this.value)`)}
        <details><summary>${esc(d.d)}</summary><p><b>Prós:</b> ${esc(d.pro)}</p><p><b>Contras:</b> ${esc(d.con)}</p>${d.need ? `<p class="muted">Atributos relevantes: ${Object.values(d.need).flat().filter((x, i, a) => a.indexOf(x) === i).map(attrName).join(', ')}</p>` : ''}</details></div>`;
    }).join('')}</div>`).join('')}
  </div>`;
}

function vSlotEditor(i) {
  const t = ST.tactic, s = t.slots[i];
  const a = slotInfo(t.ipForm, s.ipCode), b = slotInfo(t.oopForm, s.oopCode);
  const p = pById(s.pid);
  const ipRoles = rolesFor(a.group, 'IP', a.wideCB), oopRoles = rolesFor(b.group, 'OOP', b.wideCB);
  const lab = (r) => { const sc = p ? (roleScore(p, r) || {}).score : null; return r.name + (sc != null ? ` — ${sc}` : ''); };
  const ri = ROLE_BY_ID[s.ip], ro = ROLE_BY_ID[s.oop];
  const plist = players().map(q => ({ q, sc: slotScore(q, t, s) })).sort((x, y) => y.sc - x.sc);
  const roleBox = (r) => r ? `<div class="role-card"><b>${esc(r.name)}</b> <span class="chip ${r.phase === 'IP' ? 'ip' : 'oop'}">${r.phase}</span> ${srcTag(r)}
    <p class="small">${esc(r.desc)}</p><p class="small muted">💡 ${esc(r.tip)}</p>
    <div class="attrs"><b>Chave:</b> ${r.key.map(k => attrChip(k, p)).join('')}</div>
    <div class="attrs"><b>Preferidos:</b> ${r.pref.map(k => attrChip(k, p)).join('')}</div></div>` : '';
  return `<div class="panel grid"><div class="row" style="justify-content:space-between"><h3>${s.ipCode} <span class="muted small">→ sem bola ${s.oopCode}</span></h3><button class="btn sm" onclick="A.sel(${i})">Fechar</button></div>
    ${players().length ? `<label class="f">Jogador${selectHTML([['', '— vazio —']].concat(plist.map(x => [x.q.id, `${x.q.name} (${x.sc})${x.q.posRaw ? ' · ' + x.q.posRaw : ''}`])), s.pid || '', `A.setPlayer(${i},this.value)`)}</label>` : ''}
    <div class="grid g2">
      <label class="f">Papel com bola (${GROUPS[a.group]})${selectHTML(ipRoles.map(r => [r.id, lab(r)]), s.ip, `A.setRole(${i},'IP',this.value)`)}</label>
      <label class="f">Papel sem bola (${GROUPS[b.group]})${selectHTML(oopRoles.map(r => [r.id, lab(r)]), s.oop, `A.setRole(${i},'OOP',this.value)`)}</label>
    </div>
    ${p ? `<div class="small">Adequação: ${fitBadge(slotScore(p, t, s))} · melhores papéis IP aqui: ${bestRolesForPlayer(p, a.group, 'IP', a.wideCB).map(x => `${esc(x.r.name)} ${x.s}`).join(' · ')}</div>` : ''}
    ${roleBox(ri)}${roleBox(ro)}
    ${p ? (() => { const rec = recommendPIs(p, ri, ro, a.group); return rec.length ? `<div class="recs"><b class="small">Recomendadas para ${esc(p.name)}</b>${rec.map(r => `<div class="rec"><div><b>${esc(r.pi)}</b><span class="small muted"> — ${esc(r.why)}</span></div>${(s.pis || []).includes(r.pi) ? '<span class="chip ip">aplicada</span>' : `<button class="btn sm" onclick="A.applyPI(${i}, '${r.pi.replace(/'/g, "\\'")}')">Aplicar</button>`}</div>`).join('')}</div>` : ''; })() : ''}
    <details><summary>Todas as instruções individuais (${(s.pis || []).length} activas) — cada uma soma complexidade</summary>
      <div class="grid" style="margin-top:6px">${PLAYER_INSTRUCTIONS.map((pi, j) => `<label class="small"><input type="checkbox" ${(s.pis || []).includes(pi.n) ? 'checked' : ''} onchange="A.togglePI(${i}, ${j})"> <b>${esc(pi.n)}</b> — <span class="muted">${esc(pi.tip)}</span></label>`).join('')}</div>
    </details></div>`;
}

function vImport() {
  const r = ST.fmf;
  return `<div class="top"><div><h2>Importar</h2></div></div>
  <div class="grid g2">
    <div class="panel">
      <div class="drop" id="drop" ondragover="event.preventDefault();this.classList.add('over')" ondragleave="this.classList.remove('over')" ondrop="event.preventDefault();this.classList.remove('over');A.fmf(event.dataTransfer.files[0])">
        <p><b>Arraste o .fmf para aqui</b> ou</p><label class="btn primary">Escolher ficheiro<input type="file" accept=".fmf" hidden onchange="A.fmf(this)"></label>
        <p class="small muted">Os ficheiros ficam em Documentos/Sports Interactive/Football Manager 27/tactics</p></div>
      <div class="notice warn"><b>Limite do formato:</b> no FM27, o conteúdo da tática dentro do .fmf está <b>comprimido e cifrado</b> pelo jogo (blocos com nonce e tag de autenticação, padrão AES-GCM). Nenhuma ferramenta externa consegue ler formação, papéis ou instruções sem a chave do jogo. O inspector confirma o formato e mostra a estrutura; a análise faz-se recriando a tática no analisador (≈1 minuto).</div>
      <h4>Outras opções</h4>
      <div class="row"><label class="btn">Importar tática JSON (desta app)<input type="file" accept=".json" hidden onchange="A.importJSON(this)"></label><button class="btn" onclick="A.tab('tactic','analyze')">Recriar manualmente →</button></div>
    </div>
    <div class="panel">${!r ? '<div class="empty">Nenhum ficheiro inspeccionado.</div>' : r.loading ? `<div class="empty">A analisar ${esc(r.name)}…</div>` : r.error ? `<div class="notice warn">Erro: ${esc(r.error)}</div>` : `
      <h3>${esc(r.name)}</h3>
      <div class="kv"><span>Tamanho</span><b>${r.size.toLocaleString('pt-PT')} bytes</b><span>Entropia</span><b>${r.entropy} bits/byte ${+r.entropy > 7.5 ? '(comprimido/cifrado)' : ''}</b>
      <span>Cabeçalho</span><code>${esc(r.head)}</code><span>ASCII</span><code>${esc(r.headAscii)}</code>
      <span>Streams descomprimidos</span><b>${r.streams.length ? r.streams.map(s => `${s.fmt}@${s.off} → ${s.len}B`).join(', ') : 'nenhum'}</b>
      <span>Outras assinaturas</span><b>${r.magic.length ? esc(r.magic.join(', ')) : '—'}</b>
      <span>Textos encontrados</span><b>${r.strings.length}</b></div>
      ${r.container && r.container.recognized ? `<div class="notice" style="margin-top:12px"><b>✓ Formato .fmf do FM27 reconhecido.</b> ${r.container.sections.length} secções (${r.container.sections.map(x => esc(x.name) + ' ' + x.len + 'B' + (x.kind === 'zstd' ? ' — índice zstd' : '')).join(', ')}) e ${r.container.blocks.length} ficheiros internos cifrados (${r.container.blocks.map(b => b.cipher + 'B').join(', ')}). O maior (${r.container.largest.cipher}B cifrados) é a tática.
        <br><b>Resultado:</b> o conteúdo não é legível fora do jogo. Use o nome do ficheiro como ponto de partida e recrie a tática no analisador.
        <div class="row" style="margin-top:8px"><button class="btn primary" onclick="A.fmfApplyHints()">Criar “${esc(r.name.replace(/\.fmf$/i, ''))}” no analisador</button></div></div>` : ''}
      <h4 style="margin-top:12px">Texto legível</h4>
      ${r.decoded || r.matches.mentality.length || r.matches.formations.length ? `
        ${r.matches.roles.length ? `<p class="small"><b>Papéis:</b> ${r.matches.roles.map(x => `<span class="chip">${esc(x)}</span>`).join('')}</p>` : ''}
        ${r.matches.instructions.length ? `<p class="small"><b>Instruções:</b> ${r.matches.instructions.map(x => `<span class="chip">${esc(x)}</span>`).join('')}</p>` : ''}
        ${r.matches.mentality.length ? `<p class="small"><b>Mentalidade:</b> ${r.matches.mentality.map(x => `<span class="chip">${esc(x)}</span>`).join('')}</p>` : ''}
        ${r.matches.formations.length ? `<p class="small"><b>Formações:</b> ${r.matches.formations.map(x => `<span class="chip">${esc(x)}</span>`).join('')}</p>` : ''}
        <p class="small muted">Os nomes encontrados indicam o que a tática contém, mas não a que posição pertencem. Use-os como guia para recriar a tática no analisador.</p>
        <button class="btn primary" onclick="A.fmfApplyHints()">Usar nome/mentalidade e abrir analisador</button>`
      : '<p class="small muted">Nenhum nome de papel/instrução legível no ficheiro.</p>'}
      <div class="row" style="margin-top:10px"><button class="btn" onclick="A.fmfStrings()">Exportar textos (.txt)</button></div>
      <details style="margin-top:10px"><summary>Ver primeiros 200 textos</summary><pre class="mono" style="white-space:pre-wrap;max-height:300px;overflow:auto">${esc(r.strings.slice(0, 200).join('\n'))}</pre></details>`}
    </div></div>`;
}

const FM_SECTIONS = [
  ['Team Instructions › In Possession › Overview', ['passing','tempo','width','att_trans','creative','timewaste','set_pieces']],
  ['In Possession › Final Third', ['patience','long_shots','crossing','dribbling']],
  ['In Possession › Progression', ['support_runs','progress','reception']],
  ['In Possession › Buildup', ['buildup','goal_kicks','gk_speed','gk_target']],
  ['Out of Possession › Overview', ['loe','dline','dl_behaviour','trigger','def_trans','tackling']],
  ['Out of Possession › High Press / Mid Block / Low Block', ['short_gk','press_trap','cross_eng']],
];

function vSheet() {
  const t = ST.tactic, pm = Object.fromEntries(players().map(p => [p.id, p]));
  const ck = '<input type="checkbox" aria-label="feito">';
  const changed = TEAM_INSTRUCTIONS.filter(d => tiVal(t, d.k) !== d.def).length;
  return `<div class="top"><div><h2>Ficha para o FM27 — ${esc(t.name)}</h2>
    <p class="muted small" style="margin:0">Recrie a tática no jogo por esta ordem e guarde-a lá: o FM27 gera o .fmf. Marque cada passo à medida que avança.</p></div>
    <div class="row no-print"><button class="btn" onclick="A.print()">Imprimir/PDF</button><button class="btn" onclick="A.copySheet()">Copiar texto</button><button class="btn" onclick="A.tab('tactic','analyze')">Voltar</button></div></div>
  <div class="grid g2 sheet">
    <div class="panel"><h3>1. Topo do ecrã de táticas</h3>
      <table><tbody>
        <tr><td>${ck}</td><td>Style</td><td><b>Clean Slate</b> (para começar sem instruções pré-definidas)</td></tr>
        <tr><td>${ck}</td><td>Mentality</td><td><b>${esc(t.ment)}</b></td></tr>
        <tr><td>${ck}</td><td>In Possession</td><td><b>${esc(t.ipForm)}</b></td></tr>
        <tr><td>${ck}</td><td>Out of Possession</td><td><b>${esc(t.oopForm)}</b></td></tr>
      </tbody></table>
      <h3 style="margin-top:14px">2. Team Shape — papéis</h3>
      <table><thead><tr><th></th><th>Posição</th><th>In Possession</th><th>Out of Possession</th></tr></thead><tbody>
      ${t.slots.map(s => { const ri = ROLE_BY_ID[s.ip], ro = ROLE_BY_ID[s.oop], p = pm[s.pid];
        return `<tr><td>${ck}</td><td><b>${esc(s.ipCode)}</b>${s.oopCode !== s.ipCode ? ` <span class="muted small">→ ${esc(s.oopCode)}</span>` : ''}${p ? `<div class="small muted">${esc(p.name)}</div>` : ''}</td>
          <td>${esc(ri ? ri.name : '?')}</td><td>${esc(ro ? ro.name : '?')}</td></tr>`; }).join('')}
      </tbody></table>
      ${t.slots.some(s => s.pis && s.pis.length) ? `<h3 style="margin-top:14px">3. Instruções individuais</h3><table><tbody>${t.slots.filter(s => s.pis && s.pis.length).map(s => `<tr><td>${ck}</td><td><b>${esc(s.ipCode)}</b></td><td>${s.pis.map(esc).join('<br>')}</td></tr>`).join('')}</tbody></table>` : ''}
    </div>
    <div class="panel"><h3>${t.slots.some(s => s.pis && s.pis.length) ? '4' : '3'}. Team Instructions</h3>
      <p class="small muted">${changed} instrução(ões) fora do default, a <b>negrito</b>. As restantes ficam como estão.</p>
      ${FM_SECTIONS.map(([sec, keys]) => `<h4 style="margin-top:12px">${esc(sec)}</h4><table><tbody>${keys.map(k => { const d = TI_BY_KEY[k]; const v = tiVal(t, k), ch = v !== d.def;
        return `<tr class="${ch ? '' : 'muted'}"><td>${ch ? ck : ''}</td><td>${esc(d.n)}</td><td>${ch ? `<b>${esc(d.opts[v])}</b>` : esc(d.opts[v])}</td></tr>`; }).join('')}</tbody></table>`).join('')}
      ${t.ois ? `<p class="small" style="margin-top:12px">${ck} Opposition Instructions: ${t.ois} activa(s) — defina-as antes de cada jogo.</p>` : ''}
    </div>
  </div>`;
}

function sheetText(t) {
  const L = [`${t.name}`, `Style: Clean Slate | Mentality: ${t.ment}`, `In Possession: ${t.ipForm} | Out of Possession: ${t.oopForm}`, '', 'PAPÉIS (IP / OOP)'];
  t.slots.forEach(s => L.push(`${s.ipCode}${s.oopCode !== s.ipCode ? '→' + s.oopCode : ''}: ${ROLE_BY_ID[s.ip].name} / ${ROLE_BY_ID[s.oop].name}${s.pis && s.pis.length ? ' [' + s.pis.join('; ') + ']' : ''}`));
  L.push('', 'INSTRUÇÕES DE EQUIPA (só as que mudam)');
  FM_SECTIONS.forEach(([sec, keys]) => { const ch = keys.filter(k => tiVal(t, k) !== TI_BY_KEY[k].def); if (ch.length) L.push(sec + ': ' + ch.map(k => `${TI_BY_KEY[k].n} = ${TI_BY_KEY[k].opts[tiVal(t, k)]}`).join(', ')); });
  return L.join('\n');
}

