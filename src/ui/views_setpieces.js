// ============================================================
// Bolas paradas & capitães
// ============================================================
// Coordenadas da área vista de cima: x 0–68 (largura do campo), y 0 = linha de fundo
const SP_POS = {
  attack: { taker: [2, 1.5], near: [26, 4.5], centre: [34, 6.5], far: [43, 5], spot: [34, 13.5], edge: [34, 21.5], short: [9, 9], back0: [20, 35], back1: [48, 35], back2: [34, 37.5] },
  defend: { zn: [26.5, 3.2], zc: [34, 7], zf: [41.5, 3.2], m0: [27.5, 12], m1: [40.5, 12], m2: [34, 16], m3: [21, 14], de: [34, 23], o0: [25, 36], o1: [43, 36] },
};
function spPitch(rows, layout, title, gk) {
  const P = SP_POS[layout];
  const dots = rows.map(r => { const xy = P[r.k]; if (!xy) return ''; const [x, y] = xy; const col = r.k.startsWith('back') || r.k.startsWith('o') ? '#2d5f8f' : r.k === 'taker' ? '#b7791f' : '#0b5d3a';
    return `<g transform="translate(${x},${y})"><circle r="1.9" fill="${col}" stroke="#fff" stroke-width=".3"/><text y=".6" text-anchor="middle" font-size="1.5" font-weight="700" fill="#fff">${esc(r.x.code.replace(/[LR]$/, '').slice(0, 3))}</text>
      <text y="3.7" text-anchor="middle" font-size="1.55" fill="#fff" stroke="rgba(0,0,0,.6)" stroke-width=".3" paint-order="stroke">${esc(shortName(r.x.p).slice(0, 11))}</text></g>`; }).join('');
  return `<figure class="sp-fig"><svg viewBox="0 -1 68 42" class="sp-svg" role="img" aria-label="${esc(title)}">
    <rect x="0" y="-1" width="68" height="43" fill="var(--pitch)"/>
    <g fill="none" stroke="var(--pitchline)" stroke-width=".3"><line x1="0" y1="0" x2="68" y2="0"/><rect x="13.85" y="0" width="40.3" height="16.5"/><rect x="24.85" y="0" width="18.3" height="5.5"/>
      <path d="M 26.7 16.5 A 9.15 9.15 0 0 0 41.3 16.5"/><circle cx="34" cy="11" r=".3" fill="var(--pitchline)"/><path d="M0,1 A1,1 0 0 0 1,0"/></g>
    <rect x="30.34" y="-1" width="7.32" height="1" fill="rgba(255,255,255,.85)"/>
    ${gk ? `<g transform="translate(34,1.2)"><circle r="1.6" fill="#3c3f46" stroke="#fff" stroke-width=".3"/><text y=".55" text-anchor="middle" font-size="1.3" font-weight="700" fill="#fff">GK</text></g>` : ''}
    ${dots}</svg><figcaption class="small muted">${esc(title)}</figcaption></figure>`;
}
function vSetPieces() {
  const ps = players();
  if (ps.length < 11) return `<div class="panel empty"><h3>Carregue o plantel primeiro</h3><p>As bolas paradas são escolhidas a partir dos atributos dos jogadores do teu XI.</p><button class="btn primary" onclick="A.tab('team','squad')">Ir para Plantel</button> <button class="btn" onclick="A.sampleSquad();A.tab('tactic','setpieces')">Usar exemplo</button></div>`;
  const r = setPiecePlan(ST.tactic, ps, oppPlayers()); ST._sp = r;
  if (!r) return '<div class="panel empty">Não consegui montar o XI.</div>';
  const row = (label, x, extra) => `<tr><td>${esc(label)}</td><td>${x ? `<b>${esc(x.p.name)}</b> <span class="muted small">${esc(x.code)}</span>` : '—'}</td><td class="small muted">${extra || ''}</td></tr>`;
  const a = (x, ks) => ks.map(k => `${attrName(k)} ${Math.round(x.p.a[k] ?? 0)}`).join(' · ');
  return `<div class="top"><div><h2>Bolas paradas & capitães</h2><p class="muted small" style="margin:0"><span>Escolhidos entre o XI de</span> “${esc(ST.tactic.name)}”. <span>${oppPlayers().length ? 'Marcações individuais contra os mais fortes no ar do adversário carregado.' : 'Carregue o adversário para marcações individuais nominais.'}</span></p></div>
    <div class="row"><button class="btn" onclick="A.tab('tactic','sheet')">Ver na Ficha para o FM27</button></div></div>
  <div class="grid g2">
    <div class="panel"><h3>Marcadores</h3><table><tbody>
      ${row('Cantos do lado direito', r.corners.right.x, r.corners.right.swing ? 'curva ' + r.corners.right.swing : a(r.corners.right.x, ['cor', 'cro']))}
      ${row('Cantos do lado esquerdo', r.corners.left.x, r.corners.left.swing ? 'curva ' + r.corners.left.swing : a(r.corners.left.x, ['cor', 'cro']))}
      ${row('Livres directos', r.fk.direct, r.fk.direct ? a(r.fk.direct, ['fre', 'tec', 'lon']) : 'ninguém com Livres ≥ 14 — cruzar')}
      ${row('Livres para cruzar', r.fk.cross, a(r.fk.cross, ['fre', 'cro']))}
      ${r.pens.map((x, i) => row(`Penáltis — ${i + 1}.º`, x, a(x, ['pen', 'cmp']))).join('')}
      ${row('Lançamentos longos', r.longThrow, r.longThrow ? a(r.longThrow, ['lth']) : 'ninguém com Lançamentos Longos ≥ 14')}
      ${row('Capitão', r.caps[0], r.caps[0] ? a(r.caps[0], ['ldr', 'det']) : '')}
      ${row('Vice-capitão', r.caps[1], r.caps[1] ? a(r.caps[1], ['ldr', 'det']) : '')}
    </tbody></table>
    <p class="small muted">Sem dados do pé preferido o efeito do canto não é indicado. Exportações com a coluna do pé (FM View Exporter, fmsave) mostram-no.</p></div>
    <div class="panel"><h3>Ideias para os cantos e livres</h3>
      ${r.tips.map((tp, i) => { const d = tp.ti && TI_BY_KEY[tp.ti], on = d && tiVal(ST.tactic, tp.ti) === tp.v;
        return `<div class="finding"><div class="sev ${on ? 'good' : 'info'}">${on ? '✓' : i + 1}</div><div><span class="small">${esc(tp.t)}</span>${d && !on ? `<div><button class="btn sm" style="margin-top:4px" onclick="A.applySpTip(${i})">${esc(d.n)} → ${esc(d.opts[tp.v])}</button></div>` : ''}</div></div>`; }).join('') || '<p class="muted small">Nada a destacar.</p>'}
    </div>
  </div>
  <div class="grid g2" style="margin-top:16px">
    <div class="panel"><h3>Canto ofensivo</h3>${spPitch(r.attack, 'attack', 'Canto do lado esquerdo; do outro lado é o espelho.', false)}
      <table><tbody>${r.attack.map(x => `<tr><td>${esc(x.n)}</td><td><b>${esc(x.x.p.name)}</b></td><td class="small muted">${esc(x.w)}</td></tr>`).join('')}</tbody></table></div>
    <div class="panel"><h3><span>Canto defensivo</span> — <span>${esc(r.system)}</span></h3>${spPitch(r.defend, 'defend', 'Posições sem bola no canto contra.', true)}
      <table><tbody>${r.defend.map(x => `<tr><td>${esc(x.n)}</td><td><b>${esc(x.x.p.name)}</b></td><td class="small muted">${esc(x.w)}</td></tr>`).join('')}</tbody></table></div>
  </div>`;
}
