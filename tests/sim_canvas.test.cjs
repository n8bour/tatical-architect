const fs = require('fs'), vm = require('vm'), path = require('path');

const root = path.resolve(__dirname, '..');
const files = [
  'i18n/i18n.js', 'core/roles.js', 'core/tactics_data.js', 'core/engine.js',
  'core/insights.js', 'core/synergy.js', 'core/sim_model.js',
  'i18n/en_dict.js', 'i18n/pt_terms.js', 'ui/state.js', 'ui/components.js',
  'ui/sim_view.js', 'ui/clips.js',
];
const code = files.map(f => fs.readFileSync(path.join(root, 'src', f), 'utf8')).join('\n;\n');

let drawCalls = 0, badNums = 0;
const chkNum = (...xs) => xs.forEach(n => { if (typeof n === 'number' && !isFinite(n)) badNums++; });
function mockCanvas() {
  const grad = { addColorStop() {} };
  const ctx = {
    setTransform: (...a) => chkNum(...a),
    fillRect: (...a) => { drawCalls++; chkNum(...a); },
    strokeRect: (...a) => { drawCalls++; chkNum(...a); },
    beginPath() {}, closePath() {},
    moveTo: (...a) => chkNum(...a),
    lineTo: (...a) => chkNum(...a),
    quadraticCurveTo: (...a) => chkNum(...a),
    arc: (...a) => { drawCalls++; chkNum(...a); },
    ellipse: (...a) => { drawCalls++; chkNum(...a); },
    roundRect: (...a) => { drawCalls++; chkNum(...a); },
    fill() {}, stroke() {}, save() {}, restore() {},
    setLineDash() {},
    fillText: (t, x, y) => { drawCalls++; chkNum(x, y); },
    strokeText: (t, x, y) => { drawCalls++; chkNum(x, y); },
    measureText: (t) => ({ width: String(t).length * 1.1 }),
    createLinearGradient: (...a) => { chkNum(...a); return grad; },
    createRadialGradient: (...a) => { chkNum(...a); return grad; },
  };
  return { width: 0, height: 0, clientWidth: 660, getContext: () => ctx };
}

const ctxObj = {
  console, Math,
  window: { devicePixelRatio: 2 },
  document: { documentElement: {} },
  getComputedStyle: () => ({ getPropertyValue: () => '' }),
  matchMedia: () => ({ matches: false }),
  performance: { now: () => 500 },
  localStorage: { getItem: () => null, setItem() {} },
};
vm.createContext(ctxObj);
vm.runInContext(code + ';this.T={ROLES,TEMPLATES,TI_BY_KEY,PHASES,SIM_TI,CONCEPTS,OPPONENTS,CONCEPT_CLIPS,ST,tacticFromTemplate,simModel,roleClipModel,conceptClipModel,oppClipModel,clipModelTI,simBallState,drawPitchFrame,vSim};', ctxObj);
const T = ctxObj.T;
let fails = 0;
const ok = (c, m) => { if (!c) { fails++; console.error('FAIL:', m); } };

// 1. Ball elevation arc: loft rises above pitch at midpoint, ground pass stays at z=0
const loftMid = T.simBallState([{ x: 20, y: 10, kind: 'loft' }, { x: 80, y: 70, kind: 'loft' }], 0.5);
const passMid = T.simBallState([{ x: 20, y: 10, kind: 'pass' }, { x: 80, y: 70, kind: 'pass' }], 0.5);
ok(loftMid.z > 0.8, 'lofted ball should elevate at midpoint');
ok(passMid.z === 0, 'ground pass should stay at z=0');

// 2. All 77 role scenarios paint cleanly on HTML5 canvas with zero NaN and continuous ball handoffs
const c = mockCanvas();
const roleSigs = new Set();
const bDist = (a, b) => Math.hypot(a.x - b.x, a.y - b.y);
T.ROLES.forEach(r => {
  const rc = T.roleClipModel(r.id);
  rc.phases.forEach((ph, idx) => {
    const from = idx === 0 ? rc.phases[0] : rc.phases[idx - 1];
    if (idx > 0) {
      const p0 = rc.M.ball[from], p1 = rc.M.ball[ph];
      const d = bDist(p0[p0.length - 1], p1[0]);
      ok(d <= 22, `role ${r.id} ball jump ${from}->${ph} too large (${d.toFixed(1)})`);
    }
    T.drawPitchFrame(c, {
      M: rc.M, fromPh: from, toPh: ph, um: 0.7, bu: 0.5,
      oopish: r.phase === 'OOP', showOpp: true, showTrails: false,
      focus: rc.focus, pmap: null, compact: true, phaseLabel: rc.labels[idx],
      ghostPt: rc.M.P[rc.focus].pos.s0, pulse: 0.4,
      pressFn: (p, k) => k === rc.focus && rc.M.pressAt && rc.M.pressAt[ph],
    });
  });
  const me = rc.M.P[rc.focus];
  roleSigs.add(JSON.stringify([me.pos, rc.M.ball.s1, rc.M.ball.s2, rc.M.zones.s2.map(z => z.label)]));
});
ok(roleSigs.size >= 75, `expected distinct role scenarios, got ${roleSigs.size}`);

