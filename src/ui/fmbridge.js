// ============================================================
// Tactical Bible Bridge — optional BepInEx plugin inside FM27 (127.0.0.1:27029, app only).
// ALL FM-specific ids live in FMB_IDS below: after an FM27 patch this is the only block to re-check.
// Capture new ids with the plugin's EVENTS command (do the action by hand in FM, then read what FM sent).
// ============================================================
const FMB_IDS = {
  builtFor: 'FM27 27.1 (verified with Tactical Bible Bridge 0.4)',
  prop: { name: 1851878757, slots: 1919628150, selection: 1098208873 },
  ev: { place: 826434931 },
  key: { slot: 1414752632, pos: 1835167823, person: 1886157170, sel: 1883858041, personId: 1685218662, number: 1886680684, payload: 1903577654, selIdx: 1635149634 },
  ref: {
    teamTraining: 'FM.UI.TeamTrainingReference', tactic: 'FM.UI.TeamTacticReference', calendar: 'FM.UI.CalendarReference',
    teamTrainingKeys: { manager: 1348620377, team: 1416914273 }, calendarKeys: { manager: 1214805357, team: 1415930221 },
  },
  tacticPairs: (c) => `825307168=126;1198877028=1;1651271022=1;1685218662=${c.team}`,
  // set pieces (captured by fm27-assistant: corner and direct free-kick takers per foot, corner technique)
  sp: {
    pairs: (c) => `825307168=93;1198877028=1;1348620377=${c.manager};1685218662=${c.team}`,
    ev: { remove: 910380099, addLeft: 1213227378, addRight: 1416066355, technique: 927877996 },
    prop: { type: 1668375608 },
    key: { kind: 1398883404, list: 961376122 },
    kind: { corner: 5, freeKick: 6 },
    technique: { inswing: 0, outswing: 1 },
    // corner "aim for" (recorded in FM27 EN): record { label, number }. The plain-number form of the same event is another setting.
    ev2: { target: 1249537108 },
    target: { mixed: 0, short: 1, nearPost: 2, farPost: 3, edge: 4, centre: 5 },
    targetLabel: { 0: 'Mixed', 1: 'Short', 2: 'Near Post', 3: 'Far Post', 4: 'Edge of Area', 5: 'Centre' },   // FM English
    // ---- routines (corner instructions per slot). Read with READAT auto (plugin ≥ 0.3, Set Pieces screen open in FM).
    routine: {
      prop: 1867869768,                       // the routine shown on FM's Set Pieces screen (tab + attacking/defending)
      kind: 1398883404, side: 1919054668,     // routine kind (5 att corners, 0 def corners, 6 att FK, 1 def FK, 9 throw-ins); side 1 att / 2 def
      inBox: 1750096198, outBox: 945304884,   // lists of groups
      group: 1114920816, slots: 1752528242,   // group id; its slots
      slot: 1883858041, name: 1851878757, person: 1886157170, current: 1750627616, options: 1833723970, optItem: 1383036740, option: 1750627616,
      ev: 1633039705,                         // set one slot's instruction: { label, { group, slot, option, kind, 1279880290: 0 } }
      flag: 1279880290,
    },
    kinds: { attCorner: 5, defCorner: 0, attFK: 6, defFK: 1, throwIn: 9 },
    // app plan role -> FM option ids, in order of preference (ids are language-independent; verified FM27 EN, see bridge/README.md)
    attRole: { far: [6, 9, 4], near: [5, 8, 4], centre: [4], spot: [4, 11], edge: [7, 10], short: [12], back: [2, 3] },
    defRole: { zn: [7, 3], zc: [9, 8, 10], zf: [11, 4], m: [6], de: [12], o0: [2, 13, 14], o1: [14, 13, 2] },
    // decoded, not wired yet
    todo: {
      penaltyMove: 1649488964,   // { 1277188162: { sel: currentIndex, person }, 1782997593: newIndex }
      listMove: 1144009829,      // { 1277188162: { kind, list, sel: currentIndex, person }, 1782997593: newIndex } (kind 9 = long throws)
      slotMove: 1449477207,      // move an instruction to another group/position (see bridge/README.md)
    },
  },
  // FM position bits (low 20 bits) -> app slot base
  posBits: [[1, 'GK'], [2, 'SW'], [4, 'DR'], [8, 'DL'], [16, 'DC'], [32, 'WBR'], [64, 'WBL'], [128, 'DM'], [256, 'MR'], [512, 'ML'], [1024, 'MC'], [2048, 'AMR'], [4096, 'AML'], [8192, 'AMC'], [16384, 'ST']],
};
const FMB = {
  state: null,   // { ok, ready, version, fm, err }
  async call(cmd, timeoutMs) {
    if (!DESKTOP) throw new Error('Só na app instalada');
    const r = await window.__TAURI__.core.invoke('bridge_call', { cmd, timeoutMs: timeoutMs || 15000 });
    if (r && typeof r === 'object') { FMB.port = r.port; return r.reply; }
    return r;
  },
  // which plugin answered: ours (27029) or fm27-assistant's FM27Bridge (27027, same protocol)
  pluginName() { return FMB.port === 27027 ? 'FM27Bridge (fm27-assistant)' : 'Tactical Bible Bridge'; },
  async ok(cmd, timeoutMs) {
    const r = String(await FMB.call(cmd, timeoutMs) || '');
    if (!r.startsWith('OK')) throw new Error(r || 'no reply');
    return r.length > 3 ? r.slice(3) : '';
  },
  // checks at most every 15 s unless forced (views call it on render: without the throttle it would loop render -> check -> render)
  async status(force) {
    if (!DESKTOP) return (FMB.state = { ok: false, err: 'desktop' });
    if (FMB.checking || (!force && FMB.checkedAt && Date.now() - FMB.checkedAt < 15000)) return FMB.state;
    FMB.checking = true;
    try { return await FMB._status(); } finally { FMB.checking = false; FMB.checkedAt = Date.now(); }
  },
  async _status() {
    try {
      const p = String(await FMB.call('PING', 2500) || '');
      if (!p.startsWith('PONG')) throw new Error(p || 'no reply');
      let version = '', fm = '';
      try { [version, fm] = (await FMB.ok('VERSION', 2500)).split(' '); } catch (e) {}   // FM27Bridge has no VERSION
      // ours answers "PONG ready", FM27Bridge "PONG prêt" (or "PONG (attente …)" while FM loads)
      return (FMB.state = { ok: true, ready: /ready|prêt/.test(p), version, fm, plugin: FMB.pluginName(), port: FMB.port });
    } catch (e) { return (FMB.state = { ok: false, err: String(e.message || e) }); }
  },
  // reqs: [[type, pairs, prop]] -> [json|null]. at = 'auto': read under the node of FM's own open screen (plugin ≥ 0.3),
  // needed for data that depends on the screen's view (e.g. the set-piece routine).
  async read(reqs, at) {
    const out = [];
    for (let i = 0; i < reqs.length; i += 60) {
      const chunk = reqs.slice(i, i + 60);
      const r = await FMB.ok((at ? `READAT ${at} ` : 'READ ') + chunk.map(q => `FM.UI.${q[0]}|${q[1]}|${q[2]}`).join(';;'), 20000);
      const parts = r.split('\u001e');
      chunk.forEach((_, j) => {
        let p = parts[j] || 'ERR';
        if (p.startsWith('ERR')) return out.push(null);
        try { out.push(JSON.parse(p.replace(/[\u0000-\u001f]/g, ' '))); } catch (e) { out.push(null); }
      });
    }
    return out;
  },
  send(ev, refType, pairs, rec) { return FMB.ok(`SENDJ ${ev} FM.UI.${refType} ${pairs} ${JSON.stringify(rec)}`); },
  // ---- recorder: capture what FM sends while you change a setting by hand (to add new actions / re-check after a patch)
  async recStart() { await FMB.ok('RECORD 1'); await FMB.ok('EVENTS'); FMB.rec = { on: true, since: Date.now(), out: '' }; },
  async recStop() {
    const r = await FMB.ok('EVENTS');
    const evs = r ? r.split('\u001e').filter(Boolean).map(x => { try { return JSON.parse(x); } catch (e) { return x; } }) : [];
    FMB.rec = { on: false, out: JSON.stringify({ fm: FMB.state && FMB.state.fm, plugin: FMB.state && FMB.state.version, events: evs }, null, 1) };
    return evs.length;
  },
  // ---- helpers over the plugin's JSON format
  field(rec, key) { const e = rec && rec.entries && rec.entries.find(x => x[0] === key); return e ? e[2] : null; },
  num(v, d) { const n = parseFloat(v); return isFinite(n) ? Math.round(n) : (d === undefined ? NaN : d); },
  pair(ref, key) { const p = ref && ref.pairs && ref.pairs.find(x => x[0] === key); return p ? p[1] : 0; },
  items(list) { return Array.isArray(list) ? list.filter(x => Array.isArray(x) && x.length === 2).map(x => x[1]) : []; },
  position(bits) { const b = Number(bits) & 0xFFFFF; const h = FMB_IDS.posBits.find(([bit]) => (b & bit) !== 0); return h ? h[1] : 'MC'; },
  async context() {
    const refs = await FMB.ok('REFS');
    const map = {};
    refs.split(' | ').forEach(part => {
      const eq = part.indexOf('='); if (eq < 0) return;
      let type = part.slice(0, eq).trim(); if (type.startsWith('session=')) type = type.slice(type.indexOf(' ') + 1);
      const d = {}; part.slice(eq + 1).split(';').forEach(kv => { const [k, v] = kv.split('='); if (k && v !== undefined) d[k] = +v; });
      map[type] = d;
    });
    const R = FMB_IDS.ref, tt = map[R.teamTraining], cal = map[R.calendar];
    if (tt && tt[R.teamTrainingKeys.team]) return { manager: tt[R.teamTrainingKeys.manager], team: tt[R.teamTrainingKeys.team] };
    if (cal && cal[R.calendarKeys.team]) return { manager: cal[R.calendarKeys.manager], team: cal[R.calendarKeys.team] };
    throw new Error(L('Jogo ainda não reconhecido: abra uma vez o ecrã Treino no FM.', 'Game not recognised yet: open the Training screen in FM once.'));
  },
  // ---- active tactic in FM (slots + who is in each)
  async readTactic(c) {
    const K = FMB_IDS.key, P = FMB_IDS.prop, pairs = FMB_IDS.tacticPairs(c);
    const [name, slots, sel] = await FMB.read([['TeamTacticReference', pairs, P.name], ['TeamTacticReference', pairs, P.slots], ['TeamTacticReference', pairs, P.selection]]);
    const st = { name: typeof name === 'string' ? name : '', slots: [], selIndex: {} };
    FMB.items(slots).forEach(r => {
      const index = FMB.num(FMB.field(r, K.slot), -1); if (index < 0) return;
      st.slots.push({ index, pos: FMB.position(FMB.field(r, K.pos)), person: FMB.pair(FMB.field(r, K.person), K.personId) });
    });
    FMB.items(sel).forEach(r => {
      const person = FMB.pair(FMB.field(r, K.person), K.personId), s = FMB.num(FMB.field(r, K.sel), -1);
      if (person && s >= 0) st.selIndex[person] = s;
    });
    return st;
  },
  // FM person id <-> player uid (the squad comes from the save, so the uid is the link)
  async personsOf(c) {
    const K = FMB_IDS.key;
    const tp = `825307168=128;1198877028=1;${FMB_IDS.ref.teamTrainingKeys.manager}=${c.manager};${FMB_IDS.ref.teamTrainingKeys.team}=${c.team}`;
    const list = await FMB.read([['TeamTrainingReference', tp, '1432578628']]);
    const persons = [...new Set(FMB.items(list[0]).map(x => FMB.pair(x, K.personId)).filter(Boolean))];
    const vals = await FMB.read(persons.map(p => ['PersonReference', `825307168=28;1198877028=1;${K.personId}=${p}`, '1970170212']));
    const map = {};
    persons.forEach((p, i) => { const uid = FMB.num(vals[i], 0); if (uid > 0) map[uid] = p; });
    return map;
  },
  /** Replaces FM's two taker lists (left foot, right foot) for one set-piece kind. */
  async setTakers(c, kind, left, right) {
    const S = FMB_IDS.sp, K = FMB_IDS.key, pairs = S.pairs(c);
    const personRef = (x) => ({ type: 'FM.UI.PersonReference', pairs: [[825307168, 28], [1198877028, 1], [K.personId, x]] });
    const entry = (list, index, person) => ({ type: 'SI.Bindable.DynamicReference', entries: [
      [S.key.kind, 'SI.Core.DynamicNumber', String(kind)], [S.key.list, 'SI.Core.DynamicNumber', String(list)],
      [K.sel, 'SI.Core.DynamicNumber', String(index)], [K.person, 'FM.UI.PersonReference', personRef(person)]] });
    const any = left.concat(right)[0];
    // FM removes by rank: empty each list by removing the first entry up to 8 times (no effect on an empty list)
    for (let list = 0; list < 2; list++) for (let i = 0; i < 8; i++) await FMB.send(S.ev.remove, 'SetPieceReference', pairs, entry(list, 0, any));
    const asList = (ps) => ps.map(x => ['SI.Bindable.DynamicReference', entry(2, x & 0xFFFFFF, x)]);
    if (left.length) await FMB.send(S.ev.addLeft, 'SetPieceReference', pairs, asList(left));
    if (right.length) await FMB.send(S.ev.addRight, 'SetPieceReference', pairs, asList(right));
  },
  /** Applies the set-piece plan: corner + direct free-kick takers per foot, corner technique. Returns lines describing what was done. */
  /** Takers (corners + direct free kicks, per foot), corner technique and aim. Returns lines describing what was done. */
  async applyTakers(c, plan, personOf) {
    const S = FMB_IDS.sp;
    const field = plan.xi.filter(x => x.group !== 'GK' && x.p.meta && personOf[x.p.meta.uid]);
    if (!field.length) throw new Error(L('Nenhum jogador do XI encontrado no FM: o plantel tem de vir do save do clube que treina.', 'No XI player found in FM: the squad must come from the save of the club you manage.'));
    // best 2 per foot (players with "either" foot count for both); without foot data, the best overall go in both lists
    const pick = (score, foot) => {
      const sorted = [...field].sort((a, b) => score(b.p) - score(a.p));
      const withFoot = sorted.filter(x => footOf(x.p) === foot || footOf(x.p) === 'B');
      return (withFoot.length ? withFoot : sorted).slice(0, 2);
    };
    const done = [];
    for (const [kind, score, label] of [[S.kind.corner, SP.corner, L('Cantos', 'Corners')], [S.kind.freeKick, SP.fkDirect, L('Livres directos', 'Direct free kicks')]]) {
      const l = pick(score, 'L'), r = pick(score, 'R');
      await FMB.setTakers(c, kind, l.map(x => personOf[x.p.meta.uid]), r.map(x => personOf[x.p.meta.uid]));
      done.push(`${label}: ${l.map(x => shortName(x.p)).join(', ')} (${L('pé esquerdo', 'left foot')}) · ${r.map(x => shortName(x.p)).join(', ')} (${L('pé direito', 'right foot')})`);
    }
    // corner technique: inswinging when the targets are strong in the air (same threshold as the Set pieces screen)
    const air = (plan.airTop || 0) >= 14;
    await FMB.ok(`WRITE FM.UI.SetPieceReference|${S.pairs(c)}|${S.prop.type} ${JSON.stringify({ type: 'SI.Bindable.DynamicReference', entries: [[FMB_IDS.key.number, 'SI.Core.DynamicNumber', '0'], [FMB_IDS.key.payload, 'System.String', 'Corners']] })}`);
    await new Promise(r => setTimeout(r, 500));
    await FMB.send(S.ev.technique, 'SetPieceReference', S.pairs(c), ['SI.Core.DynamicNumber', String(air ? S.technique.inswing : S.technique.outswing)]);
    done.push(air ? L('Cantos: em curva para dentro', 'Corners: inswinging') : L('Cantos: em curva para fora', 'Corners: outswinging'));
    const tz = air ? S.target.farPost : S.target.short;
    await FMB.send(S.ev2.target, 'SetPieceReference', S.pairs(c), { type: 'SI.Bindable.DynamicReference', entries: [
      [FMB_IDS.key.payload, 'System.String', S.targetLabel[tz]], [FMB_IDS.key.number, 'SI.Core.DynamicNumber', String(tz)]] });
    done.push(air ? L('Cantos: procurar o 2.º poste', 'Corners: aim for the far post') : L('Cantos: curtos', 'Corners: short'));
    return done;
  },
  /** Routines need our plugin (READAT, 0.3+); FM27Bridge (27027) or older plugins can only do the takers. */
  canRoutines() {
    const st = FMB.state || {}; if (FMB.port === 27027 || !st.version) return false;
    const [a, b] = String(st.version).split('.').map(Number); return a > 0 || b >= 3;
  },
  /** The routine shown on FM's Set Pieces screen: { kind, side, slots: [{ group, slot, name, person, option, label, options: [{ id, label }] }] }
   *  or null when the screen isn't open (FM answers with its empty default, kind -1). */
  async readRoutine(c) {
    const R = FMB_IDS.sp.routine;
    const [r] = await FMB.read([['SetPieceReference', FMB_IDS.sp.pairs(c), R.prop]], 'auto');
    const kind = FMB.num(FMB.field(r, R.kind), -1);
    if (!r || kind < 0) return null;
    const optOf = (rec) => { const inner = FMB.field(rec, FMB_IDS.key.number); return { id: FMB.num(FMB.field(inner, R.option), -1), label: FMB.field(rec, FMB_IDS.key.payload) || '' }; };
    const slots = [];
    [R.inBox, R.outBox].forEach(k => FMB.items(FMB.field(r, k)).forEach(g => {
      const group = FMB.num(FMB.field(g, R.group), -1);
      FMB.items(FMB.field(g, R.slots)).forEach(s => {
        const cur = optOf(FMB.field(s, R.current));
        slots.push({ group, slot: FMB.num(FMB.field(s, R.slot), -1), name: FMB.field(s, R.name) || '', person: FMB.pair(FMB.field(s, R.person), FMB_IDS.key.personId),
          option: cur.id, label: cur.label, options: FMB.items(FMB.field(s, R.options)).map(o => optOf(FMB.field(o, R.optItem))).filter(o => o.id >= 0) });
      });
    }));
    return { kind, side: FMB.num(FMB.field(r, R.side), 0), slots };
  },
  /** Sets one slot's instruction. kind = routine kind (5 att corners, 0 def corners): writes don't depend on the view FM shows. */
  setInstruction(c, kind, s, opt) {
    const R = FMB_IDS.sp.routine, N = (k, v) => [k, 'SI.Core.DynamicNumber', String(v)];
    return FMB.send(R.ev, 'SetPieceReference', FMB_IDS.sp.pairs(c), { type: 'SI.Bindable.DynamicReference', entries: [
      [FMB_IDS.key.payload, 'System.String', opt.label],
      [FMB_IDS.key.number, 'SI.Bindable.DynamicReference', { type: 'SI.Bindable.DynamicReference', entries: [
        N(R.group, s.group), N(R.slot, s.slot), N(R.option, opt.id), N(R.kind, kind), N(R.flag, 0)] }]] });
  },
  /** Applies the app's corner plan to the routine FM shows (attacking or defending corners), slot by slot:
   *  FM fills each slot with a player from its own rankings, so each slot gets the instruction the app planned for that player. */
  async applyRoutine(c, plan, routine, personOf) {
    const S = FMB_IDS.sp, att = routine.kind === S.kinds.attCorner;
    const rows = att ? plan.attack : plan.defend, prefs = att ? S.attRole : S.defRole;
    const roleOf = {};     // FM person -> { prefs, row }
    rows.forEach(r => {
      if (r.k === 'taker' || !r.x || !r.x.p.meta) return;
      const person = personOf[r.x.p.meta.uid]; if (!person) return;
      const key = r.k.replace(/^back\d$/, 'back').replace(/^m\d$/, 'm');
      if (prefs[key]) roleOf[person] = { ids: prefs[key], row: r };
    });
    const done = [], kept = [];
    for (const s of routine.slots) {
      const want = roleOf[s.person]; if (!want) continue;
      delete roleOf[s.person];
      const opt = want.ids.map(id => s.options.find(o => o.id === id)).find(Boolean);
      const who = shortName(want.row.x.p);
      if (!opt) { kept.push(`${who} (${want.row.n})`); continue; }
      if (opt.id !== s.option) await FMB.setInstruction(c, routine.kind, s, opt);
      done.push(`${who}: ${opt.label}`);
    }
    const title = att ? L('Canto ofensivo', 'Attacking corner') : L('Canto defensivo', 'Defending corner');
    const out = [`${title}: ${done.join(' · ') || L('nada a mudar', 'nothing to change')}`];
    if (kept.length) out.push(L(`${title}: sem instrução compatível na categoria do FM para `, `${title}: no matching instruction in FM's category for `) + kept.join(', '));
    const missing = Object.values(roleOf).map(v => shortName(v.row.x.p));   // planned players FM didn't put in any slot
    if (missing.length) out.push(L(`${title}: o FM não escalou `, `${title}: FM didn't put `) + missing.join(', ') + L(' nesta rotina (escolhe pelos seus rankings)', ' in this routine (it picks from its own rankings)'));
    return out;
  },
  /** Set pieces in FM: takers, then the attacking and defending corner routines. The routines are read from FM's Set Pieces
   *  screen, so the user opens it and switches Attacking/Defending when asked (say(text) shows the prompt). */
  async applySetPieces(plan, say = () => {}) {
    await FMB.status(true);
    const c = await FMB.context(), S = FMB_IDS.sp, sleep = (ms) => new Promise(r => setTimeout(r, ms));
    const personOf = await FMB.personsOf(c);
    say(L('A aplicar os marcadores', 'Applying the takers'));
    const done = await FMB.applyTakers(c, plan, personOf);
    if (!FMB.canRoutines()) { done.push(L('Rotinas dos cantos: precisam do Tactical Bible Bridge 0.3 ou mais recente.', 'Corner routines: need Tactical Bible Bridge 0.3 or newer.')); return done; }
    const need = new Set([S.kinds.attCorner, S.kinds.defCorner]);
    const t0 = Date.now();
    while (need.size && Date.now() - t0 < 120000) {
      const r = await FMB.readRoutine(c);
      if (r && need.has(r.kind)) { say(r.kind === S.kinds.attCorner ? L('A aplicar o canto ofensivo', 'Applying the attacking corner') : L('A aplicar o canto defensivo', 'Applying the defending corner'));
        done.push(...await FMB.applyRoutine(c, plan, r, personOf)); need.delete(r.kind); continue; }
      say(!r ? L('No FM, abra Táticas › Bolas paradas › Cantos', 'In FM, open Tactics › Set Pieces › Corners')
        : need.has(S.kinds.defCorner) && r.kind === S.kinds.attCorner ? L('No FM, carregue em Defender (cantos)', 'In FM, click Defending (corners)')
        : need.has(S.kinds.attCorner) && r.kind === S.kinds.defCorner ? L('No FM, carregue em Atacar (cantos)', 'In FM, click Attacking (corners)')
        : L('No FM, abra o separador Cantos', 'In FM, open the Corners tab'));
      await sleep(1200);
    }
    if (need.size) done.push(L('Rotinas: o tempo acabou antes de abrir ', 'Routines: timed out before opening ') + [...need].map(k => k === S.kinds.attCorner ? L('o canto ofensivo', 'the attacking corner') : L('o canto defensivo', 'the defending corner')).join(L(' e ', ' and ')) + L(' no FM.', ' in FM.'));
    return done;
  },
  /** Places the app's XI into FM's active tactic, matching by position. Returns { placed, skipped[] }. */
  async applyXI(tactic, squad) {
    const c = await FMB.context(), K = FMB_IDS.key, pairs = FMB_IDS.tacticPairs(c);
    const personOf = await FMB.personsOf(c);
    const base = (code) => code.replace(/^(DC|DM|MC|ST)[LR]$/, '$1');
    let st = await FMB.readTactic(c);
    const wanted = {};   // FM slot index -> person
    const skipped = [];
    const byBase = {};
    tactic.slots.forEach(s => { if (!s.pid) return; (byBase[base(s.ipCode)] = byBase[base(s.ipCode)] || []).push(s); });
    const fmByBase = {};
    st.slots.forEach(s => (fmByBase[s.pos] = fmByBase[s.pos] || []).push(s));
    Object.keys(byBase).forEach(b => {
      const mine = byBase[b].sort((a, z) => (slotInfo(tactic.ipForm, a.ipCode).x) - (slotInfo(tactic.ipForm, z.ipCode).x));
      const theirs = (fmByBase[b] || []).sort((a, z) => a.index - z.index);
      mine.forEach((s, i) => {
        const p = squad.find(x => x.id === s.pid), uid = p && p.meta && p.meta.uid;
        const person = uid && personOf[uid];
        if (!theirs[i]) return skipped.push(`${p ? p.name : s.pid}: ${L('sem casa ' + b + ' na tática do FM', 'no ' + b + ' slot in the FM tactic')}`);
        if (!person) return skipped.push(`${p ? p.name : s.pid}: ${L('não encontrado no FM (uid)', 'not found in FM (uid)')}`);
        wanted[theirs[i].index] = person;
      });
    });
    let placed = 0;
    for (const [idx, person] of Object.entries(wanted)) {
      const now = placed === 0 ? st : await FMB.readTactic(c);   // FM swaps two players: re-read before each order
      const slot = now.slots.find(x => x.index === +idx);
      if (!slot || slot.person === person) continue;
      const sel = now.selIndex[person] !== undefined ? now.selIndex[person] : -1;
      const rec = { type: 'SI.Bindable.DynamicReference', entries: [
        [K.number, 'SI.Core.DynamicNumber', '2'],
        [K.payload, 'SI.Bindable.DynamicReference', { type: 'SI.Bindable.DynamicReference', entries: [
          [K.person, 'FM.UI.PersonReference', { type: 'FM.UI.PersonReference', pairs: [[825307168, 28], [1198877028, 1], [K.personId, person]] }],
          [K.selIdx, 'SI.Core.DynamicNumber', String(sel)],
          [K.sel, 'SI.Core.DynamicNumber', String(person & 0xFFFFFF)],
          [K.slot, 'SI.Core.DynamicNumber', String(+idx)]] }]] };
      try { await FMB.send(FMB_IDS.ev.place, 'TeamTacticReference', pairs, rec); placed++; await new Promise(r => setTimeout(r, 250)); }
      catch (e) { skipped.push(`#${idx}: ${e.message}`); }
    }
    return { placed, skipped };
  },
};

