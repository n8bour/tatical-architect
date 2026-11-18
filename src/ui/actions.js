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
  saveTactic() { const c = JSON.parse(JSON.stringify(ST.tactic)); c.savedAt = new Date().toISOString(); ST.saved = ST.saved.filter(x => x.name !== c.name); ST.saved.unshift(c); save(); render(); toast(L('Tática guardada: ', 'Tactic saved: ') + c.name); },
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
    save(); render(); toast(L(n + ' instruções aplicadas (a principal de cada jogador — mais instruções = mais complexidade)', n + ' instructions applied (each player’s main one — more instructions = more complexity)')); },
  applyPlan(list) { const rep = ST._oppRep; if (!rep) return; const items = list === 'all' ? rep.plan : [rep.plan[list]];
    items.forEach(x => { const d = TI_BY_KEY[x.ti]; if (x.v === d.def) delete ST.tactic.ti[x.ti]; else ST.tactic.ti[x.ti] = x.v; });
    save(); render(); toast(items.length === 1 ? `${TI_BY_KEY[items[0].ti].n}: ${TI_BY_KEY[items[0].ti].opts[items[0].v]}` : 'Plano aplicado à tática'); },
  applySpTip(i) { const tp = ST._sp && ST._sp.tips[i]; if (!tp || !tp.ti) return; const d = TI_BY_KEY[tp.ti]; if (tp.v === d.def) delete ST.tactic.ti[tp.ti]; else ST.tactic.ti[tp.ti] = tp.v; save(); render(); toast(`${d.n}: ${d.opts[tp.v]}`); },
  genOpt(k, v) { genOpts()[k] = v; save(); render(); },
  genToggle(k, id) { const g = genOpts(); const i = g[k].indexOf(id); if (i >= 0) g[k].splice(i, 1); else if (k !== 'youth' || g[k].length < 3) g[k].push(id); else toast('Máximo 3 jovens'); save(); render(); },
  async genRun() {
    const g = genOpts(); if (BUSY.gen) return;
    const t0 = performance.now();
    g.res = await withBusy('gen', 'A gerar a tática', (prog) => generateTacticAsync(players(), { expect: g.expect, must: g.must, star: g.star || null, youth: g.youth },
      (s) => prog(s.total ? s.done / s.total : null, `${s.label}${s.total > 1 ? ` (${Math.min(s.done + 1, s.total)}/${s.total})` : ''}`)));
    g.pick = 0; save(); render(); toast(L('Tática gerada em ', 'Tactic generated in ') + ((performance.now() - t0) / 1000 < 1 ? Math.round(performance.now() - t0) + ' ms' : ((performance.now() - t0) / 1000).toFixed(1) + ' s'));
  },
  genPick(i) { genOpts().pick = i; save(); render(); },
  genTactic() { const g = genOpts(); const all = [g.res.main, ...g.res.alts]; const t = JSON.parse(JSON.stringify((all[g.pick] || all[0]).t));
    const pm = Object.fromEntries(players().map(p => [p.id, p]));
    t.slots.forEach(s => { const r = recommendPIs(pm[s.pid], ROLE_BY_ID[s.ip], ROLE_BY_ID[s.oop], slotInfo(t.ipForm, s.ipCode).group)[0]; s.pis = r ? [r.pi] : []; });
    t.name = L(`${t.ipForm} — gerada para ${ST.myClub || 'o plantel'}`, `${t.ipForm} — generated for ${ST.myClub || 'the squad'}`); t.source = 'gerador'; return t; },
  genUse(sim, tab) { const t = A.genTactic(); ST.tactic = t; ST.saved = ST.saved.filter(x => x.name !== t.name); ST.saved.unshift(JSON.parse(JSON.stringify(t))); ST.sugg = null; save();
    if (sim) return A.simOpen(); if (tab === 'training') return A.go('training'); A.tab('tactic', tab || 'analyze'); toast('Tática gerada carregada e guardada'); },
  genCopy() { const txt = sheetText(A.genTactic()); (navigator.clipboard ? navigator.clipboard.writeText(txt) : Promise.reject()).then(() => toast('Tática copiada — pronta para colar no fórum/Discord'), () => download('tatica-gerada.txt', txt, 'text/plain')); },
  labSet(k, v) { labState()[k] = v; save(); },
  async labConnect() {
    const LS = labState(); if (BUSY['lab-connect']) return; LS.busy = true; LS.err = ''; render();
    const ports = [LS.port, ...Array.from({ length: 16 }, (_, i) => 8765 + i)].filter((p, i, a) => a.indexOf(p) === i);
    busyStart('lab-connect', 'A procurar o Tactics Lab', { p: 0 });
    for (const [i, p] of ports.entries()) {
      LS.port = p; busyUpdate('lab-connect', { p: i / ports.length, detail: 'porta ' + p });
      try { const s = await labCall('GET', '/api/status', null, 400); if (s && s.engine) { LS.status = s; break; } } catch (e) { /* tenta a seguinte */ }
    }
    LS.busy = false; busyEnd('lab-connect');
    if (!LS.status) { LS.err = 'Não encontrei o Tactics Lab. Abra-o primeiro (com o Steam aberto) e confirme a porta no endereço que ele mostra.'; LS.port = ports[0]; render(); return; }
    save(); await A.labJobs();
  },
  async labJobs() { const LS = labState(); busyStart('lab-jobs', 'A ler as experiências do Tactics Lab'); try { const j = await labCall('GET', '/api/jobs?brief=1'); LS.jobs = (Array.isArray(j) ? j : []).sort((a, b) => (b.created || 0) - (a.created || 0)); LS.err = ''; } catch (e) { LS.err = e.message; LS.status = null; } busyEnd('lab-jobs'); render(); },
  async labOpen(id) { const LS = labState(); try { LS.job = await withBusy('lab-open', 'A abrir a experiência', () => labCall('GET', '/api/jobs/' + id, null, 15000)); LS.err = ''; } catch (e) { LS.err = e.message; } render(); },
  async labCancel(id) { try { await labCall('POST', '/api/cancel', { id }); toast('Cancelamento pedido'); setTimeout(() => A.labOpen(id), 800); } catch (e) { toast(e.message); } },
  async labMore(id) {
    const LS = labState(), n = LS.more || 100; if (BUSY['lab-more']) return;
    busyStart('lab-more', L(`A pedir mais ${n} jogos ao motor do FM27`, `Asking the FM27 engine for ${n} more matches`), { detail: L('a preparar as táticas A e B', 'preparing tactics A and B') });
    try {
      const pkg = await labCall('GET', '/api/jobs/' + id + '/export', null, 8000);
      const sc = pkg.scenarios || {}; if (!sc.B) throw new Error('Esta experiência não tem a tática B exportável.');
      const used = ((pkg.provenance && pkg.provenance.seeds) || []); const start = used.length ? Math.max(...used) + 1 : 1;
      const seeds = Array.from({ length: n }, (_, i) => start + i);
      const body = { scenario: sc.B, seeds, event_details: false, name: `${(pkg.summary && pkg.summary.name) || 'Experiência'} · +${n}` };
      if (sc.A) body.reference = sc.A;
      const r = await labCall('POST', '/api/run', body, 15000);
      busyEnd('lab-more'); toast(`${n} jogos em fila no motor do FM27`); await A.labJobs(); await A.labOpen(r.id);
    } catch (e) { busyEnd('lab-more'); toast(e.message); LS.err = e.message; render(); }
  },
  async labFmf(id, g) {
    if (BUSY['lab-fmf']) return;
    busyStart('lab-fmf', L(`A exportar o .fmf da tática ${g}`, `Exporting tactic ${g}’s .fmf`), { detail: L('o próprio jogo gera e verifica o ficheiro', 'the game itself writes and verifies the file'), slow: L('o motor do FM27 está a serializar e a reler a tática — até 45 s', 'the FM27 engine is serialising and re-reading the tactic — up to 45 s'), slowAfter: 8 });
    try {
      const pkg = await labCall('GET', '/api/jobs/' + id + '/export', null, 8000);
      const sc = pkg.scenarios && pkg.scenarios[g]; if (!sc) throw new Error(L('Tática ' + g + ' indisponível', 'Tactic ' + g + ' unavailable'));
      const r = await labCall('POST', '/api/export-fmf', { side: 0, scenario: sc, name: `${(pkg.summary && pkg.summary.name) || 'Tatica'} ${g}`.slice(0, 90) }, 60000);
      const bin = Uint8Array.from(atob(r.file_base64), c => c.charCodeAt(0));
      const a = document.createElement('a'); a.href = URL.createObjectURL(new Blob([bin], { type: 'application/octet-stream' })); a.download = r.filename; document.body.appendChild(a); a.click(); a.remove();
      toast('.fmf exportado e verificado pelo jogo — importe-o no FM27');
    } catch (e) { toast(e.message); } finally { busyEnd('lab-fmf'); }
  },
  async labCatalog() {
    busyStart('lab-cat', 'A exportar o catálogo do motor');
    try { const cat = await labCall('GET', '/api/catalog', null, 8000), def = await labCall('GET', '/api/defaults', null, 8000);
      download('tactics-lab-catalogo.json', JSON.stringify({ catalog: cat, defaults: def }, null, 1)); toast('Catálogo exportado'); } catch (e) { toast(e.message); } finally { busyEnd('lab-cat'); }
  },
  labImport(input) { const f = input.files[0]; if (!f) return; f.text().then(t => { try { ingestJSON(JSON.parse(t), f.name); } catch (e) { toast('JSON inválido'); } }); },
  lang(l) {
    ST.lang = LANG = l;
    // o relatório do gerador guarda as razões no idioma em que foi feito: recalcula-as no novo idioma
    if (ST.gen && ST.gen.res && players().length >= 11) [ST.gen.res.main, ...ST.gen.res.alts].forEach(r => { const g = generateTIs(r.t, players(), ST.gen.res.expect); r.reasons = g.reasons; r.mentWhy = g.mentWhy; });
    save(); render();
  },
  sheetNames(v) { ST.sheetNames = v; save(); render(); },
  // ---------- tabela e ficha do plantel ----------
  sqMode(m) { ST.sqMode = m; save(); render(); },
  sqSort(k) { const S = ST.sqSort || { k: '', dir: 'desc' }; ST.sqSort = S.k === k ? { k, dir: S.dir === 'asc' ? 'desc' : 'asc' } : { k, dir: ['name', 'age', 'end', 'value', 'wage'].includes(k) ? 'asc' : 'desc' }; save(); render(); },
  squadOpen(id) { ST.squadOpen = id; render(); },
  squadClose() { ST.squadOpen = null; render(); },
  // ---------- prospeção ----------
  async scoutLoad() {
    if (BUSY.scout) return; SCOUT.err = '';
    try {
      const info = await window.__TAURI__.core.invoke('latest_save', { customDir: ST.auto.saveDir || null });
      if (!info) throw new Error(L('Não encontrei nenhum save do FM27 — defina a pasta dos saves nas Definições.', 'No FM27 save found — set the saves folder in Settings.'));
      const t0 = performance.now();
      await withBusy('scout', L('A ler todos os jogadores de ', 'Reading every player in ') + info.name, async (prog) => {
        prog(null, L('o fmsave está a ler o save (sobre uma cópia)', 'fmsave is reading the save (from a copy)'));
        const r = await window.__TAURI__.core.invoke('export_save_db', { path: info.path });
        await A.scoutIngest(r.players, { nations: r.nations, info: r.info, save: r.save, cached: r.cached, prog, t0 });
      }, { slow: L('a primeira leitura da base de dados inteira pode demorar 1–3 minutos', 'the first read of the whole database can take 1–3 minutes'), slowAfter: 15 });
    } catch (e) { SCOUT.err = String((e && e.message) || e); toast(L('Não consegui ler a base de dados', 'Could not read the database')); }
    render();
  },
  async scoutIngest(text, o = {}) {
    const t0 = o.t0 || performance.now();
    const db = await parseScoutDB(text, { onProgress: (f, n) => o.prog && o.prog(f, L(`a interpretar ${n.toLocaleString(LOCALE())} jogadores`, `parsing ${n.toLocaleString(LOCALE())} players`)) });
    SCOUT.db = db; SCOUT.nations = parseNations(o.nations || ''); SCOUT._val = null; SCOUT.cache = null; SCOUT.cacheKey = '';
    const d = o.info ? gameDateFromInfo(o.info) : null; SCOUT.date = d || inferGameDate(db.players); SCOUT.dateGuess = !d;
    SCOUT.save = o.save || L('ficheiro', 'file'); SCOUT.cached = !!o.cached; SCOUT.ms = performance.now() - t0;
    const S = scoutState(); S.open = null; ST.view = 'scout'; save(); render();
    toast(L(`${db.players.length.toLocaleString(LOCALE())} jogadores carregados`, `${db.players.length.toLocaleString(LOCALE())} players loaded`));
  },
  scoutPreset(k) {
    const S = scoutState(), pr = scoutPresets(scoutCtx()).find(x => x.k === k); if (!pr) return;
    S.F = JSON.parse(JSON.stringify(pr.F)); S.preset = k; S.page = 0; S.weak = ''; S.tab = 'results';
    if (pr.weakSpot) { const w = weakestSlot(); if (w) { S.F.pos = w.group; S.F.posLevel = 'nat'; S.weak = w.s.ipCode; } }
    save(); render();
  },
  scoutF(k, v) { const S = scoutState(); S.F[k] = v; S.preset = ''; S.page = 0; save(); render(); },
  scoutAttrAdd() { const S = scoutState(); S.F.attrs.push({ k: '', min: '' }); save(); render(); },
  scoutAttr(i, f, v) { const S = scoutState(); if (S.F.attrs[i]) S.F.attrs[i][f] = v; S.preset = ''; S.page = 0; save(); render(); },
  scoutAttrDel(i) { const S = scoutState(); S.F.attrs.splice(i, 1); S.page = 0; save(); render(); },
  scoutReset() { const S = scoutState(); S.F = SCOUT_DEFAULT(); S.preset = ''; S.page = 0; S.weak = ''; save(); render(); },
  scoutSort(k) { const S = scoutState(); if (S.F.sort === k) S.F.dir = S.F.dir === 'asc' ? 'desc' : 'asc'; else { S.F.sort = k; S.F.dir = ['name', 'age', 'value', 'wage', 'end'].includes(k) ? 'asc' : 'desc'; } S.page = 0; save(); render(); },
  scoutPage(n) { scoutState().page = Math.max(0, n); save(); render(); document.getElementById('main').scrollTop = 0; },
  scoutTab(t) { scoutState().tab = t; save(); render(); },
  scoutOpen(id) { scoutState().open = id; render(); },
  scoutClose() { scoutState().open = null; render(); },
  scoutStar(id) {
    ST.shortlist = ST.shortlist || []; const i = ST.shortlist.findIndex(x => x.id === id);
    if (i >= 0) ST.shortlist.splice(i, 1);
    else { const p = SCOUT.db && SCOUT.db.players.find(x => x.id === id); if (p) ST.shortlist.push({ id: p.id, name: p.name, age: p.age, club: p.club, posRaw: p.posRaw, ca: p.ca, pa: p.pa, value: p.value, wage: p.wage, end: p.end, pos: p.pos, natPos: p.natPos, a: {}, pers: {}, hid: {} }); }
    save(); render();
  },
  scoutExport() {
    const S = scoutState(), list = S.tab === 'list' ? (ST.shortlist || []) : (SCOUT.db ? scoutResults() : []);
    const head = ['name', 'age', 'club', 'positions', 'ca', 'pa', 'value', 'wage', 'contract_end'];
    const q = (v) => { const s = v == null ? '' : String(v); return /[",;\n]/.test(s) ? '"' + s.replace(/"/g, '""') + '"' : s; };
    const csv = [head.join(','), ...list.map(p => [p.name, p.age, p.club, p.posRaw, p.ca, p.pa, p.value, p.wage, p.end].map(q).join(','))].join('\n');
    download(S.tab === 'list' ? 'shortlist.csv' : 'scouting.csv', csv, 'text/csv'); toast(L(`${list.length} jogadores exportados`, `${list.length} players exported`));
  },
  planK(k) { ST.planK = k; save(); render(); },
  planTactic() { const r = ST._plan || gamePlan(ST.tactic, players(), ST.planK || 'chase', { opp: oppPlayers() }); return JSON.parse(JSON.stringify(r.t)); },
  planSave() { const t = A.planTactic(); ST.saved = ST.saved.filter(x => x.name !== t.name); ST.saved.unshift(t); save(); render(); toast(L('Plano guardado: ', 'Plan saved: ') + t.name); },
  planUse() { const t = A.planTactic(); ST.saved = ST.saved.filter(x => x.name !== t.name); ST.saved.unshift(JSON.parse(JSON.stringify(t))); ST.tactic = t; save(); A.tab('tactic', 'analyze'); toast(L('Plano carregado no analisador', 'Plan loaded into the analyser')); },
  planCopy() { const r = ST._plan; if (!r) return; const lines = [r.t.name, ''];
    r.changes.forEach(c => lines.push(c.kind === 'ti' || c.kind === 'ment' ? `${c.label}: ${c.from} → ${c.to}` : c.kind === 'role' ? `${c.code} (${c.phase}) ${c.player}: ${c.from} → ${c.to}` : `${c.code} ${c.player}: ${c.removed ? '−' : '+'} ${c.to}`));
    r.tweaks.forEach(x => lines.push('Tactical Tweak: ' + x.n)); r.subs.forEach(x => lines.push(`${L('Substituição', 'Substitution')} ${x.code}: ${x.out} → ${x.in}`));
    const txt = lines.join('\n'); (navigator.clipboard ? navigator.clipboard.writeText(txt) : Promise.reject()).then(() => toast(L('Plano copiado', 'Plan copied')), () => download('plano.txt', txt, 'text/plain')); },
  navMini() { ST.ui.navMini = !ST.ui.navMini; save(); render(); },
  collapseAll(c) { document.querySelectorAll('#main .panel.collapsible').forEach(p => { if (p.classList.contains('collapsed') !== c) p.querySelector('.pc-toggle').click(); }); },
  clipToggle(id) { ST.openClips = ST.openClips || {}; if (ST.openClips[id]) delete ST.openClips[id]; else ST.openClips[id] = 1; render(); },
  clearOpp() { ST.opp = EMPTY_SQUAD(); save(); render(); },
  setMyClub(v) { ST.myClub = v || null; save(); render(); },
  autoToggle() { ST.auto.on = !ST.auto.on; save(); render(); if (ST.auto.on) desktopPoll(true); },
  autoDir(v) { ST.auto.dir = v.trim(); ST.auto.mtime = 0; save(); desktopPoll(true); },
  autoNow() { ST.auto.mtime = 0; desktopPoll(true); },
  saveNow() { savePoll(true); },
  pickFile() { const i = document.createElement('input'); i.type = 'file'; i.accept = '.csv,.tsv,.txt,.html,.htm,.json,.fmf'; i.onchange = () => { if (i.files[0]) ingestFile(i.files[0]); }; i.click(); },
  saveToggle() { ST.auto.save = !ST.auto.save; save(); render(); if (ST.auto.save) savePoll(); },
  saveDir(v) { ST.auto.saveDir = v.trim(); ST.auto.saveMtime = 0; save(); savePoll(true); },
  // campo animado
  simOpen() { const s = simDefaults(); Object.assign(s, { src: 'tactic', t: null, focus: null, hl: null, title: '' }); A.go('tactic', 'sim'); },
  roleDemo(id) {
    const r = ROLE_BY_ID[id], d = demoTacticForRole(r), s = simDefaults();
    Object.assign(s, { src: 'demo', t: d.t, focus: d.idx, hl: null, title: L('Como joga: ', 'How it plays: ') + r.name, phase: r.phase === 'IP' ? 0 : 4, u: 0 });
    A.go('tactic', 'sim');
  },
  tiDemo(k) {
    const s = simDefaults();
    Object.assign(s, { src: 'demo', t: JSON.parse(JSON.stringify(ST.tactic)), focus: null, hl: k, title: L('Instrução: ', 'Instruction: ') + TI_BY_KEY[k].n, phase: TI_PHASE[k] ?? 0, u: 0 });
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
    try { ST.fmf = await withBusy('fmf', L('A analisar ', 'Analysing ') + f.name, () => inspectFmf(f)); } catch (e) { ST.fmf = { error: String(e), name: f.name }; }
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
  sampleSquad() { ST.squad = { ...EMPTY_SQUAD(), players: makeSample(), source: L('exemplo (fictício)', 'sample (fictional)') }; ST.myClub = null; ST.sugg = null; autoPickXI(ST.tactic, ST.squad.players); save(); render(); toast('Plantel de exemplo carregado (fictício)'); },
  clearSquad() { ST.squad = EMPTY_SQUAD(); ST.myClub = null; ST.tactic.slots.forEach(s => s.pid = null); ST.sugg = null; save(); render(); },
  runSuggest() { ST.sugg = suggestTactics(players()); render(); },
  // bíblia
  rq(v) { ST.rq = v; render(); const el = document.getElementById('rq'); if (el) { el.focus(); el.setSelectionRange(v.length, v.length); } },
  rg(v) { ST.rg = v; render(); }, rp(v) { ST.rp = v; render(); },
  // dados
  exportAll() { download('fm27tb_backup.json', JSON.stringify({ kind: 'fm27tb-all', v: 1, tactic: ST.tactic, squad: ST.squad, saved: ST.saved }, null, 2)); },
  wipe() { if (!confirm(T('Apagar tática, plantel e táticas guardadas deste computador?'))) return; localStorage.removeItem(STORE_KEY); location.reload(); },
  print() { window.print(); },
  copySheet() { const txt = sheetText(ST.tactic); (navigator.clipboard ? navigator.clipboard.writeText(txt) : Promise.reject()).then(() => toast('Ficha copiada'), () => download((ST.tactic.name || 'tatica') + '.txt', txt, 'text/plain')); },
};
