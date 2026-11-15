// ============================================================
// Clips curtos da Bíblia (mesmo motor do campo animado, em pequeno):
//  • papel: o jogador do papel em destaque, com o percurso; o resto esbatido
//  • instrução: dois mini-campos lado a lado (opção mínima × máxima) na fase em que actua
// ============================================================
const CLIPS = new Map();
let CLIP_RAF = 0;

function clipModelRole(roleId) {
  const r = ROLE_BY_ID[roleId], d = demoTacticForRole(r), M = simModel(d.t);
  const phases = r.phase === 'IP' ? ['build', 'prog', 'final'] : ['final', 'loss', 'oop'];
  return { M, phases, focus: d.idx, slot: d.t.slots[d.idx], phase: r.phase };
}
function clipModelTI(k, v) {
  const t = newTactic('4-3-3 DM Wide', '4-1-4-1 DM Wide');
  // papéis de base neutros mas com largura e profundidade, para o efeito ser visível
  const set = (code, ip, oop) => { const s = t.slots.find(x => x.ipCode === code); if (s) { if (ip) s.ip = ip; if (oop) s.oop = oop; } };
  set('DL', 'wb_ip'); set('DR', 'wb_ip'); set('AML', 'if_ip'); set('AMR', 'w_ip'); set('MCL', 'chm', null); set('ST', 'cf_ip', 'tcf');
  t.ment = 'Balanced'; t.ti = {}; if (v !== TI_BY_KEY[k].def) t.ti[k] = v;
  const i = TI_PHASE[k] ?? 0, ph = PHASES[i].k, prev = PHASES[(i + PHASES.length - 1) % PHASES.length].k;
  return { M: simModel(t), phases: [prev, ph], focus: null, ti: k };
}
function clipSVG(id, model) {
  const pitch = `<rect width="110" height="72" fill="var(--pitch)"/>
    <g fill="none" stroke="var(--pitchline)" stroke-width=".35"><rect x="3" y="2" width="104" height="68"/><line x1="55" y1="2" x2="55" y2="70"/><circle cx="55" cy="36" r="9.2"/>
    <rect x="3" y="15.8" width="16.6" height="40.4"/><rect x="90.4" y="15.8" width="16.6" height="40.4"/></g>`;
  const P = model.M.P, f = model.focus;
  const trail = f != null ? `<polyline points="${model.phases.map(ph => `${X_(P[f].pos[ph].y)},${Y_(P[f].pos[ph].x)}`).join(' ')}" fill="none" stroke="#ffe27a" stroke-width=".55" stroke-dasharray="1.2 .8"/>` : '';
  return `<svg class="clip-svg" viewBox="0 0 110 72" role="img" aria-label="${L('Clip animado', 'Animated clip')}">${pitch}${trail}
    <g class="c-opp">${OPP_SHAPE.map(() => '<circle r="1.25" fill="rgba(235,235,235,.38)"/>').join('')}</g>
    <g class="c-us">${P.map((p, i) => `<g class="c-p" opacity="${f == null || i === f ? 1 : .32}"><circle r="${i === f ? 2.6 : 2.1}" fill="${i === f ? '#0b5d3a' : '#123b29'}" stroke="${i === f ? '#ffe27a' : '#fff'}" stroke-width="${i === f ? .6 : .3}"/><text y=".7" text-anchor="middle" font-size="${i === f ? 1.8 : 1.5}" font-weight="700" fill="#fff"></text></g>`).join('')}</g>
    <circle class="c-ball" r=".9" fill="#fff" stroke="#111" stroke-width=".25"/>
    <text class="c-phase" x="5.5" y="68" font-size="2.6" font-weight="700" fill="rgba(255,255,255,.8)"></text></svg>`;
}
// HTML de um clip (o modelo é calculado só quando o clip é mostrado)
function clipHTML(id, caption) { return `<figure class="clip" data-clip="${esc(id)}"><div class="clip-box"></div>${caption ? `<figcaption class="small">${caption}</figcaption>` : ''}</figure>`; }

function initClips() {
  document.querySelectorAll('[data-clip]').forEach(el => {
    if (el._clip) return;
    const id = el.dataset.clip, [kind, a, b] = id.split(':');
    let model; try { model = kind === 'role' ? clipModelRole(a) : clipModelTI(a, +b); } catch (e) { el.querySelector('.clip-box').textContent = '—'; return; }
    el.querySelector('.clip-box').innerHTML = clipSVG(id, model);
    el._clip = { model, t0: performance.now() };
    CLIPS.set(el, el._clip);
  });
  if (CLIPS.size && !CLIP_RAF) CLIP_RAF = requestAnimationFrame(clipTick);
}
function clipTick(now) {
  CLIP_RAF = 0;
  const reduce = matchMedia('(prefers-reduced-motion: reduce)').matches;
  for (const [el, c] of CLIPS) {
    if (!el.isConnected) { CLIPS.delete(el); continue; }
    const ph = c.model.phases, M = c.model.M, SEG = 1.6, HOLD = 0.7, per = SEG + HOLD;
    const total = per * ph.length, tt = reduce ? (ph.length - 1) * per + SEG : (Math.max(0, now - c.t0) / 1000) % total;   // o 1.º fotograma pode vir antes de t0
    const i = Math.floor(tt / per), within = tt - i * per, from = ph[(i + ph.length - 1) % ph.length], to = ph[i];
    const u = i === 0 && !reduce ? Math.min(1, within / SEG) : Math.min(1, within / SEG);
    const e = ease(u), lerp = (a, b) => a + (b - a) * e;
    const svg = el.querySelector('svg'); if (!svg) continue;
    svg.querySelectorAll('.c-p').forEach((g, k) => {
      const p = M.P[k], A0 = p.pos[from], B0 = p.pos[to];
      g.setAttribute('transform', `translate(${X_(lerp(A0.y, B0.y)).toFixed(2)},${Y_(lerp(A0.x, B0.x)).toFixed(2)})`);
      const oopish = ['oop', 'loss'].includes(to);
      g.querySelector('text').textContent = (oopish ? p.oop : p.ip) ? (oopish ? p.oop.ab : p.ip.ab) : '';
    });
    svg.querySelectorAll('.c-opp circle').forEach((cc, k) => { const a0 = M.opp[from][k], b0 = M.opp[to][k]; cc.setAttribute('cx', X_(lerp(a0.y, b0.y)).toFixed(2)); cc.setAttribute('cy', Y_(lerp(a0.x, b0.x)).toFixed(2)); });
    const bp = M.ball[to], bu = Math.min(1, Math.max(0, (within - SEG * 0.25) / (SEG * 0.75))), seg = bp.length - 1, fi = bu * seg, j = Math.min(seg - 1, Math.floor(fi)), r = ease(fi - j);
    const b0 = bp[j], b1 = bp[j + 1] || bp[j], ball = svg.querySelector('.c-ball');
    ball.setAttribute('cx', X_(b0.y + (b1.y - b0.y) * r).toFixed(2)); ball.setAttribute('cy', Y_(b0.x + (b1.x - b0.x) * r).toFixed(2));
    svg.querySelector('.c-phase').textContent = T(PHASES.find(x => x.k === to).n);
  }
  if (CLIPS.size && !reduceAll()) CLIP_RAF = requestAnimationFrame(clipTick);
}
function stopClips() { CLIPS.clear(); cancelAnimationFrame(CLIP_RAF); CLIP_RAF = 0;
}
const reduceAll = () => matchMedia('(prefers-reduced-motion: reduce)').matches;
