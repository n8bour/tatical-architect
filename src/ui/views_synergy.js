// ============================================================
// Sinergia entre papéis — todas as fases (Gerador e Analisador)
// ============================================================
const SYN_COL = { build: '#14b8a6', chain: 'var(--good)', cross: '#3b82f6', front: '#d97706', flank: '#a855f7', press: '#ef4444', flankdef: '#a855f7', counter: '#f59e0b' };
function synergyMap(t, links, form, title) {
  const F = FORMATIONS[form] || [], pos = Object.fromEntries(F.map(z => [z.code, z]));
  const P = (code) => { const z = pos[code]; return z ? [6 + z.y * 0.88, 4 + z.x * 0.56] : [50, 32]; };   // horizontal: ataque para a direita
  const arrows = links.map(l => { const [x1, y1] = P(l.from.code), [x2, y2] = P(l.to.code); const dx = x2 - x1, dy = y2 - y1, d = Math.hypot(dx, dy) || 1, k = 3.2 / d;
    return `<line x1="${(x1 + dx * k).toFixed(1)}" y1="${(y1 + dy * k).toFixed(1)}" x2="${(x2 - dx * k).toFixed(1)}" y2="${(y2 - dy * k).toFixed(1)}" stroke="${SYN_COL[l.type]}" stroke-width="${0.6 + l.v * 0.9}" marker-end="url(#ar-${l.type})" ${['flank', 'flankdef'].includes(l.type) ? 'stroke-dasharray="1.6 1"' : ''} opacity=".95"/>`; }).join('');
  const defs = Object.entries(SYN_COL).map(([k, c]) => `<marker id="ar-${k}" viewBox="0 0 6 6" refX="5" refY="3" markerWidth="4" markerHeight="4" orient="auto"><path d="M0,0 L6,3 L0,6 z" fill="${c}"/></marker>`).join('');
  const phaseKey = form === t.oopForm && title === 'oop' ? 'oop' : 'ip';
  const dots = t.slots.map(s => { const code = phaseKey === 'oop' ? s.oopCode : s.ipCode, r = ROLE_BY_ID[phaseKey === 'oop' ? s.oop : s.ip], [x, y] = P(code);
    return `<g transform="translate(${x.toFixed(1)},${y.toFixed(1)})"><circle r="2.9" fill="${phaseKey === 'oop' ? '#1e3a5f' : '#0b5d3a'}" stroke="#fff" stroke-width=".35"/><text y="1" text-anchor="middle" font-size="2.2" font-weight="700" fill="#fff">${esc(r ? r.ab : '')}</text></g>`; }).join('');
  return `<svg class="syn-map" viewBox="0 0 100 64" role="img" aria-label="${L('Mapa de ligações', 'Link map')}"><defs>${defs}</defs>
    <rect width="100" height="64" rx="2" fill="var(--pitch)"/><g fill="none" stroke="var(--pitchline)" stroke-width=".3"><rect x="2" y="2" width="96" height="60"/><line x1="50" y1="2" x2="50" y2="62"/><circle cx="50" cy="32" r="7"/><rect x="83" y="17" width="15" height="30"/><rect x="2" y="17" width="15" height="30"/></g>
    ${arrows}${dots}</svg>`;
}
function synergyPanel(t, players, opts = {}) {
  const rep = synergyReport(t, players), N = SYN_NAMES(), PH = SYN_PHASES();
  const colr = (v) => v >= 75 ? 'var(--good)' : v >= 50 ? 'var(--warn)' : 'var(--crit)';
  const bar = (k, v) => `<div class="bar-row"><span>${esc(N[k] || k)}</span><div class="bar"><i style="width:${v}%;background:${colr(v)}"></i></div><b>${v}</b></div>`;
  const order = ['build', 'att', 'def', 'trans'];
  const card = (ph) => { const P = rep.phases[ph]; return `<div class="syn-phase"><div class="row" style="justify-content:space-between"><b>${esc(PH[ph])}</b><span class="syn-score" style="color:${colr(P.score)}">${P.score}</span></div>${Object.entries(P.parts).map(([k, v]) => bar(k, v)).join('')}</div>`; };
  const legend = (types) => `<div class="syn-legend small">${types.map(([k, n]) => `<span><i style="background:${SYN_COL[k]}"></i>${n}</span>`).join('')}</div>`;
  const ipLinks = rep.links.filter(l => l.phase !== 'def'), oopLinks = rep.links.filter(l => l.phase === 'def');
  const byPhase = order.map(ph => [ph, rep.issues.filter(i => i.phase === ph)]).filter(([, v]) => v.length);
  return `<div class="syn">
    <div class="syn-top">${ring(rep.score, 72)}<div class="small">${L(`Sinergia entre papéis em todas as fases: se os papéis se servem e se cobrem uns aos outros — não só se cada jogador está no melhor papel. ${rep.boxN} chegam à área · ${rep.creators} criativo(s) · ${rep.rdN} ficam atrás · ${rep.outN} saída(s).`, `Role synergy across every phase: whether roles serve and cover each other — not just whether each player is in his best role. ${rep.boxN} arrive in the box · ${rep.creators} creator(s) · ${rep.rdN} stay back · ${rep.outN} outlet(s).`)}${players.length ? '' : `<br><span class="muted">${L('Sem plantel: avaliação só pela estrutura dos papéis.', 'No squad: assessed on role structure only.')}</span>`}</div></div>
    <div class="syn-phases">${order.map(card).join('')}</div>
    <div class="grid g2" style="margin-top:12px;align-items:start">
      <div><b class="small">${L('Com bola', 'In possession')} · ${esc(t.ipForm)}</b>${synergyMap(t, ipLinks, t.ipForm, 'ip')}${legend([['build', L('saída de bola', 'build-up')], ['chain', L('criador → finalizador', 'creator → finisher')], ['cross', L('cruzamento → alvo', 'cross → target')], ['front', L('dupla da frente', 'front pair')], ['flank', L('flanco', 'flank')], ['counter', L('contra-ataque', 'counter')]])}</div>
      <div><b class="small">${L('Sem bola', 'Out of possession')} · ${esc(t.oopForm)}</b>${synergyMap(t, oopLinks, t.oopForm, 'oop')}${legend([['press', L('pressiona → cobre', 'presses → covers')], ['flankdef', L('flanco: dobra / segura', 'flank: doubles / holds')]])}</div>
    </div>
    ${rep.links.length ? `<details class="small" style="margin-top:10px"><summary>${L('Ligações principais', 'Main links')} (${rep.links.length})</summary><ul class="clean">${rep.links.map(l => `<li><span class="chip">${esc(PH[l.phase])}</span> ${esc(l.why)} <span class="muted">(${Math.round(l.v * 100)})</span></li>`).join('')}</ul></details>` : ''}
    ${byPhase.length ? `<h4 style="margin:12px 0 4px">${L('O que está a falhar entre papéis', 'What is not working between roles')}</h4>${byPhase.map(([ph, list]) => `<div class="small muted" style="margin:8px 0 2px;text-transform:uppercase;letter-spacing:.05em">${esc(PH[ph])}</div>${list.map(i => `<div class="finding"><div class="sev warn">!</div><div><b class="small">${esc(i.t)}</b><div class="small" style="color:var(--accent)">→ ${esc(i.fix)}</div></div></div>`).join('')}`).join('')}`
      : `<p class="small" style="margin-top:10px">${L('Os papéis encaixam em todas as fases. 👍', 'The roles fit together in every phase. 👍')}</p>`}
  </div>`;
}
