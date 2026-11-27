const fs = require('fs'), vm = require('vm'), path = require('path');

const root = path.resolve(__dirname, '..');
const files = [
  'i18n/i18n.js', 'core/roles.js', 'core/tactics_data.js', 'core/engine.js',
  'core/insights.js', 'core/synergy.js', 'core/sim_model.js',
  'i18n/en_dict.js', 'i18n/pt_terms.js', 'ui/state.js', 'ui/components.js',
  'ui/sim_view.js',
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
vm.runInContext(code + ';this.T={ROLES,TEMPLATES,TI_BY_KEY,PHASES,ST,tacticFromTemplate,simModel,simBallState,drawPitchFrame,vSim};', ctxObj);
const T = ctxObj.T;
let fails = 0;
const ok = (c, m) => { if (!c) { fails++; console.error('FAIL:', m); } };

// 1. Ball elevation arc: loft rises above pitch at midpoint, ground pass stays at z=0
const loftMid = T.simBallState([{ x: 20, y: 10, kind: 'loft' }, { x: 80, y: 70, kind: 'loft' }], 0.5);
const passMid = T.simBallState([{ x: 20, y: 10, kind: 'pass' }, { x: 80, y: 70, kind: 'pass' }], 0.5);
ok(loftMid.z > 0.8, 'lofted ball should elevate at midpoint');
ok(passMid.z === 0, 'ground pass should stay at z=0');

// 2. All 8 templates x 6 phases paint cleanly on full canvas with continuous ball handoffs
const c = mockCanvas();
const bDist = (a, b) => Math.hypot(a.x - b.x, a.y - b.y);
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

// 3. vSim renders HTML5 canvas + properly composed and named ui-sdk web components
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

console.log(`canvas+ui-sdk tests: ${T.TEMPLATES.length} templates x ${T.PHASES.length} phases, ${drawCalls} canvas draw calls, ${badNums} NaNs`);
if (fails) process.exit(1);
