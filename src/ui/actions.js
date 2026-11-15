// ============================================================
// Acções (ligadas aos botões da interface)
// ============================================================
const A = {
  go(v, tab) { ST.view = v; if (tab && ST.tabs[v] !== undefined) ST.tabs[v] = tab; save(); render(true); },
  tab(v, tab) { ST.view = v; ST.tabs[v] = tab; save(); render(true); },
  theme(v) { ST.theme = v; applyTheme(); save(); render(); },
  // tática
  setName(v) { ST.tactic.name = v; save(); },
  setMent(v) { ST.tactic.ment = v; save(); render(); },
  setForm(phase, v) {
    const t = ST.tactic;
    ST.tactic = retargetTactic(t, phase === 'IP' ? v : t.ipForm, phase === 'OOP' ? v : t.oopForm);
    ST.sel = null; save(); render();
  },
  setTI(k, v) { const d = TI_BY_KEY[k].def; v = +v; if (v === d) delete ST.tactic.ti[k]; else ST.tactic.ti[k] = v; save(); render(); },
  resetTI() { ST.tactic.ti = {}; save(); render(); toast('Instruções repostas'); },
  sel(i) { ST.sel = ST.sel === i ? null : i; render(); },
  pitch(m) { ST.pitchMode = m; render(); },
  setRole(i, phase, id) { ST.tactic.slots[i][phase === 'IP' ? 'ip' : 'oop'] = id; save(); render(); },
  setPlayer(i, pid) {
    const t = ST.tactic; const other = t.slots.findIndex((s, j) => j !== i && s.pid === pid && pid);
    if (other >= 0) t.slots[other].pid = t.slots[i].pid; // troca
    t.slots[i].pid = pid || null; save(); render();
  },
  togglePI(i, j) { const n = PLAYER_INSTRUCTIONS[j].n; const s = ST.tactic.slots[i]; s.pis = s.pis || []; const k = s.pis.indexOf(n); if (k >= 0) s.pis.splice(k, 1); else s.pis.push(n); save(); render(); },
  setOIs(v) { ST.tactic.ois = Math.max(0, +v || 0); save(); render(); },
  autoXI() { if (players().length < 11) return toast('Precisa de pelo menos 11 jogadores no plantel'); autoPickXI(ST.tactic, players()); save(); render(); toast('XI escolhido pela melhor adequação'); },
  clearXI() { ST.tactic.slots.forEach(s => s.pid = null); save(); render(); },
  newTactic() { ST.tactic = newTactic(ST.tactic.ipForm, ST.tactic.oopForm); ST.sel = null; save(); render(); },
  saveTactic() { const c = JSON.parse(JSON.stringify(ST.tactic)); c.savedAt = new Date().toISOString(); ST.saved = ST.saved.filter(x => x.name !== c.name); ST.saved.unshift(c); save(); render(); toast('Tática guardada: ' + c.name); },
  loadSaved(i) { ST.tactic = JSON.parse(JSON.stringify(ST.saved[i])); ST.sel = null; save(); A.go('tactic', 'analyze'); },
  delSaved(i) { ST.saved.splice(i, 1); save(); render(); },
  exportTactic() { download((ST.tactic.name || 'tatica').replace(/[^\w\-]+/g, '_') + '.fm27tb.json', JSON.stringify({ kind: 'fm27tb-tactic', v: 1, tactic: ST.tactic }, null, 2)); },
  importJSON(input) {
    const f = input.files[0]; if (!f) return;
    f.text().then(txt => {
      const d = JSON.parse(txt);
      ingestJSON(d, f.name);
    }).catch(() => toast('JSON inválido'));
  },
  useTemplate(id) { const tp = TEMPLATES.find(x => x.id === id); ST.tactic = tacticFromTemplate(tp); if (players().length >= 11) autoPickXI(ST.tactic, players()); ST.sel = null; save(); A.go('tactic', 'analyze'); },
  useSuggestion(i) { ST.tactic = JSON.parse(JSON.stringify(ST.sugg[i].t)); ST.sel = null; save(); A.go('tactic', 'analyze'); },
  applyPI(i, n) { const s = ST.tactic.slots[i]; s.pis = s.pis || []; if (!s.pis.includes(n)) s.pis.push(n); save(); render(); toast(n + ' aplicado'); },
  applyAllPIs() { const pm = Object.fromEntries(players().map(p => [p.id, p])); let n = 0;
    ST.tactic.slots.forEach(s => { const a = slotInfo(ST.tactic.ipForm, s.ipCode); recommendPIs(pm[s.pid], ROLE_BY_ID[s.ip], ROLE_BY_ID[s.oop], a.group).slice(0, 1).forEach(r => { s.pis = s.pis || []; if (!s.pis.includes(r.pi)) { s.pis.push(r.pi); n++; } }); });
    save(); render(); toast(n + ' instruções aplicadas (a principal de cada jogador — mais instruções = mais complexidade)'); },
  applyPlan(list) { const rep = ST._oppRep; if (!rep) return; const items = list === 'all' ? rep.plan : [rep.plan[list]];
    items.forEach(x => { const d = TI_BY_KEY[x.ti]; if (x.v === d.def) delete ST.tactic.ti[x.ti]; else ST.tactic.ti[x.ti] = x.v; });
    save(); render(); toast(items.length === 1 ? `${TI_BY_KEY[items[0].ti].n}: ${TI_BY_KEY[items[0].ti].opts[items[0].v]}` : 'Plano aplicado à tática'); },
  applySpTip(i) { const tp = ST._sp && ST._sp.tips[i]; if (!tp || !tp.ti) return; const d = TI_BY_KEY[tp.ti]; if (tp.v === d.def) delete ST.tactic.ti[tp.ti]; else ST.tactic.ti[tp.ti] = tp.v; save(); render(); toast(`${d.n}: ${d.opts[tp.v]}`); },
  genOpt(k, v) { genOpts()[k] = v; save(); render(); },
  genToggle(k, id) { const g = genOpts(); const i = g[k].indexOf(id); if (i >= 0) g[k].splice(i, 1); else if (k !== 'youth' || g[k].length < 3) g[k].push(id); else toast('Máximo 3 jovens'); save(); render(); },
  genRun() { const g = genOpts(); toast('A gerar…'); setTimeout(() => { const t0 = performance.now(); g.res = generateTactic(players(), { expect: g.expect, must: g.must, star: g.star || null, youth: g.youth }); g.pick = 0; save(); render(); toast(`Tática gerada em ${Math.round(performance.now() - t0)} ms`); }, 30); },
  genPick(i) { genOpts().pick = i; save(); render(); },
  genTactic() { const g = genOpts(); const all = [g.res.main, ...g.res.alts]; const t = JSON.parse(JSON.stringify((all[g.pick] || all[0]).t));
    const pm = Object.fromEntries(players().map(p => [p.id, p]));
    t.slots.forEach(s => { const r = recommendPIs(pm[s.pid], ROLE_BY_ID[s.ip], ROLE_BY_ID[s.oop], slotInfo(t.ipForm, s.ipCode).group)[0]; s.pis = r ? [r.pi] : []; });
    t.name = `${t.ipForm} — gerada para ${ST.myClub || 'o plantel'}`; t.source = 'gerador'; return t; },
  genUse(sim, tab) { const t = A.genTactic(); ST.tactic = t; ST.saved = ST.saved.filter(x => x.name !== t.name); ST.saved.unshift(JSON.parse(JSON.stringify(t))); ST.sugg = null; save();
    if (sim) return A.simOpen(); if (tab === 'training') return A.go('training'); A.tab('tactic', tab || 'analyze'); toast('Tática gerada carregada e guardada'); },
  genCopy() { const txt = sheetText(A.genTactic()); (navigator.clipboard ? navigator.clipboard.writeText(txt) : Promise.reject()).then(() => toast('Tática copiada — pronta para colar no fórum/Discord'), () => download('tatica-gerada.txt', txt, 'text/plain')); },
  labSet(k, v) { labState()[k] = v; save(); },
  async labConnect() {
    const L = labState(); L.busy = true; L.err = ''; render();
    const ports = [L.port, ...Array.from({ length: 16 }, (_, i) => 8765 + i)].filter((p, i, a) => a.indexOf(p) === i);
    for (const p of ports) {
      L.port = p;
      try { const s = await labCall('GET', '/api/status', null, 400); if (s && s.engine) { L.status = s; break; } } catch (e) { /* tenta a seguinte */ }
    }
    L.busy = false;
    if (!L.status) { L.err = 'Não encontrei o Tactics Lab. Abra-o primeiro (com o Steam aberto) e confirme a porta no endereço que ele mostra.'; L.port = ports[0]; render(); return; }
    save(); await A.labJobs();
  },
  async labJobs() { const L = labState(); try { const j = await labCall('GET', '/api/jobs?brief=1'); L.jobs = (Array.isArray(j) ? j : []).sort((a, b) => (b.created || 0) - (a.created || 0)); L.err = ''; } catch (e) { L.err = e.message; L.status = null; } render(); },
  async labOpen(id) { const L = labState(); try { L.job = await labCall('GET', '/api/jobs/' + id, null, 8000); L.err = ''; } catch (e) { L.err = e.message; } render(); },
  async labCancel(id) { try { await labCall('POST', '/api/cancel', { id }); toast('Cancelamento pedido'); setTimeout(() => A.labOpen(id), 800); } catch (e) { toast(e.message); } },
  async labMore(id) {
    const L = labState(), n = L.more || 100;
    try {
      const pkg = await labCall('GET', '/api/jobs/' + id + '/export', null, 8000);
      const sc = pkg.scenarios || {}; if (!sc.B) throw new Error('Esta experiência não tem a tática B exportável.');
      const used = ((pkg.provenance && pkg.provenance.seeds) || []); const start = used.length ? Math.max(...used) + 1 : 1;
      const seeds = Array.from({ length: n }, (_, i) => start + i);
      const body = { scenario: sc.B, seeds, event_details: false, name: `${(pkg.summary && pkg.summary.name) || 'Experiência'} · +${n}` };
      if (sc.A) body.reference = sc.A;
      const r = await labCall('POST', '/api/run', body, 15000);
      toast(`${n} jogos em fila no motor do FM27`); await A.labJobs(); await A.labOpen(r.id);
    } catch (e) { toast(e.message); L.err = e.message; render(); }
  },
  async labFmf(id, g) {
    try {
      const pkg = await labCall('GET', '/api/jobs/' + id + '/export', null, 8000);
      const sc = pkg.scenarios && pkg.scenarios[g]; if (!sc) throw new Error('Tática ' + g + ' indisponível');
      const r = await labCall('POST', '/api/export-fmf', { side: 0, scenario: sc, name: `${(pkg.summary && pkg.summary.name) || 'Tatica'} ${g}`.slice(0, 90) }, 60000);
      const bin = Uint8Array.from(atob(r.file_base64), c => c.charCodeAt(0));
      const a = document.createElement('a'); a.href = URL.createObjectURL(new Blob([bin], { type: 'application/octet-stream' })); a.download = r.filename; document.body.appendChild(a); a.click(); a.remove();
      toast('.fmf exportado e verificado pelo jogo — importe-o no FM27');
    } catch (e) { toast(e.message); }
  },
  async labCatalog() {
    try { const cat = await labCall('GET', '/api/catalog', null, 8000), def = await labCall('GET', '/api/defaults', null, 8000);
      download('tactics-lab-catalogo.json', JSON.stringify({ catalog: cat, defaults: def }, null, 1)); toast('Catálogo exportado'); } catch (e) { toast(e.message); }
  },
  labImport(input) { const f = input.files[0]; if (!f) return; f.text().then(t => { try { ingestJSON(JSON.parse(t), f.name); } catch (e) { toast('JSON inválido'); } }); },
  clearOpp() { ST.opp = EMPTY_SQUAD(); save(); render(); },
  setMyClub(v) { ST.myClub = v || null; save(); render(); },
  autoToggle() { ST.auto.on = !ST.auto.on; save(); render(); if (ST.auto.on) desktopPoll(true); },
  autoDir(v) { ST.auto.dir = v.trim(); ST.auto.mtime = 0; save(); desktopPoll(true); },
  autoNow() { ST.auto.mtime = 0; desktopPoll(true); },
  saveNow() { savePoll(true); },
  saveToggle() { ST.auto.save = !ST.auto.save; save(); render(); if (ST.auto.save) savePoll(); },
  saveDir(v) { ST.auto.saveDir = v.trim(); ST.auto.saveMtime = 0; save(); savePoll(true); },
  // campo animado
  simOpen() { const s = simDefaults(); Object.assign(s, { src: 'tactic', t: null, focus: null, hl: null, title: '' }); A.go('tactic', 'sim'); },
  roleDemo(id) {
    const r = ROLE_BY_ID[id], d = demoTacticForRole(r), s = simDefaults();
    Object.assign(s, { src: 'demo', t: d.t, focus: d.idx, hl: null, title: 'Como joga: ' + r.name, phase: r.phase === 'IP' ? 0 : 4, u: 0 });
    A.go('tactic', 'sim');
  },
  tiDemo(k) {
    const s = simDefaults();
    Object.assign(s, { src: 'demo', t: JSON.parse(JSON.stringify(ST.tactic)), focus: null, hl: k, title: 'Instrução: ' + TI_BY_KEY[k].n, phase: TI_PHASE[k] ?? 0, u: 0 });
    A.go('tactic', 'sim');
  },
  simBack() { Object.assign(ST.sim, { src: 'tactic', t: null, focus: null, hl: null }); render(true); },
  simPlay() { ST.sim.playing = !ST.sim.playing; const b = document.getElementById('sim-play'); if (b) { b.textContent = ST.sim.playing ? '❚❚ Pausar' : '▶ Reproduzir'; b.setAttribute('aria-label', ST.sim.playing ? 'Pausar' : 'Reproduzir'); } },
  simPhase(i) { ST.sim.phase = i; ST.sim.u = 0; simCaption(simTactic()); },
  simSpeed(v) { ST.sim.speed = v; render(); },
  simFocus(i) { ST.sim.focus = ST.sim.focus === i ? null : i; render(); },
  simToggle(k) { ST.sim[k] = !ST.sim[k]; render(); },
  simTI(k, v) { const t = simTactic(), d = TI_BY_KEY[k].def; v = +v; if (v === d) delete t.ti[k]; else t.ti[k] = v; if (ST.sim.src === 'tactic') save(); simRebuild(); },
  simMent(v) { simTactic().ment = v; if (ST.sim.src === 'tactic') save(); simRebuild(); },
  simRole(phase, id) { simTactic().slots[ST.sim.focus][phase === 'IP' ? 'ip' : 'oop'] = id; if (ST.sim.src === 'tactic') save(); render(); },
  // .fmf
  async fmf(input) {
    const f = input.files ? input.files[0] : input; if (!f) return;
    ST.fmf = { loading: true, name: f.name }; render();
    try { ST.fmf = await inspectFmf(f); } catch (e) { ST.fmf = { error: String(e), name: f.name }; }
    render();
  },
  fmfStrings() { if (!ST.fmf) return; download(ST.fmf.name + '.strings.txt', ST.fmf.strings.join('\n'), 'text/plain'); },
  fmfApplyHints() {
    const r = ST.fmf; if (!r) return;
    const m = r.matches.mentality[0]; if (m) ST.tactic.ment = m;
    ST.tactic = newTactic(ST.tactic.ipForm, ST.tactic.oopForm); if (m) ST.tactic.ment = m;
    ST.tactic.name = r.name.replace(/\.fmf$/i, ''); ST.tactic.source = 'fmf';
    save(); ST.sel = null; A.go('tactic', 'analyze'); toast('Tática criada — escolha formações, papéis e instruções como no jogo.');
  },
  // plantel
  squadText(force) { const txt = document.getElementById('sqtext').value; ingestSquadText(txt, 'texto colado', { force }); render(); },
  squadFile(input, force) { const f = input.files[0]; if (!f) return; f.text().then(txt => { ingestSquadText(txt, f.name, { force }); render(); }); },
  sampleSquad() { ST.squad = { ...EMPTY_SQUAD(), players: makeSample(), source: 'exemplo (fictício)' }; ST.myClub = null; ST.sugg = null; autoPickXI(ST.tactic, ST.squad.players); save(); render(); toast('Plantel de exemplo carregado (fictício)'); },
  clearSquad() { ST.squad = EMPTY_SQUAD(); ST.myClub = null; ST.tactic.slots.forEach(s => s.pid = null); ST.sugg = null; save(); render(); },
  runSuggest() { ST.sugg = suggestTactics(players()); render(); },
  // bíblia
  rq(v) { ST.rq = v; render(); const el = document.getElementById('rq'); if (el) { el.focus(); el.setSelectionRange(v.length, v.length); } },
  rg(v) { ST.rg = v; render(); }, rp(v) { ST.rp = v; render(); },
  // dados
  exportAll() { download('fm27tb_backup.json', JSON.stringify({ kind: 'fm27tb-all', v: 1, tactic: ST.tactic, squad: ST.squad, saved: ST.saved }, null, 2)); },
  wipe() { if (!confirm('Apagar tática, plantel e táticas guardadas deste computador?')) return; localStorage.removeItem(STORE_KEY); location.reload(); },
  print() { window.print(); },
  copySheet() { const txt = sheetText(ST.tactic); (navigator.clipboard ? navigator.clipboard.writeText(txt) : Promise.reject()).then(() => toast('Ficha copiada'), () => download((ST.tactic.name || 'tatica') + '.txt', txt, 'text/plain')); },
};