// 3. All 23 SIM_TI instructions produce distinct min/max visuals, continuous ball transitions, and paint cleanly
T.SIM_TI.forEach(k => {
  const d = T.TI_BY_KEY[k];
  const m0 = T.clipModelTI(k, 0), m1 = T.clipModelTI(k, d.opts.length - 1);
  const snap = (m) => JSON.stringify([m.phases.map(ph => [
    m.M.P.map(p => [Math.round(p.pos[ph].x), Math.round(p.pos[ph].y)]),
    m.M.ball[ph].map(b => [Math.round(b.x), Math.round(b.y), b.kind]),
    m.M.lines[ph].map(l => Math.round(l.y)),
    m.M.zones[ph].map(z => [z.kind, z.x1, z.x2, z.y1, z.y2]),
    m.M.arrows[ph].length,
  ]), m.tempoMul]);
  ok(snap(m0) !== snap(m1), `SIM_TI ${k} min and max are identical`);
  [m0, m1].forEach((m, mi) => {
    const [pA, pB] = m.phases;
    const bA = m.M.ball[pA], bB = m.M.ball[pB];
    const lastA = bA[bA.length - 1], firstB = bB[0];
    if (pA === 'final' || pA === 'loss' || pA === 'oop') {
      ok(bDist(lastA, firstB) <= 5, `SIM_TI ${k}[${mi}] ${pA}->${pB} ball jump ${bDist(lastA, firstB).toFixed(1)}`);
    } else {
      ok(firstB.y - lastA.y >= -10, `SIM_TI ${k}[${mi}] ${pA}->${pB} ball jumped backwards (dy=${(firstB.y - lastA.y).toFixed(1)})`);
    }
    [pA, pB].forEach(ph => {
      if (['build', 'prog', 'final'].includes(ph)) {
        const arr = m.M.ball[ph];
        for (let j = 1; j < arr.length; j++) {
          ok(arr[j].y >= arr[j - 1].y - 8, `SIM_TI ${k}[${mi}] ${ph} intra-phase backward pass (${Math.round(arr[j - 1].y)}->${Math.round(arr[j].y)})`);
        }
      }
    });
    T.drawPitchFrame(c, {
      M: m.M, fromPh: pA, toPh: pB, um: 1, bu: 0.6,
      oopish: pB === 'oop' || pB === 'loss', showOpp: true, showTrails: false,
      focus: null, pmap: null, compact: true, phaseLabel: pB, pulse: 0.3,
    });
  });
});

// 4. All 8 templates x 6 phases paint cleanly on full canvas with continuous ball handoffs
T.TEMPLATES.forEach(tp => {
  const M = T.simModel(T.tacticFromTemplate(tp));
  T.PHASES.forEach((ph, idx) => {
    const prev = idx === 0 ? ph.k : T.PHASES[idx - 1].k;
    if (idx > 0) {
      const bA = M.ball[prev], bB = M.ball[ph.k];
      const lastA = bA[bA.length - 1], firstB = bB[0];
      if (prev === 'final' || prev === 'loss' || prev === 'oop') {
        ok(bDist(lastA, firstB) <= 5, `template ${tp.id} ${prev}->${ph.k} ball jump ${bDist(lastA, firstB).toFixed(1)}`);
      } else {
        ok(firstB.y - lastA.y >= -10, `template ${tp.id} ${prev}->${ph.k} ball jumped backwards (dy=${(firstB.y - lastA.y).toFixed(1)})`);
      }
    }
    if (['build', 'prog', 'final'].includes(ph.k)) {
      const arr = M.ball[ph.k];
      for (let j = 1; j < arr.length; j++) {
        ok(arr[j].y >= arr[j - 1].y - 8, `template ${tp.id} ${ph.k} intra-phase backward pass (${Math.round(arr[j - 1].y)}->${Math.round(arr[j].y)})`);
      }
    }
    const hits = [];
    T.drawPitchFrame(c, {
      M, fromPh: prev, toPh: ph.k, um: 0.8, bu: 0.5,
      oopish: ph.k === 'oop' || ph.k === 'loss', showOpp: true, showTrails: true,
      focus: 3, pmap: {}, compact: false, pulse: 0.5, hits,
    });
    ok(hits.length === 11, `expected 11 clickable player hit targets for ${tp.id}:${ph.k}`);
  });
});

