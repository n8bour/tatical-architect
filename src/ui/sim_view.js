const SIM = { raf: 0, last: 0, model: null, ro: null, hits: [] };
const X_ = (y) => 3 + y * 1.04, Y_ = (x) => 2 + x * 0.68;

function simTactic() { return ST.sim && ST.sim.src === 'demo' ? ST.sim.t : ST.tactic; }
function simDefaults() { if (!ST.sim) ST.sim = { src: 'tactic', phase: 0, u: 0, playing: !matchMedia('(prefers-reduced-motion: reduce)').matches, speed: 1, opp: true, trails: false, focus: null, hl: null }; return ST.sim; }

function simBallState(path, bu) {
  if (!path || !path.length) return { x: 50, y: 50, z: 0, kind: 'pass' };
  if (path.length === 1) return { x: path[0].x, y: path[0].y, z: 0, kind: path[0].kind || 'pass' };
  const seg = path.length - 1, f = clamp(bu, 0, 1) * seg;
  const j = Math.min(seg - 1, Math.floor(f)), r = ease(f - j);
  const a = path[j], b = path[j + 1] || path[j];
  const kind = b.kind || a.kind || 'pass';
  const bx = a.x + (b.x - a.x) * r, by = a.y + (b.y - a.y) * r;
  const z = kind === 'loft' ? 4 * r * (1 - r) : kind === 'curve' ? 2.2 * r * (1 - r) : 0;
  if (kind === 'loft' || kind === 'curve') {
    const dx = b.y - a.y, dy = b.x - a.x, len = Math.hypot(dx, dy) || 1;
    const h = (kind === 'loft' ? 7 : 4.5) * 4 * r * (1 - r);
    return { x: clamp(bx + (dx / len) * h * 0.45, 2, 98), y: clamp(by - (dy / len) * h * 0.3, 1, 99), z, kind };
  }
  return { x: bx, y: by, z, kind };
}

function fitPitchCanvas(canvas, fallbackW = 660) {
  const w = canvas.clientWidth || fallbackW;
  const h = Math.max(1, Math.round(w * 72 / 110));
  const dpr = Math.min(window.devicePixelRatio || 1, 2.5);
  const bw = Math.round(w * dpr), bh = Math.round(h * dpr);
  if (canvas.width !== bw || canvas.height !== bh) { canvas.width = bw; canvas.height = bh; }
  const ctx = canvas.getContext('2d');
  ctx.setTransform(bw / 110, 0, 0, bh / 72, 0, 0);
  return ctx;
}

function canvasArrow(ctx, a, b, col, width, dash, bend, alpha = 0.9) {
  const x1 = X_(a.y), y1 = Y_(a.x), x2 = X_(b.y), y2 = Y_(b.x);
  const dx = x2 - x1, dy = y2 - y1, len = Math.hypot(dx, dy);
  if (len < 1.2) return;
  let cx = (x1 + x2) / 2, cy = (y1 + y2) / 2;
  if (bend) { cx -= (dy / len) * bend; cy += (dx / len) * bend; }
  ctx.save();
  ctx.globalAlpha = alpha;
  ctx.strokeStyle = col;
  ctx.fillStyle = col;
  ctx.lineWidth = width;
  ctx.lineCap = 'round';
  ctx.setLineDash(dash || []);
  ctx.beginPath();
  ctx.moveTo(x1, y1);
  if (bend) ctx.quadraticCurveTo(cx, cy, x2, y2); else ctx.lineTo(x2, y2);
  ctx.stroke();
  ctx.setLineDash([]);
  const ang = Math.atan2(y2 - (bend ? cy : y1), x2 - (bend ? cx : x1));
  const hs = Math.max(1.5, width * 3.1);
  ctx.beginPath();
  ctx.moveTo(x2, y2);
  ctx.lineTo(x2 - hs * Math.cos(ang - 0.48), y2 - hs * Math.sin(ang - 0.48));
  ctx.lineTo(x2 - hs * Math.cos(ang + 0.48), y2 - hs * Math.sin(ang + 0.48));
  ctx.closePath();
  ctx.fill();
  ctx.restore();
}

function canvasPill(ctx, text, x, y, fg, bg, size = 2.0) {
  if (!text) return;
  ctx.save();
  ctx.font = `700 ${size}px "Segoe UI",system-ui,sans-serif`;
  const w = ctx.measureText(text).width + 2.2, h = size + 1.4;
  const rx = clamp(x - w / 2, 3.6, 106.4 - w), ry = clamp(y - h / 2, 2.6, 69.4 - h);
  ctx.fillStyle = bg;
  ctx.beginPath();
  ctx.roundRect(rx, ry, w, h, 1);
  ctx.fill();
  ctx.fillStyle = fg;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(text, rx + w / 2, ry + h / 2 + 0.1);
  ctx.restore();
}

