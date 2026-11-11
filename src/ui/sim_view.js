// ============================================================
// Campo animado — runtime e vista (usa core/sim_model.js)
// ============================================================
// ---------- runtime da animação
const SIM = { raf: 0, last: 0, model: null };
const X_ = (y) => 3 + y * 1.04, Y_ = (x) => 2 + x * 0.68;

function simTactic() { return ST.sim && ST.sim.src === 'demo' ? ST.sim.t : ST.tactic; }
function simDefaults() { if (!ST.sim) ST.sim = { src: 'tactic', phase: 0, u: 0, playing: !matchMedia('(prefers-reduced-motion: reduce)').matches, speed: 1, opp: true, trails: false, focus: null, hl: null }; return ST.sim; }

function simSVG(t) {
  const pmap = Object.fromEntries(players().map(p => [p.id, p]));
  const sim = ST.sim;
  return `<svg class="simpitch" viewBox="0 0 110 72" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="Campo animado da tática">
    <rect x="0" y="0" width="110" height="72" fill="var(--pitch)"/>
    ${[0,1,2,3,4,5,6,7,8,9].map(k => k % 2 ? `<rect x="${3 + k * 10.4}" y="2" width="10.4" height="68" fill="var(--pitch2)"/>` : '').join('')}
    <g fill="none" stroke="var(--pitchline)" stroke-width=".35">
      <rect x="3" y="2" width="104" height="68"/><line x1="55" y1="2" x2="55" y2="70"/><circle cx="55" cy="36" r="9.2"/>
      <rect x="3" y="15.8" width="16.6" height="40.4"/><rect x="3" y="26.8" width="5.6" height="18.4"/>
      <rect x="90.4" y="15.8" width="16.6" height="40.4"/><rect x="101.4" y="26.8" width="5.6" height="18.4"/>
      <line x1="1.6" y1="32.4" x2="1.6" y2="39.6" stroke-width=".9"/><line x1="108.4" y1="32.4" x2="108.4" y2="39.6" stroke-width=".9"/>
    </g>
    <text x="5" y="69" font-size="2.2" fill="rgba(255,255,255,.6)">a tua baliza</text>
    <text x="105" y="69" font-size="2.2" fill="rgba(255,255,255,.6)" text-anchor="end">ataque →</text>
    <g id="sim-over"></g><g id="sim-trails"></g>
    <g id="sim-opp" ${sim.opp ? '' : 'style="display:none"'}>${OPP_SHAPE.map((_, k) => `<circle r="1.35" fill="rgba(235,235,235,.38)" stroke="rgba(0,0,0,.25)" stroke-width=".2"/>`).join('')}</g>
    <g id="sim-us">${t.slots.map((s, i) => {
      const p = pmap[s.pid]; const f = sim.focus === i;
      return `<g class="simp${f ? ' focus' : ''}" data-i="${i}" onclick="A.simFocus(${i})">
        <circle class="press" r="3.4" fill="none" stroke="#ff6b5a" stroke-width=".45" opacity="0"/>
        ${f ? '<circle r="3.6" fill="none" stroke="#ffe27a" stroke-width=".6"/>' : ''}
        <circle r="${f ? 2.7 : 2.25}" fill="${s.ipCode === 'GK' ? '#3c3f46' : f ? '#0b5d3a' : '#123b29'}" stroke="#fff" stroke-width=".35"/>
        <text class="ab" y=".75" text-anchor="middle" font-size="${f ? 1.9 : 1.6}" font-weight="700" fill="#fff"></text>
        ${p ? `<text y="4.9" text-anchor="middle" font-size="1.55" fill="#fff" stroke="rgba(0,0,0,.55)" stroke-width=".35" paint-order="stroke">${esc(p.name.split(' ').slice(-1)[0]).slice(0, 11)}</text>` : ''}
      </g>`; }).join('')}</g>
    <circle id="sim-ball" r=".95" fill="#fff" stroke="#111" stroke-width=".25"/>
  </svg>`;
}

