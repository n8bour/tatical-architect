// ============================================================
// Acções (ligadas aos botões da interface)
// ============================================================
const A = {
  go(v, tab) { ST.view = v; if (tab && ST.tabs[v] !== undefined) ST.tabs[v] = tab; save(); render(true); },
  tab(v, tab) { ST.view = v; ST.tabs[v] = tab; save(); render(true); },
  theme(v) { ST.theme = v; applyTheme(); save(); render(); },
  // tática
  async bridgeCheck() { await FMB.status(true); FMB.pluginChecked = false; render(); },
  // the bridge plugin inside FM: status and install/update (the app carries the DLL)
  async pluginCheck() {
    if (!DESKTOP) return;
    try { FMB.plugin = await window.__TAURI__.core.invoke('plugin_status', { customDir: ST.fmDir || null }); } catch (e) { FMB.plugin = null; }
    if (ST.view === 'settings') render();
  },
  async pluginInstall() {
    try {
      FMB.plugin = await withBusy('plugin', L('A instalar o plugin no FM27', 'Installing the plugin in FM27'), () => window.__TAURI__.core.invoke('plugin_install', { customDir: ST.fmDir || null }));
      toast(L('Plugin instalado. Abra o FM27 (ou reinicie-o) para o carregar.', 'Plugin installed. Start (or restart) FM27 to load it.'));
    } catch (e) { toast(String(e.message || e)); }
    render();
  },
  pluginDir(v) { ST.fmDir = String(v || '').trim(); save(); A.pluginCheck(); },
  async bridgeXI() {
    if (players().length < 11) return toast(L('Precisa de pelo menos 11 jogadores no plantel', 'You need at least 11 players in the squad'));
    if (ST.tactic.slots.every(s => !s.pid)) return toast(L('Escolha o XI primeiro (Tática › Analisar).', 'Pick the XI first (Tactic › Analyse).'));
    try {
      const r = await withBusy('bridge', L('A colocar o XI no FM', 'Placing the XI in FM'), () => FMB.applyXI(ST.tactic, players()));
      toast(L(`XI colocado no FM (${r.placed} trocas)`, `XI placed in FM (${r.placed} swaps)`) + (r.skipped.length ? ' — ' + r.skipped.slice(0, 2).join('; ') : ''));
    } catch (e) { toast(String(e.message || e)); }
    FMB.status(true).then(() => render());
  },
  async bridgeTactic(create) {
    if (players().length < 11) return toast(L('Precisa de pelo menos 11 jogadores no plantel', 'You need at least 11 players in the squad'));
    try {
      ST.tacDone = await withBusy('bridge-tac', L('A aplicar a tática no FM', 'Applying the tactic in FM'), (prog) => FMB.applyTactic(ST.tactic, players(), (t) => prog(null, t), { create: !!create }));
      toast(L('Tática aplicada no FM', 'Tactic applied in FM'));
    } catch (e) { toast(String(e.message || e)); }
    render();
  },
  async bridgeRec() {
    try {
      if (FMB.rec && FMB.rec.on) { const n = await FMB.recStop(); toast(L(`${n} ordens gravadas`, `${n} orders recorded`)); }
      else { await FMB.recStart(); toast(L('A gravar: mude a opção no FM e depois carregue em Parar.', 'Recording: change the option in FM, then press Stop.')); }
    } catch (e) { toast(String(e.message || e)); }
    render();
  },
  bridgeRecCopy() { navigator.clipboard.writeText((FMB.rec && FMB.rec.out) || '').then(() => toast(L('Copiado', 'Copied'))); },
  async bridgeSetPieces() {
    const plan = setPiecePlan(ST.tactic, players(), oppPlayers());
    if (!plan) return toast(L('Escolha o XI primeiro (Tática › Analisar).', 'Pick the XI first (Tactic › Analyse).'));
    try {
      const done = await withBusy('bridge-sp', L('A aplicar as bolas paradas no FM', 'Applying set pieces in FM'), () => FMB.applySetPieces(plan, (t) => busyUpdate('bridge-sp', { detail: t })), { detail: L('No FM, abra Táticas › Bolas paradas', 'In FM, open Tactics › Set Pieces') });
      ST.spDone = done; toast(L('Bolas paradas aplicadas no FM', 'Set pieces applied in FM'));
    } catch (e) { toast(String(e.message || e)); }
    render();
  },
  async bridgeSquad() {
    if (!DESKTOP || BUSY['bridge-sq']) return;
    try {
      const r = await withBusy('bridge-sq', L('A ler o plantel do FM', 'Reading squad from FM'), (prog) => FMB.readSquad((t) => prog(null, t)));
      if (!r.players.length) throw new Error(L('Nenhum jogador com atributos foi lido do FM.', 'No players with attributes were read from FM.'));
      // preserve fmsave-only extras (CA/PA, personality, hidden attrs, contract) by unique_id when available
      const prevByUid = new Map();
      (SCOUT.db ? SCOUT.db.players : []).forEach(p => { if (p.uniq) prevByUid.set(String(p.uniq), p); });
      players().forEach(p => { const u = p.uniq || (p.meta && p.meta.uid); if (u) prevByUid.set(String(u), p); });
      const keep = ['ca', 'pa', 'paRange', 'hid', 'pers', 'value', 'wage', 'end', 'status', 'rep', 'wrep', 'nat', 'loan', 'loanFrom'];
      r.players.forEach(p => {
        const old = p.uniq && prevByUid.get(String(p.uniq));
        if (!old) return;
        keep.forEach(k => { if (old[k] != null) p[k] = old[k]; });
      });
      ST.squad = { players: r.players, warnings: [], mapped: [], source: L('FM ao vivo (ponte)', 'FM live (bridge)'), at: new Date().toISOString() };
      if (r.club) ST.myClub = r.club;
      ST.sugg = null;
      const ids = new Set(r.players.map(p => p.id));
      ST.tactic.slots.forEach(s => { if (s.pid && !ids.has(s.pid)) s.pid = null; });
      if (ST.gen) {
        ST.gen.must = (ST.gen.must || []).filter(id => ids.has(id));
        ST.gen.youth = (ST.gen.youth || []).filter(id => ids.has(id));
        if (ST.gen.star && !ids.has(ST.gen.star)) ST.gen.star = '';
      }
      if (ST.tactic.slots.every(s => !s.pid) && r.players.length >= 11) autoPickXI(ST.tactic, r.players);
      save();
      toast(L(`Plantel actualizado do FM${r.club ? ' — ' + r.club : ''}: ${r.players.length} jogadores`, `Squad refreshed from FM${r.club ? ' — ' + r.club : ''}: ${r.players.length} players`));
    } catch (e) { toast(String(e.message || e)); }
    render();
  },
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
  async scoutLoad(opts = {}) {
    if (BUSY.scout) return; SCOUT.err = '';
    try {
      const info = await window.__TAURI__.core.invoke('latest_save', { customDir: ST.auto.saveDir || null });
      if (!info) throw new Error(L('Não encontrei nenhum save do FM27 — defina a pasta dos saves nas Definições.', 'No FM27 save found — set the saves folder in Settings.'));
      const t0 = performance.now();
      await withBusy('scout', L('A ler todos os jogadores de ', 'Reading every player in ') + info.name, async (prog) => {
        prog(null, L('o fmsave está a ler o save (sobre uma cópia)', 'fmsave is reading the save (from a copy)'));
        const r = await window.__TAURI__.core.invoke('export_save_db', { path: info.path });
        await A.scoutIngest(r.players, { nations: r.nations, info: r.info, save: r.save, cached: r.cached, prog, t0, stay: opts.stay });
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
    const S = scoutState(); S.open = null; if (!o.stay) ST.view = 'scout'; save(); render();
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
  // ---- shortlist and scouting in FM (Tactical Bible Bridge)
  async fmShortlist() {
    const list = ST.shortlist || []; if (!list.length) return toast(L('A lista está vazia: use ☆ nos resultados.', 'The list is empty: use ☆ in the results.'));
    try {
      const r = await withBusy('bridge-sl', L('A juntar a lista à shortlist do FM', 'Adding the list to FM’s shortlist'), (prog) => FMB.pushShortlist(list, (t, p) => prog(p, t)));
      toast(L(`${r.added.length} na shortlist do FM`, `${r.added.length} added to FM’s shortlist`) + (r.missing.length ? L(` · não encontrados: ${r.missing.join(', ')}`, ` · not found: ${r.missing.join(', ')}`) : ''));
    } catch (e) { toast(String(e.message || e)); }
  },
  focusOpen() { if (!SCOUT.db) return; scoutState().focus = focusDefaults(scoutResults()); render(); },
  focusClose() { scoutState().focus = null; render(); },
  focusSet(k, v) { const d = scoutState().focus; if (!d) return; d[k] = v; if (k === 'ageMin' && d.ageMax < v) d.ageMax = v; if (k === 'ageMax' && d.ageMin > v) d.ageMin = v; if (k !== 'name') render(); },
  focusArea(nat) { const d = scoutState().focus; if (!d) return; d.areas = d.areas.includes(nat) ? d.areas.filter(x => x !== nat) : [...d.areas, nat]; render(); },
  async focusCreate() {
    const d = scoutState().focus; if (!d || !d.areas.length) return;
    const o = { name: d.name || 'Tactical Bible', ageMin: d.ageMin, ageMax: d.ageMax, caStars: d.caStars, paStars: d.paStars,
      areas: d.areas.map(n => ({ id: FMB_IDS.focus.nation(n), label: SCOUT.nations[n] || '' })) };
    try {
      const made = await withBusy('bridge-focus', L('A criar a observação no FM', 'Creating the scouting assignment in FM'), (prog) => FMB.createFocus(o, (t) => prog(null, t)));
      scoutState().focus = null; toast(L(`Recruitment Focus “${made.name}” criado no FM`, `Recruitment Focus “${made.name}” created in FM`));
    } catch (e) { toast(String(e.message || e)); }
    render();
  },
  scoutStar(id) {
    ST.shortlist = ST.shortlist || []; const i = ST.shortlist.findIndex(x => x.id === id);
    if (i >= 0) ST.shortlist.splice(i, 1);
    else { const p = SCOUT.db && SCOUT.db.players.find(x => x.id === id); if (p) ST.shortlist.push({ id: p.id, uniq: p.uniq, nat: p.nat, name: p.name, age: p.age, club: p.club, posRaw: p.posRaw, ca: p.ca, pa: p.pa, value: p.value, wage: p.wage, end: p.end, pos: p.pos, natPos: p.natPos, a: {}, pers: {}, hid: {} }); }
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
  /** Next opponent from the running FM (bridge): FM gives the fixture and the opponent's players (unique_id), the save
   *  database gives their attributes. silent: background check, only acts if the opponent changed. */
  async oppFromFM(silent) {
    if (!DESKTOP || BUSY['bridge-opp'] || BUSY.scout) return;
    const st = FMB.state || {}; if (!(st.ok && st.ready)) { if (!silent) toast(L('Abra o FM27 com o plugin (Definições › Integrações).', 'Open FM27 with the plugin (Settings › Integrations).')); return; }
    try {
      const nx = await withBusy('bridge-opp', L('A procurar o próximo adversário no FM', 'Finding the next opponent in FM'), () => FMB.nextOpponent());
      if (!nx) { if (!silent) toast(L('O FM não tem um próximo jogo marcado.', 'FM has no next match scheduled.')); return; }
      if (silent && ST.opp.fmTeam === nx.team && oppPlayers().length) return;
      if (!SCOUT.db) await A.scoutLoad({ stay: true });
      if (!SCOUT.db) return;
      const by = new Map(SCOUT.db.players.filter(p => p.uniq).map(p => [String(p.uniq), p]));
      const ps = nx.uniqs.map(u => by.get(u)).filter(Boolean).map(p => ({ ...p, club: nx.name, meta: { ...(p.meta || {}), club: nx.name } }));
      if (ps.length < 11) throw new Error(L(`Só encontrei ${ps.length} jogadores do ${nx.name} no save: grave o jogo e volte a tentar.`, `Only found ${ps.length} ${nx.name} players in the save: save the game and try again.`));
      ST.opp = { ...EMPTY_SQUAD(), players: ps, source: nx.name + L(' (próximo jogo, do FM)', ' (next match, from FM)'), at: new Date().toISOString(), fmTeam: nx.team };
      ST._oppRep = null; save();
      toast(L(`Próximo adversário: ${nx.name} (${ps.length} jogadores)`, `Next opponent: ${nx.name} (${ps.length} players)`));
    } catch (e) { if (!silent) toast(String(e.message || e)); }
    render();
  },
  setMyClub(v) { ST.myClub = v || null; save(); render(); },
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