function drawPitchFrame(canvas, opts) {
  const { M, fromPh, toPh, um, bu, carryU, prevBallEnd, isReset, fadeIn, oopish, showOpp, showTrails, focus, pmap, compact, phaseLabel, ghostPt, pulse, pressFn, hits } = opts;
  const ctx = fitPitchCanvas(canvas, compact ? 360 : 680);
  const cs = getComputedStyle(document.documentElement);
  const pitch1 = cs.getPropertyValue('--pitch').trim() || '#2c5a36';
  const pitch2 = cs.getPropertyValue('--pitch2').trim() || '#275131';
  const pline = cs.getPropertyValue('--pitchline').trim() || 'rgba(255,255,255,.65)';
  const moveU = isReset ? 1 : um;
  const sceneAlpha = fadeIn != null ? clamp(fadeIn, 0, 1) : 1;

  ctx.fillStyle = pitch1;
  ctx.fillRect(0, 0, 110, 72);
  ctx.fillStyle = pitch2;
  for (let k = 1; k < 10; k += 2) ctx.fillRect(3 + k * 10.4, 2, 10.4, 68);
  const vig = ctx.createRadialGradient(55, 36, 20, 55, 36, 68);
  vig.addColorStop(0, 'rgba(255,255,255,.04)');
  vig.addColorStop(1, 'rgba(0,0,0,.2)');
  ctx.fillStyle = vig;
  ctx.fillRect(0, 0, 110, 72);

  ctx.save();
  ctx.strokeStyle = pline;
  ctx.fillStyle = pline;
  ctx.lineWidth = 0.36;
  ctx.strokeRect(3, 2, 104, 68);
  ctx.beginPath();
  ctx.moveTo(55, 2); ctx.lineTo(55, 70);
  ctx.moveTo(64.2, 36); ctx.arc(55, 36, 9.2, 0, Math.PI * 2);
  ctx.stroke();
  ctx.beginPath(); ctx.arc(55, 36, 0.55, 0, Math.PI * 2); ctx.fill();
  ctx.strokeRect(3, 15.8, 16.6, 40.4); ctx.strokeRect(3, 26.8, 5.6, 18.4);
  ctx.strokeRect(90.4, 15.8, 16.6, 40.4); ctx.strokeRect(101.4, 26.8, 5.6, 18.4);
  ctx.beginPath(); ctx.arc(14.5, 36, 0.45, 0, Math.PI * 2); ctx.arc(95.5, 36, 0.45, 0, Math.PI * 2); ctx.fill();
  ctx.beginPath(); ctx.arc(14.5, 36, 8.2, -0.65, 0.65); ctx.stroke();
  ctx.beginPath(); ctx.arc(95.5, 36, 8.2, Math.PI - 0.65, Math.PI + 0.65); ctx.stroke();
  ctx.fillStyle = 'rgba(255,255,255,.12)';
  ctx.fillRect(1.3, 32.4, 1.7, 7.2); ctx.fillRect(107, 32.4, 1.7, 7.2);
  ctx.lineWidth = 0.65;
  ctx.strokeRect(1.3, 32.4, 1.7, 7.2); ctx.strokeRect(107, 32.4, 1.7, 7.2);
  ctx.restore();

  if (!compact) {
    ctx.save();
    ctx.font = '600 2.1px "Segoe UI",system-ui,sans-serif';
    ctx.fillStyle = 'rgba(255,255,255,.55)';
    ctx.textAlign = 'left'; ctx.fillText(L('a tua baliza', 'your goal'), 5, 68.5);
    ctx.textAlign = 'right'; ctx.fillText(L('ataque →', 'attack →'), 105, 68.5);
    ctx.restore();
  }

  if (toPh === 'oop' && M.overlays && M.overlays.length) {
    M.overlays.forEach(o => {
      const x1 = X_(o.y1), x2 = X_(o.y2); if (x2 <= x1) return;
      ctx.save();
      ctx.fillStyle = o.kind === 'gap' ? 'rgba(255,107,90,.18)' : 'rgba(255,200,80,.16)';
      ctx.strokeStyle = o.kind === 'gap' ? 'rgba(255,107,90,.75)' : 'rgba(255,200,80,.75)';
      ctx.lineWidth = 0.35; ctx.setLineDash([1.2, 0.9]);
      ctx.fillRect(x1, 2, x2 - x1, 68); ctx.strokeRect(x1, 2, x2 - x1, 68);
      ctx.restore();
      canvasPill(ctx, o.label, (x1 + x2) / 2, 5.4, '#fff', 'rgba(20,18,16,.72)', 1.9);
    });
  }

  ((M.zones && M.zones[toPh]) || []).forEach(z => {
    ctx.save();
    if (z.kind === 'circle') {
      const cx = X_(z.y), cy = Y_(z.x), r = (z.r || 12) * 0.72;
      const g = ctx.createRadialGradient(cx, cy, 1, cx, cy, r);
      g.addColorStop(0, z.color); g.addColorStop(1, 'rgba(0,0,0,0)');
      ctx.fillStyle = g;
      ctx.beginPath(); ctx.arc(cx, cy, r, 0, Math.PI * 2); ctx.fill();
      ctx.strokeStyle = z.stroke; ctx.lineWidth = 0.4; ctx.setLineDash([1.5, 1]);
      ctx.beginPath(); ctx.arc(cx, cy, r, 0, Math.PI * 2); ctx.stroke();
      ctx.restore();
      if (z.label) canvasPill(ctx, z.label, cx, cy - r - 1.8, '#fff', 'rgba(20,18,16,.75)', 1.9);
    } else {
      const x1 = X_(z.y1), x2 = X_(z.y2), y1 = Y_(z.x1), y2 = Y_(z.x2);
      ctx.fillStyle = z.color; ctx.strokeStyle = z.stroke; ctx.lineWidth = 0.38; ctx.setLineDash([1.5, 1]);
      ctx.beginPath(); ctx.roundRect(x1, y1, Math.max(1, x2 - x1), Math.max(1, y2 - y1), 1.6);
      ctx.fill(); ctx.stroke();
      ctx.restore();
      if (z.label) canvasPill(ctx, z.label, (x1 + x2) / 2, y1 + 2.8, '#fff', 'rgba(20,18,16,.75)', 1.9);
    }
  });

  ((M.lines && M.lines[toPh]) || []).forEach(ln => {
    const x = X_(ln.y);
    ctx.save();
    ctx.strokeStyle = ln.color; ctx.lineWidth = 0.48; ctx.setLineDash([2, 1.3]);
    ctx.beginPath(); ctx.moveTo(x, 2); ctx.lineTo(x, 70); ctx.stroke();
    ctx.restore();
    if (ln.label && !compact) canvasPill(ctx, ln.label, x, 5.2, ln.color, 'rgba(20,18,16,.78)', 1.85);
  });

  if (showTrails || focus != null) {
    M.P.forEach(p => {
      if (!showTrails && p.i !== focus) return;
      const keys = Object.keys(p.pos);
      if (keys.length < 2) return;
      ctx.save();
      ctx.strokeStyle = p.i === focus ? 'rgba(255,226,122,.72)' : 'rgba(255,255,255,.26)';
      ctx.lineWidth = p.i === focus ? 0.52 : 0.28;
      ctx.setLineDash([1.3, 0.9]);
      ctx.beginPath();
      keys.forEach((k, idx) => { const px = X_(p.pos[k].y), py = Y_(p.pos[k].x); if (idx === 0) ctx.moveTo(px, py); else ctx.lineTo(px, py); });
      if (!compact && keys.length === PHASES.length) ctx.closePath();
      ctx.stroke();
      ctx.restore();
    });
  }

  if (ghostPt) {
    ctx.save();
    ctx.strokeStyle = 'rgba(255,226,122,.72)'; ctx.lineWidth = 0.48; ctx.setLineDash([1.1, 0.8]);
    ctx.beginPath(); ctx.arc(X_(ghostPt.y), Y_(ghostPt.x), 2.35, 0, Math.PI * 2); ctx.stroke();
    ctx.restore();
  }

  ((M.arrows && M.arrows[toPh]) || []).forEach(ar => {
    if (!ar.from || !ar.to) return;
    canvasArrow(ctx, ar.from, ar.to, ar.color, compact ? 0.62 : 0.56, ar.kind === 'cover' ? [1.4, 0.9] : null, 0, ar.dim ? 0.35 : 0.88);
  });

  const bp = (M.ball && M.ball[toPh]) || [];
  const fullBall = !isReset && prevBallEnd && bp.length && Math.hypot(prevBallEnd.x - bp[0].x, prevBallEnd.y - bp[0].y) > 1.5
    ? { carryFrom: prevBallEnd, carryTo: bp[0] } : null;
  ctx.save();
  ctx.globalAlpha = sceneAlpha;
  for (let i = 0; i < bp.length - 1; i++) {
    const a = bp[i], b = bp[i + 1];
    const kind = b.kind || a.kind || 'pass';
    const col = kind === 'shot' ? '#ff9b42' : kind === 'dribble' ? '#ffe27a' : kind === 'opp' ? 'rgba(235,235,235,.7)' : 'rgba(255,255,255,.9)';
    const dash = kind === 'loft' ? [2.1, 1.3] : kind === 'dribble' ? [0.8, 0.9] : kind === 'opp' ? [1.5, 1.1] : null;
    const bend = kind === 'loft' ? -7 : kind === 'curve' ? -4.5 : 0;
    canvasArrow(ctx, a, b, col, kind === 'shot' ? 0.72 : kind === 'opp' ? 0.44 : 0.52, dash, bend, kind === 'opp' ? 0.75 : 0.92);
  }

  let ballNow;
  if (fullBall && carryU != null && carryU < 1) {
    const cu = ease(clamp(carryU, 0, 1));
    ballNow = {
      x: fullBall.carryFrom.x + (fullBall.carryTo.x - fullBall.carryFrom.x) * cu,
      y: fullBall.carryFrom.y + (fullBall.carryTo.y - fullBall.carryFrom.y) * cu,
      z: 0, kind: 'pass',
    };
  } else {
    ballNow = simBallState(bp, bu);
  }
  const bxPx = X_(ballNow.y), byPx = Y_(ballNow.x);

  if (showOpp && M.opp && M.opp[fromPh] && M.opp[toPh]) {
    M.opp[fromPh].forEach((a, k) => {
      const b = M.opp[toPh][k];
      const ox = X_(a.y + (b.y - a.y) * moveU), oy = Y_(a.x + (b.x - a.x) * moveU);
      ctx.save();
      ctx.fillStyle = 'rgba(0,0,0,.28)';
      ctx.beginPath(); ctx.ellipse(ox + 0.3, oy + 0.65, 1.5, 0.95, 0, 0, Math.PI * 2); ctx.fill();
      ctx.fillStyle = 'rgba(226,232,240,.78)';
      ctx.strokeStyle = 'rgba(15,23,42,.55)';
      ctx.lineWidth = 0.28;
      ctx.beginPath(); ctx.arc(ox, oy, compact ? 1.4 : 1.48, 0, Math.PI * 2); ctx.fill(); ctx.stroke();
      ctx.restore();
    });
  }

  if (hits) hits.length = 0;
  M.P.forEach((p, i) => {
    const a = p.pos[fromPh], b = p.pos[toPh];
    const x = a.x + (b.x - a.x) * moveU, y = a.y + (b.y - a.y) * moveU;
    const px = X_(y), py = Y_(x);
    const isF = focus === i, isGK = p.s.ipCode === 'GK';
    const r = isF ? 2.75 : compact ? 2.15 : 2.3;
    if (hits) hits.push({ i, px, py, r: r + 1.4 });
    const dim = focus != null && compact && !isF;

    ctx.save();
    if (dim) ctx.globalAlpha = 0.42;

    if (isF || (showTrails && !compact)) {
      const ax = X_(a.y), ay = Y_(a.x);
      if (Math.hypot(px - ax, py - ay) > 1.2) {
        const g = ctx.createLinearGradient(ax, ay, px, py);
        g.addColorStop(0, 'rgba(255,226,122,0)');
        g.addColorStop(1, isF ? 'rgba(255,226,122,.55)' : 'rgba(255,255,255,.25)');
        ctx.strokeStyle = g; ctx.lineWidth = isF ? 1.6 : 0.9; ctx.lineCap = 'round';
        ctx.beginPath(); ctx.moveTo(ax, ay); ctx.lineTo(px, py); ctx.stroke();
      }
    }

    if (pressFn && pressFn(p, i)) {
      const pr = r + 1.1 + 1.1 * pulse;
      ctx.strokeStyle = `rgba(255,107,90,${(0.95 - pulse * 0.6).toFixed(2)})`;
      ctx.lineWidth = 0.55;
      ctx.beginPath(); ctx.arc(px, py, pr, 0, Math.PI * 2); ctx.stroke();
    }

    ctx.fillStyle = 'rgba(0,0,0,.36)';
    ctx.beginPath(); ctx.ellipse(px + 0.35, py + 0.85, r * 0.96, r * 0.62, 0, 0, Math.PI * 2); ctx.fill();

    const mdx = X_(b.y) - X_(a.y), mdy = Y_(b.x) - Y_(a.x);
    const ang = Math.hypot(mdx, mdy) > 1 ? Math.atan2(mdy, mdx) : Math.atan2(byPx - py, bxPx - px);
    ctx.fillStyle = isF ? '#ffe27a' : 'rgba(255,255,255,.85)';
    ctx.beginPath();
    ctx.moveTo(px + Math.cos(ang) * (r + 0.85), py + Math.sin(ang) * (r + 0.85));
    ctx.lineTo(px + Math.cos(ang + 0.5) * (r - 0.1), py + Math.sin(ang + 0.5) * (r - 0.1));
    ctx.lineTo(px + Math.cos(ang - 0.5) * (r - 0.1), py + Math.sin(ang - 0.5) * (r - 0.1));
    ctx.closePath(); ctx.fill();

    if (isF) {
      ctx.strokeStyle = '#ffe27a'; ctx.lineWidth = 0.65;
      ctx.beginPath(); ctx.arc(px, py, r + 0.95, 0, Math.PI * 2); ctx.stroke();
    }

    ctx.fillStyle = isGK ? '#464a52' : oopish ? (isF ? '#2563eb' : '#1d416b') : (isF ? '#0f8a52' : '#135737');
    ctx.strokeStyle = isGK ? '#facc15' : isF ? '#ffe27a' : '#fff';
    ctx.lineWidth = isF ? 0.5 : 0.36;
    ctx.beginPath(); ctx.arc(px, py, r, 0, Math.PI * 2); ctx.fill(); ctx.stroke();

    const role = oopish ? p.oop : p.ip;
    const ab = role ? role.ab : '?';
    ctx.fillStyle = '#fff';
    ctx.font = `700 ${isF ? 1.85 : compact ? 1.48 : 1.6}px "Segoe UI",system-ui,sans-serif`;
    ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
    ctx.fillText(ab, px, py + 0.1);

    if (!compact && pmap && pmap[p.s.pid]) {
      const nm = surname(pmap[p.s.pid].name).slice(0, 11);
      ctx.font = '600 1.65px "Segoe UI",system-ui,sans-serif';
      ctx.lineWidth = 0.55; ctx.strokeStyle = 'rgba(0,0,0,.72)';
      ctx.strokeText(nm, px, py + r + 2.2);
      ctx.fillStyle = '#fff'; ctx.fillText(nm, px, py + r + 2.2);
    }
    ctx.restore();
  });

  const isCarrying = !!(fullBall && carryU != null && carryU < 1);
  if (!isCarrying && bp.length >= 2 && bu > 0.01) {
    const prevB = simBallState(bp, Math.max(0, bu - 0.14));
    const pbx = X_(prevB.y), pby = Y_(prevB.x) - prevB.z * 3.1;
    const nby = byPx - ballNow.z * 3.1;
    if (Math.hypot(bxPx - pbx, nby - pby) > 0.8) {
      const bg = ctx.createLinearGradient(pbx, pby, bxPx, nby);
      bg.addColorStop(0, 'rgba(255,255,255,0)');
      bg.addColorStop(1, ballNow.kind === 'shot' ? 'rgba(255,155,66,.7)' : 'rgba(255,255,255,.55)');
      ctx.save(); ctx.strokeStyle = bg; ctx.lineWidth = 1.1; ctx.lineCap = 'round';
      ctx.beginPath(); ctx.moveTo(pbx, pby); ctx.lineTo(bxPx, nby); ctx.stroke(); ctx.restore();
    }
  }
  const br = 1.0 + ballNow.z * 0.45;
  ctx.save();
  ctx.fillStyle = `rgba(0,0,0,${(0.38 - ballNow.z * 0.18).toFixed(2)})`;
  ctx.beginPath(); ctx.ellipse(bxPx + 0.35 + ballNow.z * 0.7, byPx + 0.55 + ballNow.z * 0.45, 1.05 + ballNow.z * 0.25, 0.65 + ballNow.z * 0.1, 0, 0, Math.PI * 2); ctx.fill();
  const ballY = byPx - ballNow.z * 3.1;
  ctx.fillStyle = '#fff'; ctx.strokeStyle = '#111'; ctx.lineWidth = 0.28;
  ctx.beginPath(); ctx.arc(bxPx, ballY, br, 0, Math.PI * 2); ctx.fill(); ctx.stroke();
  ctx.fillStyle = '#1f2937';
  ctx.beginPath(); ctx.arc(bxPx + br * 0.22, ballY - br * 0.18, br * 0.32, 0, Math.PI * 2); ctx.fill();
  ctx.restore();
  ctx.restore();

  if (phaseLabel) canvasPill(ctx, phaseLabel, 18, 67.2, '#fff', 'rgba(20,18,16,.76)', 2.15);
}

