const CLIPS = new Map();
let CLIP_RAF = 0, CLIP_IO = null;

function clipModelRole(roleId) {
  return roleClipModel(roleId);
}
function clipModelTI(k, v) {
  const t = newTactic('4-3-3 DM Wide', '4-1-4-1 DM Wide');
  const set = (code, ip, oop) => { const s = t.slots.find(x => x.ipCode === code); if (s) { if (ip) s.ip = ip; if (oop) s.oop = oop; } };
  set('DL', 'wb_ip', 'fb_oop'); set('DR', 'wb_ip', 'fb_oop');
  set('AML', 'if_ip', 'w_oop'); set('AMR', 'w_ip', 'w_oop');
  set('MCL', 'chm', 'cm_oop'); set('ST', 'cf_ip', 'tcf');
  t.ment = 'Balanced'; t.ti = {}; if (v !== TI_BY_KEY[k].def) t.ti[k] = v;
  const i = TI_PHASE[k] ?? 0;
  const phases = i === 0 ? ['build', 'prog'] : [PHASES[i - 1].k, PHASES[i].k];
  const tempoMul = k === 'tempo' ? [1.55, 1.25, 1, 0.78, 0.58][v] : 1;
  return { M: simModel(t), phases, focus: null, ti: k, tempoMul };
}

function clipHTML(id, caption) { return `<figure class="clip" data-clip="${esc(id)}"><div class="clip-box"></div>${caption ? `<figcaption class="small">${caption}</figcaption>` : ''}</figure>`; }

function initClips() {
  if (!CLIP_IO && typeof IntersectionObserver !== 'undefined') {
    // Pause off-screen canvases so opening many Bible clips stays fast.
    CLIP_IO = new IntersectionObserver((entries) => {
      entries.forEach(en => { const c = CLIPS.get(en.target); if (c) c.visible = en.isIntersecting; });
    }, { rootMargin: '120px' });
  }
  document.querySelectorAll('[data-clip-toggle]').forEach(tg => {
    if (tg._bound) return;
    tg._bound = true;
    const cid = tg.dataset.clipToggle;
    tg.addEventListener('pressedchange', () => A.clipToggle(cid));
    tg.addEventListener('click', () => { if (!customElements.get('ui-toggle')) A.clipToggle(cid); });
  });
  document.querySelectorAll('[data-clip]').forEach(el => {
    if (el._clip) return;
    const id = el.dataset.clip, [kind, a, b] = id.split(':');
    let model; try { model = kind === 'role' ? clipModelRole(a) : kind === 'concept' ? conceptClipModel(a) : kind === 'opp' ? oppClipModel(a) : clipModelTI(a, +b); } catch (e) { el.querySelector('.clip-box').textContent = '—'; return; }
    const box = el.querySelector('.clip-box');
    box.innerHTML = `<canvas class="clip-canvas" role="img" aria-label="${L('Clip animado', 'Animated clip')}"></canvas>`;
    const canvas = box.querySelector('canvas');
    el._clip = { model, canvas, t0: performance.now(), visible: true };
    CLIPS.set(el, el._clip);
    if (CLIP_IO) CLIP_IO.observe(el);
  });
  if (CLIPS.size && !CLIP_RAF) CLIP_RAF = requestAnimationFrame(clipTick);
}

function clipTick(now) {
  CLIP_RAF = 0;
  const reduce = matchMedia('(prefers-reduced-motion: reduce)').matches;
  for (const [el, c] of CLIPS) {
    if (!el.isConnected) { if (CLIP_IO) CLIP_IO.unobserve(el); CLIPS.delete(el); continue; }
    if (!c.visible && !reduce) continue;
    const M = c.model.M, ph = c.model.phases, mul = c.model.tempoMul || 1;
    const SEG = 1.45 * mul, HOLD = 0.85 * mul, per = SEG + HOLD;
    const total = per * ph.length;
    const tt = reduce ? (ph.length - 1) * per + SEG : (Math.max(0, now - c.t0) / 1000) % total;
    const i = Math.floor(tt / per), within = tt - i * per;
    const isReset = i === 0;
    const from = isReset ? ph[0] : ph[i - 1], to = ph[i];
    const moveU = isReset ? 1 : Math.min(1, within / (SEG * 0.46));
    const prevPath = !isReset && M.ball[from] ? M.ball[from] : null;
    const prevBallEnd = prevPath && prevPath.length ? prevPath[prevPath.length - 1] : null;
    const carryU = isReset ? 1 : moveU;
    const bu = isReset ? Math.min(1, Math.max(0, (within - SEG * 0.12) / (SEG * 0.85)))
      : Math.min(1, Math.max(0, (within - SEG * 0.46) / (SEG * 0.54)));
    const fadeIn = isReset && !reduce ? Math.min(1, Math.max(0.15, within / 0.25)) : 1;
    const oopish = c.model.isRoleClip ? c.model.phase === 'OOP'
      : c.model.phaseByStep ? c.model.phaseByStep[i] === 'OOP'
      : ['oop', 'loss'].includes(to);
    const lbl = c.model.labels ? c.model.labels[i] : T((PHASES.find(x => x.k === to) || {}).n || '');
    const f = c.model.focus;
    drawPitchFrame(c.canvas, {
      M, fromPh: from, toPh: to, um: ease(moveU), bu, carryU, prevBallEnd, isReset, fadeIn,
      oopish, showOpp: true, showTrails: false,
      focus: f, pmap: null, compact: true, phaseLabel: lbl,
      ghostPt: c.model.isRoleClip && f != null ? M.P[f].pos.s0 : null,
      pulse: (now % 1100) / 1100,
      pressFn: (p, k) => c.model.isRoleClip ? (k === f && M.pressAt && M.pressAt[to] && moveU > 0.5) : (to === 'oop' && p.pressing && moveU > 0.5),
    });
  }
  if (CLIPS.size && !reduceAll()) CLIP_RAF = requestAnimationFrame(clipTick);
}
function stopClips() {
  if (CLIP_IO) CLIP_IO.disconnect();
  CLIPS.clear(); cancelAnimationFrame(CLIP_RAF); CLIP_RAF = 0;
}
const reduceAll = () => matchMedia('(prefers-reduced-motion: reduce)').matches;