// 4b. All 8 concept clips + 6 opponent matchup clips paint cleanly with zero NaN and smooth transitions
const checkClip = (label, m) => {
  ok(m && m.phases.length >= 2, label);
  m.phases.forEach((ph, idx) => {
    const from = idx === 0 ? m.phases[0] : m.phases[idx - 1];
    if (idx > 0) {
      const bA = m.M.ball[from], bB = m.M.ball[ph];
      const lastA = bA[bA.length - 1], firstB = bB[0];
      if (from === 'final' || from === 'loss' || from === 'oop') {
        ok(bDist(lastA, firstB) <= 5, `${label} ${from}->${ph} ball jump ${bDist(lastA, firstB).toFixed(1)}`);
      } else if (from === 's0' || from === 's1') {
        ok(bDist(lastA, firstB) <= 22, `${label} ${from}->${ph} ball jump ${bDist(lastA, firstB).toFixed(1)}`);
      } else {
        ok(firstB.y - lastA.y >= -10, `${label} ${from}->${ph} ball jumped backwards (dy=${(firstB.y - lastA.y).toFixed(1)})`);
      }
    }
    T.drawPitchFrame(c, { M: m.M, fromPh: from, toPh: ph, um: 0.8, bu: 0.5, oopish: m.phaseByStep[idx] === 'OOP', showOpp: true, showTrails: false, focus: null, pmap: null, compact: true, phaseLabel: m.labels[idx], pulse: 0.4 });
  });
};
T.CONCEPTS.filter(x => T.CONCEPT_CLIPS.has(x.k)).forEach(x => checkClip(`concept:${x.k}`, T.conceptClipModel(x.k)));
T.OPPONENTS.forEach(o => checkClip(`opp:${o.k}`, T.oppClipModel(o.k)));

// 5. vSim renders HTML5 canvas + properly composed and named ui-sdk web components
const html = T.vSim();
ok(html.includes('<canvas class="simpitch" id="sim-canvas"'), 'vSim renders HTML5 canvas');
ok(html.includes('<ui-toggle id="sim-play-tg"') && html.includes('<button type="button" id="sim-play">'), 'ui-toggle wraps native button without duplicate aria-label');
ok(!/<button[^>]*id="sim-play"[^>]*aria-label=/.test(html), 'button with visible text should not have aria-label');
ok(html.includes('<ui-segmented-control id="sim-speed" aria-label='), 'speed uses named ui-segmented-control');
ok(html.includes('<ui-segmented-control id="sim-phases" class="phases" aria-label='), 'phases use named ui-segmented-control');
ok(html.includes('<ui-switch id="sim-sw-opp"') && html.includes('<ui-switch-thumb></ui-switch-thumb>'), 'opponent uses ui-switch with thumb');
ok(html.includes('<ui-switch id="sim-sw-trails"'), 'trails uses ui-switch');
ok(html.includes('<ui-accordion id="sim-acc" multiple default-value="ip,oop" aria-label='), 'instructions use named multi ui-accordion');
ok(html.includes('<ui-button id="sim-edit"><button type="button"'), 'action button wrapped in ui-button');

// 6. CSS starts with @layer ui-sdk;
const css = fs.readFileSync(path.join(root, 'src', 'styles.css'), 'utf8');
ok(css.trimStart().startsWith('@layer ui-sdk;'), 'styles.css declares @layer ui-sdk; at top');
ok(badNums === 0, `canvas draw produced ${badNums} NaN/non-finite coordinates`);

console.log(`canvas+ui-sdk tests: ${T.ROLES.length} roles (${roleSigs.size} unique), ${T.SIM_TI.size} TIs, ${drawCalls} canvas draw calls, ${badNums} NaNs`);
if (fails) process.exit(1);