function vSim() {
  const sim = simDefaults(), t = simTactic();
  const forms = Object.keys(FORMATIONS).map(f => [f, f]);
  const ctl = (k) => { const d = TI_BY_KEY[k]; return `<label class="f ${sim.hl === k ? 'hl' : ''}">${esc(d.n)}${selectHTML(d.opts.map((o, i) => [i, o]), tiVal(t, k), `A.simTI('${k}',this.value)`)}</label>`; };
  const fr = sim.focus != null ? t.slots[sim.focus] : null;
  const fRole = fr ? [ROLE_BY_ID[fr.ip], ROLE_BY_ID[fr.oop]] : null;
  return `<div class="top"><div><h2>${sim.src === 'demo' ? esc(sim.title) : 'Campo animado — ' + esc(t.name)}</h2>
      <p class="muted small" style="margin:0">${esc(t.ipForm)} com bola → ${esc(t.oopForm)} sem bola · ${esc(t.ment)}. Clique num jogador para o seguir.</p></div>
    <div class="row">${sim.src === 'demo' ? '<button class="btn" onclick="A.simBack()">Voltar à minha tática</button>' : '<button class="btn" onclick="A.tab(\'tactic\',\'analyze\')">Editar no analisador</button>'}</div></div>
  <div class="simgrid">
    <div>
      <div class="simwrap">${simSVG(t)}</div>
      <div class="simbar">
        <button class="btn primary" id="sim-play" onclick="A.simPlay()" aria-label="${sim.playing ? 'Pausar' : 'Reproduzir'}">${sim.playing ? '❚❚ Pausar' : '▶ Reproduzir'}</button>
        <div class="phases">${PHASES.map((p, i) => `<button class="ph ${p.ph}" data-ph="${i}" onclick="A.simPhase(${i})"><span>${p.n}</span><i></i></button>`).join('')}</div>
        <div class="seg">${[0.5, 1, 2].map(s => `<button class="${sim.speed === s ? 'on' : ''}" onclick="A.simSpeed(${s})">${s}×</button>`).join('')}</div>
      </div>
      <div class="panel simcap" id="sim-cap"></div>
    </div>
    <div class="grid simside">
      ${fr ? `<div class="panel"><h3>${esc(fr.ipCode)} → ${esc(fr.oopCode)}</h3>
        ${fRole.map(r => r ? `<p class="small"><span class="chip ${r.phase === 'IP' ? 'ip' : 'oop'}">${r.phase}</span> <b>${esc(r.name)}</b> — ${esc(r.desc)}</p>` : '').join('')}
        ${sim.src === 'demo' ? '' : `<div class="grid g2">
          <label class="f">Papel com bola${selectHTML(rolesFor(slotInfo(t.ipForm, fr.ipCode).group, 'IP', slotInfo(t.ipForm, fr.ipCode).wideCB).map(r => [r.id, r.name]), fr.ip, `A.simRole('IP',this.value)`)}</label>
          <label class="f">Papel sem bola${selectHTML(rolesFor(slotInfo(t.oopForm, fr.oopCode).group, 'OOP', slotInfo(t.oopForm, fr.oopCode).wideCB).map(r => [r.id, r.name]), fr.oop, `A.simRole('OOP',this.value)`)}</label></div>`}
        <button class="btn sm" onclick="A.simFocus(${sim.focus})">Deixar de seguir</button></div>` : ''}
      <div class="panel grid">
        <h3>Experimente as instruções</h3>
        <p class="small muted" style="margin:0">Cada mudança redesenha o movimento na hora${sim.src === 'demo' ? ' (só nesta demonstração)' : ' e fica gravada na tática'}.</p>
        <label class="f">Mentalidade${selectHTML(MENTALITIES.map(m => [m, m]), t.ment, 'A.simMent(this.value)')}</label>
        ${sim.src === 'demo' ? '' : `<div class="grid g2"><label class="f">Formação com bola${selectHTML(forms, t.ipForm, "A.setForm('IP',this.value)")}</label><label class="f">Sem bola${selectHTML(forms, t.oopForm, "A.setForm('OOP',this.value)")}</label></div>`}
        <details open><summary>Com bola</summary><div class="grid g2" style="margin-top:6px">${['buildup','goal_kicks','passing','tempo','width','support_runs','progress','att_trans'].map(ctl).join('')}</div></details>
        <details open><summary>Sem bola</summary><div class="grid g2" style="margin-top:6px">${['loe','dline','dl_behaviour','trigger','press_trap','short_gk','def_trans'].map(ctl).join('')}</div></details>
        <div class="row small"><label><input type="checkbox" ${sim.opp ? 'checked' : ''} onchange="A.simToggle('opp')"> Adversário</label><label><input type="checkbox" ${sim.trails ? 'checked' : ''} onchange="A.simToggle('trails')"> Percursos</label></div>
      </div>
      <p class="small muted">O campo é um modelo simplificado: mostra a intenção de cada papel e instrução, não reproduz o motor de jogo do FM27.</p>
    </div>
  </div>`;
}