// "Apply in FM" button for a screen (only in the installed app). Connected: primary button. Not connected: grey, opens
// Settings and the tooltip says what is missing. kind: 'xi' (place the XI) | 'sp' (set pieces).
function fmButton(kind) {
  if (!DESKTOP) return '';
  fmbRefresh(ST.view);
  const st = FMB.state || {}, ready = !!(st.ok && st.ready);
  const K = {
    xi: ['bridge', 'A.bridgeXI()', L('Colocar XI no FM', 'Place XI in FM'), L('Põe os jogadores do teu XI na tática activa do FM.', 'Puts your XI into the active tactic in FM.')],
    sp: ['bridge-sp', 'A.bridgeSetPieces()', L('Aplicar no FM', 'Apply in FM'), L('Aplica os marcadores e as rotinas de bolas paradas no FM. No FM, abra Táticas › Bolas paradas.', 'Applies the takers and set-piece routines in FM. In FM, open Tactics › Set Pieces.')],
  }[kind];
  const off = st.ok ? L('O FM está aberto mas ainda não carregou um jogo.', 'FM is open but has not loaded a game yet.')
    : L('Precisa do plugin Tactical Bible Bridge dentro do FM (Definições › Integrações).', 'Needs the Tactical Bible Bridge plugin inside FM (Settings › Integrations).');
  return `<button class="btn fm-btn${ready ? ' primary' : ''}" data-busy="${K[0]}" onclick="${ready ? K[1] : "A.go('settings')"}" title="${esc(ready ? K[3] : off)}"><span class="dot ${ready ? 'ok' : ''}" aria-hidden="true"></span>${K[2]}</button>`;
}

// called from views: refresh the bridge status in the background; re-render only if it changed and we're still on that view
function fmbRefresh(view) {
  const before = JSON.stringify(FMB.state);
  FMB.status().then(st => { if (JSON.stringify(st) !== before && ST.view === view) render(); });
}
