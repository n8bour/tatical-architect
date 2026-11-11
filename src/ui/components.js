// ============================================================
// Componentes reutilizáveis
// ============================================================
function ring(score, size = 96) {
  const r = 40, c = 2 * Math.PI * r, off = c * (1 - score / 100);
  return `<svg class="ring" viewBox="0 0 100 100" width="${size}" height="${size}" role="img" aria-label="Pontuação ${score}">
    <circle cx="50" cy="50" r="${r}" fill="none" stroke="var(--panel2)" stroke-width="10"/>
    <circle cx="50" cy="50" r="${r}" fill="none" stroke="${fitColor(score)}" stroke-width="10" stroke-linecap="round" stroke-dasharray="${c}" stroke-dashoffset="${off}" transform="rotate(-90 50 50)"/>
    <text x="50" y="56" text-anchor="middle" font-size="24" font-weight="800" fill="var(--text)">${score}</text></svg>`;
}

function fitBadge(s) { return s == null ? '<span class="fit" style="background:var(--muted)">—</span>' : `<span class="fit" style="background:${fitColor(s)}">${s}</span>`; }

function srcTag(r) { return ATTR_VERIFIED.has(r.id) ? '<span class="tag v" title="Atributos-chave confirmados em fontes FM26">CONF</span>' : '<span class="tag e" title="Pesos estimados — confirmar no jogo">EST</span>'; }

function pitchSVG(t, mode, selIdx, opts = {}) {
  const X = (x) => 7 + x * 0.86, Y = (y) => 146 - y * 1.36;
  const pmap = Object.fromEntries(players().map(p => [p.id, p]));
  let g = '';
  t.slots.forEach((s, i) => {
    const a = slotInfo(t.ipForm, s.ipCode), b = slotInfo(t.oopForm, s.oopCode); if (!a || !b) return;
    const pos = mode === 'OOP' ? b : a;
    const role = ROLE_BY_ID[mode === 'OOP' ? s.oop : s.ip];
    const p = pmap[s.pid]; const sc = p ? slotScore(p, t, s) : null;
    const fill = p ? fitColor(sc) : (mode === 'OOP' ? '#2d5f8f' : 'rgba(0,0,0,.42)');
    if (mode === 'BOTH') g += `<line x1="${X(a.x)}" y1="${Y(a.y)}" x2="${X(b.x)}" y2="${Y(b.y)}" stroke="rgba(255,255,255,.5)" stroke-width=".5" stroke-dasharray="1.5 1"/>
      <circle cx="${X(b.x)}" cy="${Y(b.y)}" r="3.2" fill="none" stroke="#9cc9ff" stroke-width=".8"/>`;
    const sel = selIdx === i;
    const name = p ? esc(p.name.split(' ').slice(-1)[0]).slice(0, 12) : '';
    g += `<g class="slot" ${opts.static ? '' : `onclick="A.sel(${i})"`}>
      <circle class="dot" cx="${X(pos.x)}" cy="${Y(pos.y)}" r="${sel ? 6 : 5.2}" fill="${fill}" stroke="${sel ? '#ffe680' : 'rgba(255,255,255,.85)'}" stroke-width="${sel ? 1.2 : .5}"/>
      <text x="${X(pos.x)}" y="${Y(pos.y) + 1.2}" text-anchor="middle" font-size="3.1" font-weight="700" fill="#fff">${esc(role ? role.ab : '?')}</text>
      ${name ? `<text x="${X(pos.x)}" y="${Y(pos.y) + 9.3}" text-anchor="middle" font-size="3.1" fill="#fff" stroke="rgba(0,0,0,.55)" stroke-width=".6" paint-order="stroke">${name}</text>` : ''}
      ${sc != null ? `<text x="${X(pos.x) + 5.5}" y="${Y(pos.y) - 4}" font-size="2.6" fill="#fff" stroke="rgba(0,0,0,.6)" stroke-width=".5" paint-order="stroke">${sc}</text>` : ''}
    </g>`;
  });
  return `<svg class="pitch" viewBox="0 0 100 150" xmlns="http://www.w3.org/2000/svg">
    <rect width="100" height="150" fill="var(--pitch)"/>
    ${[0,1,2,3,4,5,6,7].map(k => k % 2 ? `<rect x="0" y="${5 + k * 17.5}" width="100" height="17.5" fill="var(--pitch2)"/>` : '').join('')}
    <g fill="none" stroke="var(--pitchline)" stroke-width=".45">
      <rect x="4" y="4" width="92" height="142"/><line x1="4" y1="75" x2="96" y2="75"/><circle cx="50" cy="75" r="9"/>
      <rect x="25" y="4" width="50" height="20"/><rect x="38" y="4" width="24" height="7"/>
      <rect x="25" y="126" width="50" height="20"/><rect x="38" y="139" width="24" height="7"/>
    </g>${g}</svg>`;
}

function selectHTML(opts, val, on) {
  return `<select onchange="${on}">${opts.map(([v, l]) => `<option value="${esc(v)}" ${String(v) === String(val) ? 'selected' : ''}>${esc(l)}</option>`).join('')}</select>`;
}

function attrChip(k, p) {
  const v = p && p.a[k] != null ? p.a[k] : null;
  const c = v == null ? '' : `style="background:${fitColor((v - 1) / 19 * 100)};color:#fff"`;
  return `<span class="chip" ${c}>${esc(attrName(k))}${v != null ? ' ' + Math.round(v) : ''}</span>`;
}