function simStart() {
  cancelAnimationFrame(SIM.raf);
  const svg = document.querySelector('.simpitch'); if (!svg) return;
  const t = simTactic(); SIM.model = simModel(t); SIM.last = performance.now();
  simDrawStatic(t); simCaption(t);
  const tick = (now) => {
    if (!document.querySelector('.simpitch')) return;
    const sim = ST.sim, dt = Math.min(0.1, (now - SIM.last) / 1000); SIM.last = now;
    const tempo = [1.25, 1.1, 1, 0.9, 0.8][tiVal(simTactic(), 'tempo')];
    if (sim.playing) {
      sim.u += dt * sim.speed / (3.6 * tempo);
      if (sim.u >= 1) { sim.u = 0; sim.phase = (sim.phase + 1) % PHASES.length; simCaption(simTactic()); }
    }
    simFrame();
    SIM.raf = requestAnimationFrame(tick);
  };
  SIM.raf = requestAnimationFrame(tick);
}
function simRebuild() { const t = simTactic(); SIM.model = simModel(t); simDrawStatic(t); simCaption(t); }

function simDrawStatic(t) {
  const M = SIM.model, sim = ST.sim;
  const over = document.getElementById('sim-over'); if (!over) return;
  // avisos só aparecem na fase sem bola (ver simFrame)
  over.innerHTML = M.overlays.map(o => {
    const x1 = X_(o.y1), x2 = X_(o.y2); if (x2 <= x1) return '';
    return `<g class="ov" data-kind="${o.kind}"><rect x="${x1}" y="2" width="${x2 - x1}" height="68" fill="${o.kind === 'gap' ? 'rgba(255,107,90,.18)' : 'rgba(255,200,80,.16)'}" stroke="${o.kind === 'gap' ? 'rgba(255,107,90,.7)' : 'rgba(255,200,80,.7)'}" stroke-width=".3" stroke-dasharray="1 .8"/>
      <text x="${(x1 + x2) / 2}" y="5.2" text-anchor="middle" font-size="2" font-weight="700" fill="#fff" stroke="rgba(0,0,0,.5)" stroke-width=".35" paint-order="stroke">${esc(o.label)}</text></g>`;
  }).join('');
  const tr = document.getElementById('sim-trails');
  tr.innerHTML = sim.trails || sim.focus != null ? M.P.filter(p => sim.trails || p.i === sim.focus).map(p => {
    const pts = PHASES.map(ph => `${X_(p.pos[ph.k].y)},${Y_(p.pos[ph.k].x)}`).join(' ');
    return `<polyline points="${pts} ${X_(p.pos.build.y)},${Y_(p.pos.build.x)}" fill="none" stroke="${p.i === sim.focus ? '#ffe27a' : 'rgba(255,255,255,.35)'}" stroke-width="${p.i === sim.focus ? .45 : .25}" stroke-dasharray="1 .7"/>`;
  }).join('') : '';
  document.querySelectorAll('.simp').forEach((g, i) => { const p = M.P[i]; g.dataset.ab = p.ip ? p.ip.ab : '?'; g.dataset.abo = p.oop ? p.oop.ab : '?'; });
}

