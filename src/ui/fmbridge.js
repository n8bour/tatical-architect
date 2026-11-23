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
  // ---- tactic (verified FM27 EN 22 Nov, see bridge/README.md): active tactic = TeamTacticReference
  tactic: {
    pairs: (c) => `825307168=126;1198877028=1;1651271022=1;1685218662=${c.team}`,
    slots: 1919628150, slotIdx: 1414752632, ipPos: 1835167823, oopPos: 927294289,
    ipRoles: 930166885, ipRole: 1698852728, oopRoles: 1798391651, oopRole: 1349481322,
    move: 21, moveFrom: 929121399, moveTo: 944851554, moveIP: 1769491558, moveSlot: 857762170,
    setRole: 1467503190, roleRec: 1349481322, roleName: 909713440, roleIP: 1265792339,
    mentality: 1162170689,
    menu: 1949648203, menuCopy: 0, menuSub: 876950356,   // tactic menu: item 0 (Copy Tactic) submenu = every tactic
    maxTactics: 3,   // FM27 keeps 3 tactics
    wizard: { open: 1412787285, next: 1515149153, finish: 879515246, style: 1381059181, styleBase: 10 },   // styleBase 10 = Clean Slate
    name: 1851878757,
    // player instructions (verified FM27): arrows per category id, which CYCLE (wrap around); revert = role defaults (all Standard)
    // player instructions (verified FM27, user checked the panel): arrows per category id; the options CYCLE (wrap). The ‹ event works on ANY tile,
    // the › event only on the tile the user last clicked by hand: so we only use ‹. Revert = role defaults (every tile back to Standard).
    piEnabled: true,
    piPrev: 1668445234, piCat: 1449616730, piRevert: 1984657952, piRevertRec: 1936487524,
    // app player instruction -> [in possession?, FM category id, number of ‹ presses from Standard]. Option order from Standard (‹):
    //  0 width: Sit Narrower, Stay Wider · 2 dribbling: Less, More · 3 runs: More, Fewer · 4 freedom: Roam From Position · 6 shooting: Less, More
    //  8 passing risk: Fewer, More · 9 crossing target: Target Forward, Near Post, Centre, Far Post · 10 cross from: Byline, Cross From Deep
    //  11 trigger press: Less, More Often · 12 marking: Mark Tighter · 13 tackling: Ease Off Tackles, Tackle Harder
    pi: {
      'Sit narrower': [true, 0, 1], 'Stay wider': [true, 0, 2], 'Dribble less': [true, 2, 1], 'Dribble more': [true, 2, 2],
      'Get further forward': [true, 3, 1], 'Roam from position': [true, 4, 1], 'Shoot less': [true, 6, 1], 'Shoot more': [true, 6, 2],
      'Take fewer risks': [true, 8, 1], 'Take more risks': [true, 8, 2], 'Target Near Post': [true, 9, 2], 'Target Centre': [true, 9, 3],
      'Target Far Post': [true, 9, 4], 'Cross from byline': [true, 10, 1], 'Cross from deep': [true, 10, 2],
      'Press less': [false, 11, 1], 'Press more': [false, 11, 2], 'Mark tighter': [false, 12, 1], 'Ease off tackles': [false, 13, 1], 'Tackle harder': [false, 13, 2],
    },
    tiPrev: 1249276755, tiNext: 862931569,   // value = instruction id; FM27's ‹ / › arrows (they stop at the ends)
    left: 1048576, right: 2097152,           // added to central positions (DCL = DC + left); verified on FM's screen
    roles: {acb: '16384', am_ip: '512', am_oop: '1073741824', ap: '131072', awb: '274877906944', b2b: '65536', b2bp: '70368744177664', bpcb: '16777216', bpgk: '4096', cb_ip: '2', cb_oop: '8', ccb: '32', cf_ip: '2048', cf_oop: '4398046511104', chf: '2147483648', chm: '72057594037927936', cm_ip: '32', cm_oop: '1048576', coam: '4294967296', cocf: '17592186044416', ddm: '65536', dlf: '1024', dlp: '32768', dm_ip: '16', dm_oop: '32768', f9: '1099511627776', fb_ip: '4', fb_oop: '512', gk_ip: '1', gk_oop: '1', hb: '34359738368', hfb: '2048', if_ip: '134217728', ifb: '4503599627370496', iow: '1099511627776', iw_ip: '562949953421312', iwb: '17592186044416', lhk: '4', mp: '144115188075855872', nncb: '536870912', nngk: '9007199254740992', pcm: '134217728', pdm: '131072', pfb: '1024', poa: '524288', pw: '8796093022208', pwb: '36028797018963968', scb: '16', scm: '268435456', sdm: '262144', socf: '35184372088832', swk: '2', tam: '2147483648', tcf: '8796093022208', tf: '262144', tw: '549755813888', twm: '68719476736', w_ip: '128', w_oop: '274877906944', wb_ip: '8', wb_oop: '8', wccm: '536870912', wcdm: '524288', wcm: '281474976710656', wf_ip: '35184372088832', wm_ip: '64', wm_oop: '34359738368', wow: '2199023255552', wowm: '137438953472'},
    // app instruction key -> FM instruction id, number of FM options, app option index -> FM option index (null = no FM equivalent)
    ti: {
      passing: [1, 5, [0, 1, 2, 3, 4]], tempo: [21, 5, [0, 1, 2, 3, 4]], width: [4, 5, [0, 1, 2, 3, 4]], creative: [5, 3, [0, 1, 2]],
      att_trans: [3, 3, [0, 1, 2]], set_pieces: [6, 2, [0, null, 1]], timewaste: [7, 3, [0, 1, 2]], reception: [20, 3, [2, 1, 0]],
      buildup: [24, 3, [0, 1, 2]], goal_kicks: [29, 3, [0, 1, 2]], gk_speed: [17, 3, [0, 1, 2]], gk_target: [16, 7, [1, 2, 0, 3, 5]],
      support_runs: [28, 4, [null, 0, 3]], dribbling: [19, 3, [0, 1, 2]], progress: [30, 5, [2, 0, 1, 3]], patience: [18, 3, [0, 1, 2]],
      long_shots: [27, 3, [0, 1, 2]], crossing: [26, 4, [3, 0, 2, 1]], loe: [8, 3, [0, 1, 2]], dline: [9, 5, [0, 1, 2, 3, 4]],
      trigger: [10, 5, [0, 1, 2, 3, 4]], def_trans: [11, 3, [0, 1, 2]], tackling: [12, 3, [0, 1, 2]], press_trap: [25, 3, [0, 1, 2]],
      cross_eng: [22, 3, [2, 1, 0]], short_gk: [23, 2, [0, 1]], dl_behaviour: [15, 3, [2, 1, 0]],
    },
  },
  // ---- next opponent (verified FM27: TeamReference props; fixture keys)
  team: { pairs: (id) => `825307168=28;1198877028=1;1685218662=${id}`, nextFixture: 1852130921, players: 926359584, name: 1851878757 },
  fixture: { opp: 1717663572, us: 1718119764, date: 1717859429 },
  // ---- recruitment (decoded and verified from the bridge, see bridge/README.md)
  search: { pairs: '825307168=98;1198877028=1;1719300978=1;1954115685=0', text: 1851878757, people: 1886547059, uniqueId: 1970170212 },
  action: { ev: 44, pairs: (id) => `825307168=1;1198877028=1;1633904996=${id}`, shortlistAdd: 1097488485, shortlistRemove: 1500280650 },
  focus: {
    pairs: (c, id) => `825307168=85;1198877028=1;1348620377=${c.manager};1769566810=${c.team};1919314276=${id}`,
    info: (c) => `825307168=97;1198877028=1;1348620377=${c.manager};1769566810=${c.team}`,
    name: 1851878757, ageMin: 1394625906, ageMax: 929909872, ca: 1346584898, pa: 1347436866, level: 1298233430,
    caStep: 1517250124, paStep: 1431517038, area: 1817143629, addArea: 0, areas: 1986161206, confirm: 55,
    list: 1953983593, remove: 53, id: 1919314276,
    nation: (natId) => 167772160 + natId,   // FM NationReference id = fmsave nation_id + 0x0A000000
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
  // FM display strings come as "\u0001<formatting>\u0002<text>\u0001": keep the text
  text(v) { return typeof v === 'string' ? v.split('\u0002').pop().replace(/\u0001/g, '').trim() : ''; },
  /** Next match of the managed team: { team, name, uniqs: [fmsave unique_id of each opponent player] } or null. */
  async nextOpponent() {
    const c = await FMB.context(), T = FMB_IDS.team, X = FMB_IDS.fixture;
    const [fx] = await FMB.read([['TeamReference', T.pairs(c.team), T.nextFixture]]);
    const p = FMB.pair(fx, X.opp), q = FMB.pair(fx, X.us), opp = p && p !== c.team ? p : q && q !== c.team ? q : 0;
    if (!opp) return null;
    const [name, list] = await FMB.read([['TeamReference', T.pairs(opp), T.name], ['TeamReference', T.pairs(opp), T.players]]);
    const persons = FMB.items(list).map(x => FMB.pair(x, FMB_IDS.key.personId)).filter(Boolean);
    const uq = await FMB.read(persons.map(id => ['PersonReference', `825307168=28;1198877028=1;${FMB_IDS.key.personId}=${id}`, FMB_IDS.search.uniqueId]));
    return { team: opp, name: FMB.text(name), uniqs: uq.map(v => v == null ? '' : String(v)).filter(Boolean) };
  },
  // ---- recruitment: FM search, shortlist, recruitment focus (scout assignment)
  /** FM person id for a player of the save: FM's own search by name, confirmed by fmsave's unique_id. null if not found. */
  async findPerson(name, uniq) {
    const S = FMB_IDS.search, sleep = (ms) => new Promise(r => setTimeout(r, ms));
    await FMB.ok(`WRITE FM.UI.SearchReference|${S.pairs}|${S.text} ${JSON.stringify(['System.String', name])}`);
    for (let i = 0; i < 10; i++) {
      await sleep(350);
      const [list] = await FMB.read([['SearchReference', S.pairs, S.people]]);
      const ids = FMB.items(list).map(x => FMB.pair(x, FMB_IDS.key.personId)).filter(Boolean);
      if (!ids.length) continue;
      if (!uniq) return ids.length === 1 ? ids[0] : null;
      const uq = await FMB.read(ids.map(id => ['PersonReference', `825307168=28;1198877028=1;${FMB_IDS.key.personId}=${id}`, S.uniqueId]));
      const k = uq.findIndex(v => String(v) === String(uniq));
      return k >= 0 ? ids[k] : null;
    }
    return null;
  },
  shortlistAdd(c, person) {
    const A = FMB_IDS.action, P = (id) => ({ type: 'SI.Bindable.DynamicReference', entries: [[FMB_IDS.key.personId, 'SI.Core.DynamicNumber', String(id)]] });
    return FMB.send(A.ev, 'ActionReference', A.pairs(A.shortlistAdd), { type: 'SI.Bindable.DynamicReference', entries: [
      [1853189474, 'SI.Core.DynamicNumber', '1'], [2004054900, 'System.Collections.Generic.List`1', [['System.String', '']]],
      [1214805357, 'SI.Bindable.DynamicReference', P(c.manager)], [FMB_IDS.key.personId, 'SI.Bindable.DynamicReference', P(person)],
      [1685218636, 'System.Collections.Generic.List`1', [['SI.Bindable.DynamicReference', P(person)]]],
      [1936223853, 'System.String', ''], [1954115685, 'SI.Core.DynamicNumber', '0'], [1818719336, 'SI.Core.DynamicNumber', '0']] });
  },
  /** Adds the app's shortlist to FM's shortlist. say(text) shows progress. Returns { added: [names], missing: [names] }. */
  async pushShortlist(list, say = () => {}) {
    const c = await FMB.context(), out = { added: [], missing: [] };
    for (const [i, p] of list.entries()) {
      say(`${i + 1}/${list.length} · ${p.name}`, i / list.length);
      const person = await FMB.findPerson(p.name, p.uniq);
      if (!person) { out.missing.push(p.name); continue; }
      await FMB.shortlistAdd(c, person); out.added.push(p.name);
    }
    return out;
  },
  /** FM's recruitment focuses: [{ id, name }]. */
  async focusList(c) {
    const F = FMB_IDS.focus;
    const [list] = await FMB.read([['ScoutingInfoReference', F.info(c), F.list]]);
    const ids = FMB.items(list).map(x => FMB.pair(x, F.id)).filter(x => x != null);
    const names = ids.length ? await FMB.read(ids.map(id => ['RecruitmentFocusItemReference', F.pairs(c, id), F.name])) : [];
    return ids.map((id, k) => ({ id, name: names[k] || '' }));
  },
  /** Creates a recruitment focus (scout assignment) in FM. o: { name, ageMin, ageMax, caStars, paStars, areas: [{ id, label }] }.
   *  Fills FM's "new focus" form in the game (any position) and confirms it. Returns the new focus { id, name }. */
  async createFocus(o, say = () => {}) {
    const c = await FMB.context(), F = FMB_IDS.focus, P = F.pairs(c, -1), sleep = (ms) => new Promise(r => setTimeout(r, ms));
    const W = (prop, v) => FMB.ok(`WRITE FM.UI.RecruitmentFocusItemReference|${P}|${prop} ${JSON.stringify(v)}`);
    const E = (ev, v) => FMB.send(ev, 'RecruitmentFocusItemReference', P, v);
    const num = (n) => ({ type: 'SI.Bindable.DynamicReference', entries: [[FMB_IDS.key.number, 'SI.Core.DynamicNumber', String(n)], [FMB_IDS.key.payload, 'System.String', String(n)]] });
    const [areas] = await FMB.read([['RecruitmentFocusItemReference', P, F.areas]]);
    if (FMB.items(areas).length) throw new Error(L('Há um Recruitment Focus a meio no FM (com áreas escolhidas): termine-o ou cancele-o primeiro.', 'There is a Recruitment Focus half-created in FM (with areas chosen): finish or cancel it first.'));
    const before = new Set((await FMB.focusList(c)).map(f => f.id));
    say(L('Idade e capacidade', 'Age and ability'));
    await W(F.ageMin, num(o.ageMin)); await W(F.ageMax, num(o.ageMax));
    const stepTo = async (prop, ev, stars) => {      // half-star steps: value = 10 + 2 × stars
      const target = Math.round(10 + 2 * stars);
      for (let i = 0; i < 14; i++) {
        const [r] = await FMB.read([['RecruitmentFocusItemReference', P, prop]]);
        const v = FMB.num(FMB.field(r, F.level), target); if (v === target) return;
        await E(ev, ['SI.Core.DynamicNumber', v < target ? '1' : '0']); await sleep(200);
      }
    };
    await stepTo(F.ca, F.caStep, o.caStars); await stepTo(F.pa, F.paStep, o.paStars);
    say(L('Áreas', 'Areas'));
    for (const a of o.areas) {
      await W(F.area, { type: 'SI.Bindable.DynamicReference', entries: [[FMB_IDS.key.payload, 'System.String', a.label],
        [FMB_IDS.key.number, 'SI.Bindable.DynamicReference', { type: 'SI.Bindable.DynamicReference', entries: [[FMB_IDS.key.personId, 'SI.Core.DynamicNumber', String(a.id)]] }]] });
      await E(F.addArea, null); await sleep(250);
    }
    await W(F.name, ['System.String', String(o.name).slice(0, 25)]); await sleep(300);
    say(L('A confirmar', 'Confirming'));
    await E(F.confirm, null); await sleep(900);
    const made = (await FMB.focusList(c)).find(f => !before.has(f.id));
    if (!made) throw new Error(L('O FM não criou o Recruitment Focus.', 'FM did not create the Recruitment Focus.'));
    return made;
  },
  // ---- tactic: formation (in possession), XI, roles, mentality, team instructions into FM's ACTIVE tactic
  /** FM in-possession position bits for an app slot code (DCL = DC + left flag). */
  ipBits(code) {
    const T = FMB_IDS.tactic, m = /^(DC|DM|MC|AMC|ST)([LR])$/.exec(code), base = m ? m[1] : code;
    const map = { GK: 1, DR: 4, DL: 8, DC: 16, WBR: 32, WBL: 64, DM: 128, MR: 256, ML: 512, MC: 1024, AMR: 2048, AML: 4096, AMC: 8192, ST: 16384 };
    const b = map[base]; if (b == null) return null;
    return b + (m ? (m[2] === 'L' ? T.left : T.right) : 0);
  },
  async readTacticSlots(c) {
    const T = FMB_IDS.tactic, P = T.pairs(c);
    const [slots, ips, oops] = await FMB.read([['TeamTacticReference', P, T.slots], ['TeamTacticReference', P, T.ipRoles], ['TeamTacticReference', P, T.oopRoles]]);
    const ipR = {}, oopR = {}, side = 2097152n | 4194304n | 8388608n;   // OOP role field = role bits + flags 2^21..2^23 (2^20 is a real role: Central Midfielder)
    FMB.items(ips).forEach(r => { ipR[FMB.num(FMB.field(r, T.slotIdx))] = String(FMB.field(r, T.ipRole)); });
    FMB.items(oops).forEach(r => { const v = FMB.field(r, T.oopRole); if (v != null) oopR[FMB.num(FMB.field(r, T.slotIdx))] = (BigInt(v) & ~side).toString(); });
    return FMB.items(slots).map(r => { const slot = FMB.num(FMB.field(r, T.slotIdx)); return { slot, ip: FMB.num(FMB.field(r, T.ipPos)), oop: FMB.num(FMB.field(r, T.oopPos)), ipRole: ipR[slot], oopRole: oopR[slot] }; });
  },
  /** FM's tactics (names): the "Copy Tactic" submenu of the tactic menu lists all of them (menu item id 0; read live,
   *  unlike the recruitment screen's list, which stays stale after a delete). */
  async tacticList(c) {
    const T = FMB_IDS.tactic, [menu] = await FMB.read([['TeamTacticReference', T.pairs(c), T.menu]]);
    const copy = FMB.items(menu).find(m => FMB.num(FMB.field(FMB.field(m, 1383036740), FMB_IDS.key.number), -1) === T.menuCopy);
    return FMB.items(FMB.field(copy, T.menuSub)).map(x => String(FMB.field(x, FMB_IDS.key.payload) || ''));
  },
  /** Creates a NEW tactic in FM (the others are not touched) by replaying its "Create New Tactic" wizard, then applies on it.
   *  Sequence from FM27 Assistant (TacticFm.cs): new, style, Next, Next, confirm. No event 90 (replaying it replaced the active tactic).
   *  FM keeps at most 3 tactics: with 3 it refuses. Returns the new tactic's name. */
  async createTactic(say = () => {}) {
    const c = await FMB.context(), T = FMB_IDS.tactic, P = T.pairs(c), W = T.wizard, sleep = (ms) => new Promise(r => setTimeout(r, ms));
    const have = await FMB.tacticList(c);
    if (have.length >= T.maxTactics) throw new Error(L(`O FM j\u00e1 tem ${have.length} t\u00e1ticas (o m\u00e1ximo): apague uma no FM primeiro, ou use \u201cAplicar no FM\u201d, que escreve na t\u00e1tica activa.`, `FM already has ${have.length} tactics (the maximum): delete one in FM first, or use \u201cApply in FM\u201d, which writes into the active tactic.`));
    say(L('A criar uma t\u00e1tica nova', 'Creating a new tactic'));
    const ev = (e) => FMB.send(e, 'TeamTacticReference', P, null);
    await ev(W.open); await sleep(600);
    await FMB.ok(`WRITE FM.UI.TeamTacticReference|${P}|${W.style} ${JSON.stringify(['SI.Core.DynamicNumber', String(W.styleBase)])}`); await sleep(3500);
    await ev(W.next); await sleep(400); await ev(W.next); await sleep(400); await ev(W.finish); await sleep(1500);
    const now = await FMB.tacticList(c);
    if (now.length !== have.length + 1) throw new Error(L('O FM n\u00e3o criou a t\u00e1tica nova.', 'FM did not create the new tactic.'));
    return (await FMB.read([['TeamTacticReference', P, T.name]]))[0] || '';
  },
  /** Applies the app's tactic to FM's active tactic. say(text) shows progress. Returns lines describing what was done. */
  async applyTactic(t, squad, say = () => {}, opts = {}) {
    await FMB.status(true);
    const v = String((FMB.state || {}).version || '0').split('.').map(Number);
    if (FMB.port === 27027 || v[0] * 10000 + (v[1] || 0) * 100 + (v[2] || 0) < 402) throw new Error(L('Aplicar a tática precisa do Tactical Bible Bridge 0.4.2 ou mais recente (Definições › Integrações).', 'Applying the tactic needs Tactical Bible Bridge 0.4.2 or newer (Settings › Integrations).'));
    const c = await FMB.context(), T = FMB_IDS.tactic, P = T.pairs(c), sleep = (ms) => new Promise(r => setTimeout(r, ms)), done = [];
    const N = (k, v) => [k, 'SI.Core.DynamicNumber', String(v)], B = (k, v) => [k, 'System.Boolean', v ? 'True' : 'False'];
    if (opts.create) { const nm = await FMB.createTactic(say); done.push(L(`Tática nova no FM: ${nm}`, `New tactic in FM: ${nm}`)); }
    // 1. in-possession formation: move the FM slots that aren't on a target position to the free target positions
    say(L('Formação', 'Formation'));
    const want = t.slots.map(s => FMB.ipBits(s.ipCode));
    if (want.some(b => b == null)) throw new Error(L('Posição da formação sem equivalente no FM.', 'Formation position with no FM equivalent.'));
    let cur = await FMB.readTacticSlots(c);
    const free = [...want], movers = [];
    cur.forEach(s => { const i = free.indexOf(s.ip); if (i >= 0) free.splice(i, 1); else movers.push(s); });
    for (const [i, s] of movers.entries()) {
      const to = free[i]; if (to == null) break;
      await FMB.send(T.move, 'TeamTacticReference', P, { type: 'SI.Core.Record', entries: [
        [T.moveFrom, 'SI.Core.Record', { type: 'SI.Core.Record', entries: [N(T.moveFrom, s.ip), B(T.moveIP, true), N(T.moveSlot, s.slot)] }],
        [T.moveTo, 'SI.Core.Record', { type: 'SI.Core.Record', entries: [N(T.moveTo, to), B(T.moveIP, true)] }]] });
      await sleep(150);
    }
    done.push(L(`Formação com bola: ${t.ipForm} (${movers.length} posições mudadas)`, `In-possession formation: ${t.ipForm} (${movers.length} positions moved)`));
    // 1b. out-of-possession formation: each FM slot (one player) goes to the OOP position the app pairs with its IP position.
    //     Same move event with the out-of-possession flag; a slot whose target is taken waits, cycles go through a free spot.
    cur = await FMB.readTacticSlots(c);
    const wantOOP = {}, usedS = new Set();
    for (const s of t.slots) { const fm = cur.find(x => x.ip === FMB.ipBits(s.ipCode) && !usedS.has(x.slot)); if (fm) { usedS.add(fm.slot); wantOOP[fm.slot] = FMB.ipBits(s.oopCode); } }
    if (Object.values(wantOOP).some(b => b == null)) throw new Error(L('Posição sem bola sem equivalente no FM.', 'Out-of-possession position with no FM equivalent.'));
    const moveOOP = (slot, from, to) => FMB.send(T.move, 'TeamTacticReference', P, { type: 'SI.Core.Record', entries: [
      [T.moveFrom, 'SI.Core.Record', { type: 'SI.Core.Record', entries: [N(T.moveFrom, from), B(T.moveIP, false), N(T.moveSlot, slot)] }],
      [T.moveTo, 'SI.Core.Record', { type: 'SI.Core.Record', entries: [N(T.moveTo, to), B(T.moveIP, false)] }]] });
    const pos = Object.fromEntries(cur.map(x => [x.slot, x.oop]));
    const spare = [256, 512, 32, 64, 2048, 4096, 8192, 16384, 1024, 128].filter(b => !Object.values(wantOOP).includes(b));
    let movedOOP = 0;
    for (let guard = 0; guard < 40; guard++) {
      const todo = Object.keys(wantOOP).map(Number).filter(s => pos[s] !== wantOOP[s]); if (!todo.length) break;
      const s = todo.find(x => !Object.values(pos).includes(wantOOP[x]));
      if (s != null) { await moveOOP(s, pos[s], wantOOP[s]); pos[s] = wantOOP[s]; movedOOP++; }
      else { const x = todo[0], tmp = spare.find(b => !Object.values(pos).includes(b)); if (tmp == null) break; await moveOOP(x, pos[x], tmp); pos[x] = tmp; }
      await sleep(150);
    }
    const afterOOP = await FMB.readTacticSlots(c), okOOPpos = afterOOP.filter(x => wantOOP[x.slot] === x.oop).length;
    done.push(L(`Formação sem bola: ${t.oopForm} (${movedOOP} mudadas, ${okOOPpos}/11 confirmadas)`, `Out-of-possession formation: ${t.oopForm} (${movedOOP} moved, ${okOOPpos}/11 confirmed)`));
    // 2. XI
    say('XI');
    const xi = await FMB.applyXI(t, squad); done.push(L(`XI: ${xi.placed} trocas`, `XI: ${xi.placed} swaps`) + (xi.skipped.length ? ' — ' + xi.skipped.slice(0, 3).join('; ') : ''));
    // 3. roles, per FM slot holding that position
    say(L('Papéis', 'Roles'));
    cur = await FMB.readTacticSlots(c);
    const used = new Set(), missing = [], fmSlotOf = [];
    const setRole = (slot, bits, name, ip) => FMB.send(T.setRole, 'TeamTacticReference', P, { type: 'SI.Core.Record', entries: [N(T.slotIdx, slot),
      [T.roleRec, 'SI.Bindable.DynamicReference', { type: 'SI.Bindable.DynamicReference', entries: [N(FMB_IDS.key.number, 1), [FMB_IDS.key.payload, 'SI.Bindable.DynamicReference', { type: 'SI.Bindable.DynamicReference', entries: [
        N(T.roleRec, bits), [1464367445, 'SI.Bindable.DynamicReference', { type: 'SI.Bindable.DynamicReference', entries: [N(1298233430, 14), N(1298755158, 14), B(1397715049, false)] }], B(1936487524, false), [T.roleName, 'System.String', name]] }]] }], B(T.roleIP, ip)] });
    for (const s of t.slots) {
      const bits = FMB.ipBits(s.ipCode), fm = cur.find(x => x.ip === bits && !used.has(x.slot)); if (!fm) continue; used.add(fm.slot); fmSlotOf.push([s, fm.slot]);
      for (const [rid, ip] of [[s.ip, true], [s.oop, false]]) {
        const rb = rid && T.roles[rid], r = rid && ROLE_BY_ID[rid];
        if (!rb) { if (rid) missing.push(roleName(r) || rid); continue; }
        await setRole(fm.slot, rb, (r && (r.en || r.name)) || '', ip); await sleep(80);
      }
    }
    const after = await FMB.readTacticSlots(c);
    let okIP = 0, okOOP = 0, n = 0;
    for (const s of t.slots) { const fm = after.find(x => x.ip === FMB.ipBits(s.ipCode)); if (!fm) continue; n++; if (fm.ipRole === T.roles[s.ip]) okIP++; if (fm.oopRole === T.roles[s.oop]) okOOP++; }
    done.push(L(`Papéis: ${okIP}/${n} com bola e ${okOOP}/${n} sem bola confirmados no FM`, `Roles: ${okIP}/${n} in possession and ${okOOP}/${n} out of possession confirmed in FM`) + (missing.length ? L(' · sem equivalente: ', ' · no FM equivalent: ') + [...new Set(missing)].join(', ') : ''));
    // 3b. player instructions: revert every slot to its role defaults, then step each wanted category (they cycle) from Standard
    say(L('Instruções individuais', 'Player instructions'));
    let nPI = 0; const noPI = new Set();
    if (!T.piEnabled) {
      const lines = fmSlotOf.filter(([s]) => (s.pis || []).length).map(([s]) => `${s.ipCode}: ${s.pis.join(', ')}`);
      if (lines.length) done.push(L('Instruções individuais (a fazer à mão no FM): ', 'Player instructions (set these by hand in FM): ') + lines.join(' · '));
    }
    const revert = (slot, ip) => FMB.send(T.piRevert, 'TeamTacticReference', P, { type: 'SI.Core.Record', entries: [N(T.slotIdx, slot), B(T.roleIP, ip),
      [T.piRevertRec, 'SI.Bindable.DynamicReference', { type: 'SI.Bindable.DynamicReference', entries: [N(FMB_IDS.key.number, 0), [FMB_IDS.key.payload, 'System.String', 'Revert to Role']] }]] });
    const piPress = (slot, cat, ip) => FMB.send(T.piPrev, 'TeamTacticReference', P, { type: 'SI.Bindable.DynamicReference', entries: [N(T.slotIdx, slot), N(T.piCat, cat), B(T.roleIP, ip)] });
    for (const [s, slot] of (T.piEnabled ? fmSlotOf : [])) {
      const want = (s.pis || []).map(n => [n, T.pi[n]]);
      if (!want.length) continue;
      await revert(slot, true); await sleep(150); await revert(slot, false); await sleep(250);
      for (const [n, m] of want) {
        if (!m) { noPI.add(n); continue; }
        for (let i = 0; i < m[2]; i++) { await piPress(slot, m[1], m[0]); await sleep(150); }
        nPI++;
      }
    }
    if (T.piEnabled) done.push(L(`Instruções individuais: ${nPI} aplicadas (o FM ignora as que o papel bloqueia)`, `Player instructions: ${nPI} applied (FM ignores the ones a role locks)`) + (noPI.size ? L(' · sem equivalente no FM: ', ' · no FM equivalent: ') + [...noPI].join(', ') : ''));
    // 4. mentality
    const mi = MENTALITIES.indexOf(t.ment);
    if (mi >= 0) { await FMB.ok(`WRITE FM.UI.TeamTacticReference|${P}|${T.mentality} ${JSON.stringify({ type: 'SI.Bindable.DynamicReference', entries: [N(FMB_IDS.key.number, mi + 1), [FMB_IDS.key.payload, 'System.String', t.ment]] })}`); done.push(L('Mentalidade: ', 'Mentality: ') + t.ment); }
    // 5. team instructions: ‹ to the first option, then › to the wanted one
    say(L('Instruções de equipa', 'Team instructions'));
    let nTI = 0; const noTI = [];
    for (const [k, [id, count, map]] of Object.entries(T.ti)) {
      const v = tiVal(t, k), target = v == null ? null : map[v];
      if (target == null) { if (v != null && TI_BY_KEY[k]) noTI.push(TI_BY_KEY[k].n); continue; }
      // FM drops arrow presses that arrive too fast: 150 ms between presses (verified on FM's screen)
      for (let i = 0; i < count; i++) { await FMB.send(T.tiPrev, 'TeamTacticReference', P, ['SI.Core.DynamicNumber', String(id)]); await sleep(150); }
      for (let i = 0; i < target; i++) { await FMB.send(T.tiNext, 'TeamTacticReference', P, ['SI.Core.DynamicNumber', String(id)]); await sleep(150); }
      nTI++;
    }
    done.push(L(`Instruções de equipa: ${nTI} aplicadas`, `Team instructions: ${nTI} applied`) + (noTI.length ? L(' · sem opção igual no FM: ', ' · no matching FM option: ') + noTI.join(', ') : ''));
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
    tacticnew: ['bridge-tac', 'A.bridgeTactic(true)', L('Criar nova no FM', 'Create new in FM'), L('Cria uma tática nova no FM (as outras ficam como estão) e aplica esta nela. O FM só aceita 3 táticas.', 'Creates a new tactic in FM (the others stay as they are) and applies this one to it. FM only holds 3 tactics.')],
    tactic: ['bridge-tac', 'A.bridgeTactic()', L('Aplicar no FM', 'Apply in FM'), L('Escreve esta tática na tática activa do FM: formação com bola, XI, papéis, mentalidade e instruções de equipa.', 'Writes this tactic into FM’s active tactic: in-possession formation, XI, roles, mentality and team instructions.')],
    sp: ['bridge-sp', 'A.bridgeSetPieces()', L('Aplicar no FM', 'Apply in FM'), L('Aplica os marcadores e as rotinas de bolas paradas no FM. No FM, abra Táticas › Bolas paradas.', 'Applies the takers and set-piece routines in FM. In FM, open Tactics › Set Pieces.')],
    shortlist: ['bridge-sl', 'A.fmShortlist()', L('Lista → FM', 'Shortlist → FM'), L('Junta os jogadores da tua lista à shortlist do FM.', 'Adds the players on your list to FM\'s shortlist.')],
    focus: ['bridge-focus', 'A.focusOpen()', L('Observação no FM', 'Scout in FM'), L('Cria um Recruitment Focus (missão de observação) no FM a partir desta pesquisa.', 'Creates a Recruitment Focus (scouting assignment) in FM from this search.')],
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