function vSim() {
  const sim = simDefaults(), t = simTactic();
  const forms = Object.keys(FORMATIONS).map(f => [f, f]);
  const ctl = (k) => { const d = TI_BY_KEY[k]; return `<ui-field class="${sim.hl === k ? 'hl' : ''}"><ui-field-label>${esc(d.n)}</ui-field-label>${selectHTML(d.opts.map((o, i) => [i, o]), tiVal(t, k), `A.simTI('${k}',this.value)`)}</ui-field>`; };
  const fr = sim.focus != null ? t.slots[sim.focus] : null;
  const fRole = fr ? [ROLE_BY_ID[fr.ip], ROLE_BY_ID[fr.oop]] : null;
  return `<div class="top"><div><h2>${sim.src === 'demo' ? esc(sim.title) : '<span>' + L('Campo animado', 'Animated pitch') + '</span> — ' + esc(tName(t))}</h2>
      <p class="muted small" style="margin:0">${esc(t.ipForm)} <span>com bola</span> → ${esc(t.oopForm)} <span>sem bola</span> · ${esc(t.ment)}. <span>Clique num jogador para o seguir.</span></p></div>
    <div class="row">${sim.src === 'demo'
      ? `<ui-button id="sim-back"><button type="button" class="btn">${L('Voltar à minha tática', 'Back to my tactic')}</button></ui-button>`
      : `<ui-button id="sim-edit"><button type="button" class="btn">${L('Editar no analisador', 'Edit in analyser')}</button></ui-button>`}</div></div>
  <div class="simgrid">
    <div>
      <div class="simwrap"><canvas class="simpitch" id="sim-canvas" role="img" aria-label="${L('Campo animado da tática', 'Animated tactic pitch')}"></canvas></div>
      <div class="simlegend small muted">
        <span><i style="background:#fff"></i>${L('Passe / bola', 'Pass / ball')}</span>
        <span><i style="background:#ffe27a"></i>${L('Desmarcação / contra-ataque', 'Attacking run / counter')}</span>
        <span><i style="background:#ff6b5a"></i>${L('Pressão / subida', 'Press / step up')}</span>
        <span><i style="background:#7dd3fc"></i>${L('Cobertura / recuo', 'Cover / drop')}</span>
      </div>
      <div class="simbar">
        <div class="simbar-row">
          <ui-toggle id="sim-play-tg"${sim.playing ? ' default-pressed' : ''}>
            <button type="button" id="sim-play">${L('Reproduzir', 'Play')}</button>
          </ui-toggle>
          <ui-segmented-control id="sim-speed" aria-label="${L('Velocidade', 'Playback speed')}" default-value="${sim.speed}">
            <ui-segmented-control-indicator></ui-segmented-control-indicator>
            ${[0.5, 1, 2].map(s => `<ui-segmented-control-item value="${s}">${s}×</ui-segmented-control-item>`).join('')}
          </ui-segmented-control>
        </div>
        <ui-segmented-control id="sim-phases" class="phases" aria-label="${L('Fase do jogo', 'Match phase')}" default-value="${sim.phase}">
          <ui-segmented-control-indicator></ui-segmented-control-indicator>
          ${PHASES.map((p, i) => `<ui-segmented-control-item class="ph ${p.ph}${i === sim.phase ? ' on' : ''}" data-ph="${i}" value="${i}"><span>${p.n}</span><i></i></ui-segmented-control-item>`).join('')}
        </ui-segmented-control>
      </div>
      <div class="panel simcap" id="sim-cap"></div>
    </div>
    <div class="grid simside">
      ${fr ? `<div class="panel"><h3>${esc(fr.ipCode)} → ${esc(fr.oopCode)}</h3>
        ${fRole.map(r => r ? `<p class="small"><span class="chip ${r.phase === 'IP' ? 'ip' : 'oop'}">${r.phase}</span> <b>${esc(roleName(r))}</b> — ${esc(r.desc)}</p>` : '').join('')}
        ${sim.src === 'demo' ? '' : `<div class="grid g2">
          <ui-field><ui-field-label>${L('Papel com bola', 'In possession role')}</ui-field-label>${selectHTML(rolesFor(slotInfo(t.ipForm, fr.ipCode).group, 'IP', slotInfo(t.ipForm, fr.ipCode)).map(r => [r.id, roleName(r)]), fr.ip, `A.simRole('IP',this.value)`)}</ui-field>
          <ui-field><ui-field-label>${L('Papel sem bola', 'Out of possession role')}</ui-field-label>${selectHTML(rolesFor(slotInfo(t.oopForm, fr.oopCode).group, 'OOP', slotInfo(t.oopForm, fr.oopCode)).map(r => [r.id, roleName(r)]), fr.oop, `A.simRole('OOP',this.value)`)}</ui-field></div>`}
        <ui-button id="sim-unfocus"><button type="button" class="btn sm">${L('Deixar de seguir', 'Stop following')}</button></ui-button></div>` : ''}
      <div class="panel grid">
        <h3>Experimente as instruções</h3>
        <p class="small muted" style="margin:0">${sim.src === 'demo' ? L('Cada mudança redesenha o movimento na hora (só nesta demonstração).', 'Every change redraws the movement instantly (in this demo only).') : L('Cada mudança redesenha o movimento na hora e fica gravada na tática.', 'Every change redraws the movement instantly and is saved to the tactic.')}</p>
        <ui-field><ui-field-label>${L('Mentalidade', 'Mentality')}</ui-field-label>${selectHTML(MENTALITIES.map(m => [m, m]), t.ment, 'A.simMent(this.value)')}</ui-field>
        ${sim.src === 'demo' ? '' : `<div class="grid g2"><ui-field><ui-field-label>${L('Formação com bola', 'In possession formation')}</ui-field-label>${selectHTML(forms, t.ipForm, "A.setForm('IP',this.value)")}</ui-field><ui-field><ui-field-label>${L('Sem bola', 'Out of possession')}</ui-field-label>${selectHTML(forms, t.oopForm, "A.setForm('OOP',this.value)")}</ui-field></div>`}
        <ui-accordion id="sim-acc" multiple default-value="ip,oop" aria-label="${L('Instruções de equipa', 'Team instructions')}">
          <ui-accordion-item value="ip">
            <ui-accordion-header><h4><ui-accordion-trigger><button type="button" aria-expanded="true">${L('Com bola', 'In possession')}</button></ui-accordion-trigger></h4></ui-accordion-header>
            <ui-accordion-panel><div class="grid g2" style="margin-top:6px">${['buildup','goal_kicks','gk_target','passing','tempo','reception','width','support_runs','dribbling','progress','patience','long_shots','crossing','creative','att_trans'].map(ctl).join('')}</div></ui-accordion-panel>
          </ui-accordion-item>
          <ui-accordion-item value="oop">
            <ui-accordion-header><h4><ui-accordion-trigger><button type="button" aria-expanded="true">${L('Sem bola', 'Out of possession')}</ui-accordion-trigger></h4></ui-accordion-header>
            <ui-accordion-panel><div class="grid g2" style="margin-top:6px">${['loe','dline','dl_behaviour','trigger','press_trap','cross_eng','short_gk','def_trans'].map(ctl).join('')}</div></ui-accordion-panel>
          </ui-accordion-item>
        </ui-accordion>
        <div class="sim-switches">
          <ui-field class="sim-sw-field"><ui-switch id="sim-sw-opp"${sim.opp ? ' default-checked' : ''}><ui-switch-thumb></ui-switch-thumb></ui-switch><ui-field-label>${L('Adversário', 'Opponent')}</ui-field-label></ui-field>
          <ui-field class="sim-sw-field"><ui-switch id="sim-sw-trails"${sim.trails ? ' default-checked' : ''}><ui-switch-thumb></ui-switch-thumb></ui-switch><ui-field-label>${L('Percursos', 'Trails')}</ui-field-label></ui-field>
        </div>
      </div>
      <p class="small muted">O campo é um modelo simplificado: mostra a intenção de cada papel e instrução, não reproduz o motor de jogo do FM27.</p>
    </div>
  </div>`;
}