function simCaption(t) {
  const el = document.getElementById('sim-cap'); if (!el) return;
  const sim = ST.sim, ph = PHASES[sim.phase], c = phaseText(t, ph.k, SIM.model);
  el.innerHTML = `<div class="row" style="justify-content:space-between"><h3 style="margin:0">${ph.n}</h3><span class="chip ${ph.ph === 'OOP' ? 'oop' : 'ip'}">${ph.ph === 'IP' ? 'com bola' : ph.ph === 'OOP' ? 'sem bola' : 'transição'}</span></div>
    <p class="small" style="margin:.4em 0">${esc(c.text)}</p>
    ${c.notes.length ? `<ul class="clean small">${c.notes.map(n => `<li>${n}</li>`).join('')}</ul>` : ''}
    ${ph.k === 'oop' && SIM.model.overlays.length ? `<p class="small" style="color:var(--warn)"><b>Atenção:</b> ${SIM.model.overlays.map(o => esc(o.label)).join(' e ')}.</p>` : ''}`;
  document.querySelectorAll('.phases .ph').forEach((b, i) => b.classList.toggle('on', i === sim.phase));
}

function simFrame() {
  const M = SIM.model, sim = ST.sim; if (!M) return;
  const k = PHASES[sim.phase].k, kp = PHASES[(sim.phase + PHASES.length - 1) % PHASES.length].k;
  const um = ease(clamp(sim.u / 0.62, 0, 1));
  const oopish = k === 'oop' || k === 'loss' || (k === 'win' && sim.u < 0.3);
  document.querySelectorAll('.simp').forEach((g, i) => {
    const p = M.P[i]; const a = p.pos[kp], b = p.pos[k];
    const x = a.x + (b.x - a.x) * um, y = a.y + (b.y - a.y) * um;
    g.setAttribute('transform', `translate(${X_(y).toFixed(2)},${Y_(x).toFixed(2)})`);
    g.querySelector('.ab').textContent = oopish ? g.dataset.abo : g.dataset.ab;
    g.querySelector('.press').setAttribute('opacity', k === 'oop' && p.pressing && um > 0.5 ? 1 : 0);
  });
  const opp = document.querySelectorAll('#sim-opp circle');
  opp.forEach((c, i) => { const a = M.opp[kp][i], b = M.opp[k][i]; c.setAttribute('cx', X_(a.y + (b.y - a.y) * um).toFixed(2)); c.setAttribute('cy', Y_(a.x + (b.x - a.x) * um).toFixed(2)); });
  // bola: percorre os pontos da fase entre 15% e 95% do tempo
  const path = M.ball[k]; const bu = clamp((sim.u - 0.15) / 0.8, 0, 1);
  const seg = path.length - 1, f = bu * seg, j = Math.min(seg - 1, Math.floor(f)), r = ease(f - j);
  const A0 = path[j], B0 = path[j + 1] || path[j];
  const bx = A0.x + (B0.x - A0.x) * r, by = A0.y + (B0.y - A0.y) * r;
  const ball = document.getElementById('sim-ball'); ball.setAttribute('cx', X_(by).toFixed(2)); ball.setAttribute('cy', Y_(bx).toFixed(2));
  document.querySelectorAll('#sim-over .ov').forEach(o => o.style.display = k === 'oop' ? '' : 'none');
  document.querySelectorAll('.phases .ph').forEach((b, i) => { const bar = b.querySelector('i'); bar.style.width = (i === sim.phase ? sim.u * 100 : i < sim.phase ? 100 : 0) + '%'; });
}

