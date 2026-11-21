// ============================================================
// Tactical Bible Bridge — optional BepInEx plugin inside FM27 (127.0.0.1:27029, app only).
// ALL FM-specific ids live in FMB_IDS below: after an FM27 patch this is the only block to re-check.
// Capture new ids with the plugin's EVENTS command (do the action by hand in FM, then read what FM sent).
// ============================================================
const FMB_IDS = {
  builtFor: 'FM27 (ids captured with fm27-assistant 3.1.x)',
  prop: { name: 1851878757, slots: 1919628150, selection: 1098208873 },
  ev: { place: 826434931 },
  key: { slot: 1414752632, pos: 1835167823, person: 1886157170, sel: 1883858041, personId: 1685218662, number: 1886680684, payload: 1903577654, selIdx: 1635149634 },
  ref: {
    teamTraining: 'FM.UI.TeamTrainingReference', tactic: 'FM.UI.TeamTacticReference', calendar: 'FM.UI.CalendarReference',
    teamTrainingKeys: { manager: 1348620377, team: 1416914273 }, calendarKeys: { manager: 1214805357, team: 1415930221 },
  },
  tacticPairs: (c) => `825307168=126;1198877028=1;1651271022=1;1685218662=${c.team}`,
  // FM position bits (low 20 bits) -> app slot base
  posBits: [[1, 'GK'], [2, 'SW'], [4, 'DR'], [8, 'DL'], [16, 'DC'], [32, 'WBR'], [64, 'WBL'], [128, 'DM'], [256, 'MR'], [512, 'ML'], [1024, 'MC'], [2048, 'AMR'], [4096, 'AML'], [8192, 'AMC'], [16384, 'ST']],
};
const FMB = {
  state: null,   // { ok, ready, version, fm, err }
  async call(cmd, timeoutMs) {
    if (!DESKTOP) throw new Error('Só na app instalada');
    return window.__TAURI__.core.invoke('bridge_call', { cmd, timeoutMs: timeoutMs || 15000 });
  },
  async ok(cmd, timeoutMs) {
    const r = await FMB.call(cmd, timeoutMs);
    if (!r.startsWith('OK')) throw new Error(r);
    return r.length > 3 ? r.slice(3) : '';
  },
  async status() {
    if (!DESKTOP) return (FMB.state = { ok: false, err: 'desktop' });
    try {
      const p = await FMB.call('PING', 2500);
      let version = '', fm = '';
      try { [version, fm] = (await FMB.ok('VERSION', 2500)).split(' '); } catch (e) {}
      return (FMB.state = { ok: true, ready: /ready/.test(p), version, fm });
    } catch (e) { return (FMB.state = { ok: false, err: String(e.message || e) }); }
  },
  async read(reqs) {   // reqs: [[type, pairs, prop]] -> [json|null]
    const out = [];
    for (let i = 0; i < reqs.length; i += 60) {
      const chunk = reqs.slice(i, i + 60);
      const r = await FMB.ok('READ ' + chunk.map(q => `FM.UI.${q[0]}|${q[1]}|${q[2]}`).join(';;'), 20000);
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