function simBindUI(canvas) {
  const playTg = document.getElementById('sim-play-tg');
  if (playTg && !playTg._bound) {
    playTg._bound = true;
    playTg.addEventListener('pressedchange', (e) => A.simPlay(!!e.detail.value));
    playTg.addEventListener('click', () => { if (!customElements.get('ui-toggle')) A.simPlay(); });
  }
  const spd = document.getElementById('sim-speed');
  if (spd && !spd._bound) {
    spd._bound = true;
    spd.addEventListener('valuechange', (e) => { if (e.detail.value != null) A.simSpeed(+e.detail.value); });
    spd.addEventListener('click', (e) => {
      if (customElements.get('ui-segmented-control')) return;
      const it = e.target.closest('ui-segmented-control-item'); if (it) A.simSpeed(+it.getAttribute('value'));
    });
  }
  const phs = document.getElementById('sim-phases');
  if (phs && !phs._bound) {
    phs._bound = true;
    phs.addEventListener('valuechange', (e) => { if (e.detail.value != null) A.simPhase(+e.detail.value); });
    phs.addEventListener('click', (e) => {
      if (customElements.get('ui-segmented-control')) return;
      const it = e.target.closest('ui-segmented-control-item'); if (it) A.simPhase(+it.getAttribute('value'));
    });
  }
  const swOpp = document.getElementById('sim-sw-opp');
  if (swOpp && !swOpp._bound) {
    swOpp._bound = true;
    swOpp.addEventListener('checkedchange', (e) => A.simToggle('opp', !!e.detail.value));
    swOpp.addEventListener('click', () => { if (!customElements.get('ui-switch')) A.simToggle('opp'); });
  }
  const swTr = document.getElementById('sim-sw-trails');
  if (swTr && !swTr._bound) {
    swTr._bound = true;
    swTr.addEventListener('checkedchange', (e) => A.simToggle('trails', !!e.detail.value));
    swTr.addEventListener('click', () => { if (!customElements.get('ui-switch')) A.simToggle('trails'); });
  }
  const bindBtn = (id, fn) => {
    const el = document.getElementById(id);
    if (!el || el._bound) return;
    el._bound = true;
    el.addEventListener('action', fn);
    el.addEventListener('click', () => { if (!customElements.get('ui-button')) fn(); });
  };
  bindBtn('sim-back', () => A.simBack());
  bindBtn('sim-edit', () => A.tab('tactic', 'analyze'));
  bindBtn('sim-unfocus', () => A.simFocus(ST.sim.focus));

  const hitAt = (ev) => {
    const r = canvas.getBoundingClientRect();
    const mx = ((ev.clientX - r.left) / r.width) * 110, my = ((ev.clientY - r.top) / r.height) * 72;
    let best = null, bd = Infinity;
    SIM.hits.forEach(h => { const d = Math.hypot(mx - h.px, my - h.py); if (d <= h.r && d < bd) { bd = d; best = h.i; } });
    return best;
  };
  if (!canvas._bound) {
    canvas._bound = true;
    canvas.addEventListener('mousemove', (ev) => { canvas.style.cursor = hitAt(ev) != null ? 'pointer' : 'default'; });
    canvas.addEventListener('click', (ev) => { const i = hitAt(ev); if (i != null) A.simFocus(i); });
  }
  if (typeof ResizeObserver !== 'undefined') {
    if (SIM.ro) SIM.ro.disconnect();
    SIM.ro = new ResizeObserver(() => simFrame());
    SIM.ro.observe(canvas.parentElement || canvas);
  }
}

function simStart() {
  cancelAnimationFrame(SIM.raf);
  const canvas = document.getElementById('sim-canvas'); if (!canvas) return;
  const t = simTactic(); SIM.model = simModel(t); SIM.last = performance.now();
  simBindUI(canvas);
  simCaption(t);
  const tick = (now) => {
    if (!document.getElementById('sim-canvas')) { if (SIM.ro) { SIM.ro.disconnect(); SIM.ro = null; } return; }
    const sim = ST.sim, dt = Math.min(0.1, (now - SIM.last) / 1000); SIM.last = now;
    const tempo = [1.35, 1.15, 1, 0.85, 0.7][tiVal(simTactic(), 'tempo')];
    if (sim.playing) {
      sim.u += dt * sim.speed / (3.6 * tempo);
      if (sim.u >= 1) {
        sim.u = 0;
        sim.phase = (sim.phase + 1) % PHASES.length;
        const phEl = document.getElementById('sim-phases');
        if (phEl) phEl.value = String(sim.phase);
        simCaption(simTactic());
      }
    }
    simFrame(now);
    SIM.raf = requestAnimationFrame(tick);
  };
  SIM.raf = requestAnimationFrame(tick);
}
function simRebuild() { const t = simTactic(); SIM.model = simModel(t); simCaption(t); simFrame(); }

function simCaption(t) {
  const el = document.getElementById('sim-cap'); if (!el) return;
  const sim = ST.sim, ph = PHASES[sim.phase], c = phaseText(t, ph.k, SIM.model);
  el.innerHTML = `<div class="row" style="justify-content:space-between"><h3 style="margin:0">${ph.n}</h3><span class="chip ${ph.ph === 'OOP' ? 'oop' : 'ip'}">${ph.ph === 'IP' ? 'com bola' : ph.ph === 'OOP' ? 'sem bola' : 'transição'}</span></div>
    <p class="small" style="margin:.4em 0">${esc(c.text)}</p>
    ${c.notes.length ? `<ul class="clean small">${c.notes.map(n => `<li>${n}</li>`).join('')}</ul>` : ''}
    ${ph.k === 'oop' && SIM.model.overlays.length ? `<p class="small" style="color:var(--warn)"><b>Atenção:</b> ${SIM.model.overlays.map(o => esc(o.label)).join(' e ')}.</p>` : ''}`;
  translateDOM(el);
  document.querySelectorAll('.phases .ph').forEach((b, i) => b.classList.toggle('on', i === sim.phase));
}

function simFrame(now = performance.now()) {
  const M = SIM.model, sim = ST.sim, canvas = document.getElementById('sim-canvas');
  if (!M || !canvas) return;
  const k = PHASES[sim.phase].k, kp = PHASES[(sim.phase + PHASES.length - 1) % PHASES.length].k;
  const isReset = sim.phase === 0;
  const fromPh = isReset ? k : kp;
  const prevPath = M.ball[kp] || [];
  const prevBallEnd = !isReset && prevPath.length ? prevPath[prevPath.length - 1] : null;
  const moveU = isReset ? 1 : clamp(sim.u / 0.44, 0, 1);
  const um = ease(moveU);
  const carryU = isReset ? 1 : moveU;
  const bu = isReset ? clamp((sim.u - 0.1) / 0.86, 0, 1) : clamp((sim.u - 0.44) / 0.52, 0, 1);
  const fadeIn = isReset ? clamp(sim.u / 0.16, 0.15, 1) : 1;
  const oopish = k === 'oop' || k === 'loss' || (k === 'win' && sim.u < 0.22);
  const pmap = Object.fromEntries(players().map(p => [p.id, p]));
  drawPitchFrame(canvas, {
    M, fromPh, toPh: k, um, bu, carryU, prevBallEnd, isReset, fadeIn,
    oopish, showOpp: !!sim.opp, showTrails: !!sim.trails,
    focus: sim.focus, pmap, compact: false,
    pulse: (now % 1100) / 1100,
    pressFn: (p) => k === 'oop' && p.pressing && um > 0.45,
    hits: SIM.hits,
  });
  document.querySelectorAll('.phases .ph').forEach((b, i) => {
    const bar = b.querySelector('i');
    if (bar) bar.style.width = (i === sim.phase ? sim.u * 100 : i < sim.phase ? 100 : 0) + '%';
  });
}
