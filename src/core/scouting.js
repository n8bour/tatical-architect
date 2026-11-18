// ============================================================
// Prospeção (scouting): todos os jogadores do save, lidos pelo fmsave
// (`fmsave export SAVE players --all`). Parser rápido por blocos, métricas
// derivadas (potencial provável, idade, contrato) e pesquisas pré-definidas.
// ============================================================

// colunas pedidas ao fmsave (nomes planos: <grupo>_<campo>); a ordem não importa
const SCOUT_ATTR_FM = ['crossing','dribbling','finishing','heading','long_shots','marking','off_the_ball','passing','penalty_taking','tackling','vision','handling','aerial_reach',
  'command_of_area','communication','kicking','throwing','anticipation','decisions','one_on_ones','positioning','reflexes','first_touch','technique','flair','corners','teamwork',
  'work_rate','long_throws','eccentricity','rushing_out','punching','acceleration','free_kick_taking','strength','stamina','pace','jumping_reach','leadership','dirtiness','balance',
  'bravery','consistency','aggression','agility','important_matches','injury_proneness','versatility','natural_fitness','determination','composure','concentration'];
const SCOUT_POS_FM = ['gk','dl','dc','dr','dm','ml','mc','mr','aml','amc','amr','stc','wbl','wbr'];
const SCOUT_PERS = ['ambition','professionalism','pressure','loyalty','temperament','adaptability','sportsmanship','controversy'];
const SCOUT_COLUMNS = ['uid','unique_id','name','age','nation_id','height_cm','ability_current','ability_potential','ability_potential_range_code','reputation_current','reputation_world',
  'club_uid','club_name','club_nation_id','club_reputation','on_loan','loan_parent_club_name','transfer_value','contract_wage','contract_end','contract_squad_status','left_foot','right_foot',
  ...SCOUT_POS_FM.map(x => 'positions_' + x), ...SCOUT_PERS.map(x => 'personality_' + x), ...SCOUT_ATTR_FM.map(x => 'attributes_' + x)];
const SCOUT_HIDDEN = { consistency: 'cons', important_matches: 'imp', injury_proneness: 'inj', versatility: 'vers', dirtiness: 'dirt' };
// grupos de posição da app a partir das 14 notas do fmsave
const SCOUT_POS_GROUP = { gk: ['GK'], dl: ['D_LR', 'D_LR:L'], dc: ['DC'], dr: ['D_LR', 'D_LR:R'], dm: ['DM'], ml: ['M_LR', 'M_LR:L'], mc: ['MC'], mr: ['M_LR', 'M_LR:R'],
  aml: ['AM_LR', 'AM_LR:L'], amc: ['AMC'], amr: ['AM_LR', 'AM_LR:R'], stc: ['ST'], wbl: ['WB_LR', 'WB_LR:L'], wbr: ['WB_LR', 'WB_LR:R'] };
// potencial guardado como intervalo (código negativo do FM) → [mín, máx]
const PA_RANGE = { '-1': [1, 20], '-2': [10, 40], '-25': [20, 50], '-3': [30, 60], '-35': [40, 70], '-4': [50, 80], '-45': [60, 90], '-5': [70, 100], '-55': [80, 110], '-6': [90, 120],
  '-65': [100, 130], '-7': [110, 140], '-75': [120, 150], '-8': [130, 160], '-85': [140, 170], '-9': [150, 180], '-95': [160, 190], '-10': [170, 200] };

// ---------- parser (por blocos, para não bloquear a janela) ----------
async function parseScoutDB(text, opts = {}) {
  const t0 = Date.now();
  const src = String(text || '').replace(/\r/g, '');
  let nl = src.indexOf('\n'); if (nl < 0) nl = src.length;
  const head = splitCSV(src.slice(0, nl), ',');
  const ix = {}; head.forEach((h, i) => { ix[h.trim()] = i; });
  const col = (name) => ix[name] ?? -1;
  const need = ['name', 'ability_current'];
  if (need.some(n => col(n) < 0)) throw new Error(L('Este ficheiro não parece uma exportação de jogadores do fmsave (faltam as colunas name/ability_current).', 'This file does not look like an fmsave players export (the name/ability_current columns are missing).'));
  // mapa das colunas de atributos para as chaves da app
  const attrCols = []; const hidCols = [];
  head.forEach((h, i) => {
    const m = h.match(/^attributes_(.+)$/); if (!m) return;
    if (SCOUT_HIDDEN[m[1]]) { hidCols.push([i, SCOUT_HIDDEN[m[1]]]); return; }
    const k = headerToKey(h); if (k && ATTRS[k]) attrCols.push([i, k]);
  });
  const posCols = SCOUT_POS_FM.map(p => [col('positions_' + p), p]).filter(x => x[0] >= 0);
  const persCols = SCOUT_PERS.map(p => [col('personality_' + p), p]).filter(x => x[0] >= 0);
  const C = Object.fromEntries(['uid', 'unique_id', 'name', 'age', 'nation_id', 'height_cm', 'ability_current', 'ability_potential', 'ability_potential_range_code', 'reputation_current', 'reputation_world',
    'club_uid', 'club_name', 'club_nation_id', 'club_reputation', 'on_loan', 'loan_parent_club_name', 'transfer_value', 'contract_wage', 'contract_end', 'contract_squad_status', 'left_foot', 'right_foot'].map(n => [n, col(n)]));
  const num = (v) => { if (v == null || v === '') return null; const n = +v; return isFinite(n) ? n : null; };
  const players = [];
  let pos = nl + 1, rows = 0;
  const total = src.length;
  while (pos < total) {
    const end = Math.min(total, pos + 2_000_000);           // ~2 MB por bloco
    let cut = end >= total ? total : src.indexOf('\n', end); if (cut < 0) cut = total;
    const chunk = src.slice(pos, cut).split('\n');
    for (const line of chunk) {
      if (!line) continue;
      const r = line.indexOf('"') >= 0 ? splitCSV(line, ',') : line.split(',');
      const g = (c) => (c >= 0 ? r[c] : undefined);
      const name = g(C.name); if (!name) continue;
      const a = {}; for (const [i, k] of attrCols) { const v = num(r[i]); if (v != null) a[k] = v; }
      const hid = {}; for (const [i, k] of hidCols) { const v = num(r[i]); if (v != null) hid[k] = v; }
      const pers = {}; for (const [i, k] of persCols) { const v = num(r[i]); if (v != null) pers[k] = v; }
      const posR = {}; const posSet = new Set(), natSet = new Set();
      for (const [i, k] of posCols) { const v = num(r[i]); if (v == null) continue; posR[k] = v; if (v >= 15) SCOUT_POS_GROUP[k].forEach(x => posSet.add(x)); if (v >= 18) natSet.add(SCOUT_POS_GROUP[k][0]); }
      const pa = num(g(C.ability_potential)), code = num(g(C.ability_potential_range_code)), rng = code != null ? PA_RANGE[String(code)] : null;
      const lf = num(g(C.left_foot)), rf = num(g(C.right_foot));
      const endStr = g(C.contract_end) || '';
      const p = {
        id: 'db' + (g(C.uid) || rows), uid: g(C.uid) || null, name, age: num(g(C.age)), nat: num(g(C.nation_id)), height: num(g(C.height_cm)),
        ca: num(g(C.ability_current)), pa: pa != null ? pa : rng ? Math.round((rng[0] + rng[1]) / 2) : null, paRange: pa == null && rng ? rng : null,
        rep: num(g(C.reputation_current)), wrep: num(g(C.reputation_world)), club: g(C.club_name) || '', clubUid: g(C.club_uid) || '', clubNat: num(g(C.club_nation_id)), clubRep: num(g(C.club_reputation)),
        loan: /^true$/i.test(g(C.on_loan) || ''), loanFrom: g(C.loan_parent_club_name) || '', value: num(g(C.transfer_value)), wage: num(g(C.contract_wage)),
        end: /^\d{4}-\d{2}-\d{2}/.test(endStr) ? endStr.slice(0, 10) : '', status: g(C.contract_squad_status) || '',
        foot: lf != null && rf != null ? (lf >= 15 && rf >= 15 ? 'B' : lf > rf ? 'L' : 'R') : null, lf, rf,
        posR, pos: [...posSet], natPos: [...natSet], a, hid, pers, src: 'db',
      };
      p.posRaw = Object.entries(posR).filter(([, v]) => v >= 15).sort((x, y) => y[1] - x[1]).map(([k]) => k.toUpperCase()).join(', ');
      p.meta = { foot: p.foot === 'L' ? 'Left' : p.foot === 'R' ? 'Right' : p.foot === 'B' ? 'Either' : '', height: p.height };
      players.push(p); rows++;
    }
    pos = cut + 1;
    if (opts.onProgress) opts.onProgress(Math.min(1, pos / total), rows);
    await new Promise(res => setTimeout(res, 0));
  }
  return { players, ms: Date.now() - t0, columns: head.length, attrCols: attrCols.length };
}

function parseNations(text) {
  const rows = tableFromText(text || ''); if (rows.length < 2) return {};
  const h = rows[0].map(x => x.trim().toLowerCase());
  const id = ['uid', 'nation_id', 'id'].map(k => h.indexOf(k)).find(i => i >= 0), nm = ['name', 'nation_name', 'short_name'].map(k => h.indexOf(k)).find(i => i >= 0);
  if (id == null || nm == null) return {};
  return Object.fromEntries(rows.slice(1).filter(r => r[id] !== '' && r[nm]).map(r => [+r[id], r[nm]]));
}
// data do jogo (fmsave info --json): o primeiro campo com uma data ISO cujo nome fale de data
function gameDateFromInfo(json) {
  try {
    const o = typeof json === 'string' ? JSON.parse(json) : json; let found = null;
    const walk = (x, key) => { if (found) return; if (typeof x === 'string' && /^\d{4}-\d{2}-\d{2}/.test(x) && /date|day|clock|current/i.test(key || '')) found = x.slice(0, 10); else if (x && typeof x === 'object') for (const [k, v] of Object.entries(x)) walk(v, k); };
    walk(o, ''); return found;
  } catch (e) { return null; }
}
// sem data do jogo: o fim de época mais próximo (a data de fim de contrato mais comum entre as próximas)
function inferGameDate(players) {
  const ends = players.map(p => p.end).filter(Boolean).sort();
  if (!ends.length) return null;
  const years = {}; ends.forEach(e => { const y = e.slice(0, 4); years[y] = (years[y] || 0) + 1; });
  const ys = Object.keys(years).sort(); const first = ys.find(y => years[y] >= ends.length * 0.04) || ys[0];
  return `${first}-01-15`;    // aproximação: janela de Janeiro da época que termina nesse ano
}

// ---------- métricas derivadas ----------
// probabilidade (0–100) de um jovem atingir o potencial: personalidade, determinação, idade, lesões e contexto
function reachLikelihood(p) {
  if (p.pa == null || p.ca == null) return null;
  const P = (k, d = 10) => (p.pers && p.pers[k] != null ? p.pers[k] : d), H = (k, d = 10) => (p.hid && p.hid[k] != null ? p.hid[k] : d), A = (k, d = 10) => (p.a && p.a[k] != null ? p.a[k] : d);
  const age = p.age ?? 20, ageF = age <= 17 ? 1 : age <= 18 ? 0.92 : age <= 19 ? 0.82 : age <= 20 ? 0.7 : age <= 21 ? 0.56 : age <= 23 ? 0.38 : 0.2;
  const gap = Math.max(0, p.pa - p.ca), gapF = gap === 0 ? 1 : Math.max(0, 1 - gap / 120);
  const env = p.clubRep != null ? Math.min(1, 0.45 + p.clubRep / 10000 * 0.7) : 0.7;
  const s = P('professionalism') / 20 * 0.30 + A('det') / 20 * 0.18 + P('ambition') / 20 * 0.10 + (21 - H('inj')) / 20 * 0.10 + H('cons') / 20 * 0.04 + ageF * 0.16 + gapF * 0.06 + env * 0.06;
  return Math.round(Math.min(100, s * 100));
}
// Probabilidade de querer vir para o meu clube — ESTIMATIVA. O fmsave não lê o "interessado em vir" do jogo,
// por isso usa-se o que pesa nessa decisão no FM: reputação do meu clube vs a do clube dele, reputação dele,
// situação contratual (livre, a terminar, estatuto no plantel), ambição e lealdade.
const FRINGE_STATUS = /backup|not needed|surplus|rotation|youngster|emergency|reserve|prospect/i;
function joinLikelihood(p, ctx) {
  const myRep = ctx && ctx.myRep; if (myRep == null) return null;
  let s = 50; const why = [];
  const clamp = (v, lo, hi) => Math.max(lo, Math.min(hi, v));
  if (!p.club) { s += 18; why.push(L('sem clube', 'no club')); }
  else if (p.clubRep != null) {
    const d = (myRep - p.clubRep) / 1000; s += clamp(d * 9, -30, 22);
    why.push(d >= 0.5 ? L('o teu clube é maior que o dele', 'your club is bigger than his') : d <= -0.5 ? L('o clube dele é maior que o teu', 'his club is bigger than yours') : L('clubes de dimensão parecida', 'clubs of similar size'));
  }
  if (p.rep != null && p.rep > myRep) { s -= Math.min(30, (p.rep - myRep) / 1000 * 10); why.push(L('a reputação dele está acima da do teu clube', 'his reputation is above your club’s')); }
  const m = monthsUntil(p.end, ctx.date);
  if (m != null && m <= 6) { s += 10; why.push(L('contrato a acabar em 6 meses', 'contract ending within 6 months')); }
  else if (m != null && m <= 12) { s += 6; why.push(L('contrato a acabar em 12 meses', 'contract ending within 12 months')); }
  if (p.status && FRINGE_STATUS.test(p.status)) { s += 10; why.push(L('não é titular no clube dele', 'not a regular at his club')); }
  const up = !p.club || p.clubRep == null || myRep >= p.clubRep;
  const amb = p.pers && p.pers.ambition, loy = p.pers && p.pers.loyalty;
  if (amb != null) { s += (up ? 1 : -1) * (amb - 10) * 1.2; if (amb >= 15) why.push(up ? L('ambicioso: quer subir', 'ambitious: wants to step up') : L('ambicioso: não quer descer', 'ambitious: does not want a step down')); }
  if (loy != null && p.club) { s -= (loy - 10) * 1.0; if (loy >= 15) why.push(L('muito leal ao clube actual', 'very loyal to his current club')); }
  return { score: Math.round(clamp(s, 0, 100)), why };
}
function monthsUntil(dateISO, fromISO) { if (!dateISO || !fromISO) return null; const a = new Date(fromISO), b = new Date(dateISO); return (b - a) / (1000 * 60 * 60 * 24 * 30.44); }

// ---------- filtros ----------
const SCOUT_DEFAULT = () => ({ q: '', ageMin: '', ageMax: '', caMin: '', caMax: '', paMin: '', paMax: '', pos: '', posLevel: 'acc', foot: '', nation: '', club: '', league: '',
  valueMax: '', wageMax: '', endMonths: '', free: false, excludeMine: true, loan: 'any', profMin: '', ambMin: '', detMin: '', consMin: '', impMin: '', injMax: '', heightMin: '',
  attrs: [], role: '', roleMin: '', likelyMin: '', clubRepMax: '', joinMin: '', sort: 'ca', dir: 'desc' });

function scoutFilter(db, F, ctx = {}) {
  const n = (v) => (v === '' || v == null ? null : +v);
  const ageMin = n(F.ageMin), ageMax = n(F.ageMax), caMin = n(F.caMin), caMax = n(F.caMax), paMin = n(F.paMin), paMax = n(F.paMax), valueMax = n(F.valueMax), wageMax = n(F.wageMax), endM = n(F.endMonths);
  const clubRepMax = n(F.clubRepMax), profMin = n(F.profMin), ambMin = n(F.ambMin), detMin = n(F.detMin), consMin = n(F.consMin), impMin = n(F.impMin), injMax = n(F.injMax), heightMin = n(F.heightMin), likelyMin = n(F.likelyMin);
  const q = (F.q || '').trim().toLowerCase(), club = (F.club || '').trim().toLowerCase();
  const mine = ctx.myClub ? ctx.myClub.toLowerCase() : null;
  const attrs = (F.attrs || []).filter(x => x.k && x.min !== '' && x.min != null);
  const out = [];
  for (const p of db.players) {
    if (q && !p.name.toLowerCase().includes(q)) continue;
    if (ageMin != null && !(p.age >= ageMin)) continue; if (ageMax != null && !(p.age <= ageMax)) continue;
    if (caMin != null && !(p.ca >= caMin)) continue; if (caMax != null && !(p.ca <= caMax)) continue;
    if (paMin != null && !(p.pa >= paMin)) continue; if (paMax != null && !(p.pa <= paMax)) continue;
    if (F.pos) { const list = F.posLevel === 'nat' ? p.natPos : p.pos; if (!list.includes(F.pos)) continue; }
    if (F.foot === 'L' && !(p.lf >= 15)) continue; if (F.foot === 'R' && !(p.rf >= 15)) continue; if (F.foot === 'B' && p.foot !== 'B') continue;
    if (F.nation !== '' && F.nation != null && p.nat !== +F.nation) continue;
    if (F.league !== '' && F.league != null && p.clubNat !== +F.league) continue;
    if (club && !p.club.toLowerCase().includes(club)) continue;
    if (F.free && p.club) continue;
    if (clubRepMax != null && p.clubRep != null && p.clubRep > clubRepMax) continue;
    if (F.excludeMine && mine && p.club.toLowerCase() === mine) continue;
    if (F.loan === 'exclude' && p.loan) continue; if (F.loan === 'only' && !p.loan) continue;
    // valor/salário desconhecido (o fmsave só os preenche quando os lê com segurança): com um máximo definido,
    // fica de fora — excepto os jogadores livres, que não têm custo de transferência
    if (valueMax != null) { if (p.value == null) { if (p.club) continue; } else if (p.value > valueMax) continue; }
    if (wageMax != null) { if (p.wage == null) { if (p.club) continue; } else if (p.wage > wageMax) continue; }
    if (endM != null) { const m = monthsUntil(p.end, ctx.date); if (m == null || m > endM || m < -1) continue; }
    if (profMin != null && !((p.pers.professionalism ?? 0) >= profMin)) continue;
    if (ambMin != null && !((p.pers.ambition ?? 0) >= ambMin)) continue;
    if (detMin != null && !((p.a.det ?? 0) >= detMin)) continue;
    if (consMin != null && !((p.hid.cons ?? 0) >= consMin)) continue;
    if (impMin != null && !((p.hid.imp ?? 0) >= impMin)) continue;
    if (injMax != null && !((p.hid.inj ?? 20) <= injMax)) continue;
    if (heightMin != null && !((p.height ?? 0) >= heightMin)) continue;
    let okA = true; for (const x of attrs) if (!((p.a[x.k] ?? 0) >= +x.min)) { okA = false; break; } if (!okA) continue;
    out.push(p);
  }
  // métricas caras só no resultado já filtrado
  if (likelyMin != null || F.sort === 'likely') out.forEach(p => { p._lk = reachLikelihood(p); });
  let res = likelyMin != null ? out.filter(p => (p._lk ?? 0) >= likelyMin) : out;
  const joinMin = n(F.joinMin);
  if ((joinMin != null || F.sort === 'join') && ctx.myRep != null) { res.forEach(p => { const j = joinLikelihood(p, ctx); p._jn = j ? j.score : null; }); if (joinMin != null) res = res.filter(p => (p._jn ?? 0) >= joinMin); }
  if (F.role) {
    const r = ROLE_BY_ID[F.role], rm = n(F.roleMin);
    const pool = res.length > 30000 ? res.sort((x, y) => (y.ca ?? 0) - (x.ca ?? 0)).slice(0, 30000) : res;
    pool.forEach(p => { p._rs = (roleScore(p, r) || { score: 0 }).score; });
    res = pool.filter(p => rm == null || p._rs >= rm);
  }
  if (F.sort === 'gain' && ctx.tactic && ctx.squad && ctx.squad.length) {
    const pool = res.sort((x, y) => (y.ca ?? 0) - (x.ca ?? 0)).slice(0, 3000);
    pool.forEach(p => { const g = bestGain(p, ctx.tactic, ctx.squad); p._gain = g ? g.gain : -99; p._gainSlot = g ? g.code : ''; });
    res = pool;
  }
  const key = { ca: 'ca', pa: 'pa', age: 'age', value: 'value', wage: 'wage', likely: '_lk', role: '_rs', gain: '_gain', join: '_jn', name: 'name', end: 'end' }[F.sort] || 'ca';
  const dir = F.dir === 'asc' ? 1 : -1;
  res.sort((x, y) => { const a = x[key], b = y[key]; if (a == null && b == null) return 0; if (a == null) return 1; if (b == null) return -1; return (a > b ? 1 : a < b ? -1 : 0) * dir; });
  return res;
}

// onde entraria no meu XI e quanto melhora (adequação na casa vs o titular actual)
function bestGain(p, t, squad) {
  if (!t || !t.slots) return null;
  const pm = Object.fromEntries((squad || []).map(x => [x.id, x]));
  let best = null;
  for (const s of t.slots) {
    const a = slotInfo(t.ipForm, s.ipCode); if (!a) continue;
    if (posFactor(p, a.group, a.side) < 0.85) continue;
    const mine = pm[s.pid], cur = mine ? slotScore(mine, t, s) : 0, his = slotScore(p, t, s);
    const g = his - cur;
    if (!best || g > best.gain) best = { code: s.ipCode, gain: g, his, cur, starter: mine ? mine.name : '' };
  }
  return best;
}
// melhores papéis (todas as posições onde é pelo menos competente)
function bestRolesAny(p, n = 5) {
  const groups = [...new Set(p.pos.filter(x => !x.includes(':')))];
  const seen = new Set(), out = [];
  for (const g of groups) for (const r of rolesFor(g, 'IP', g === 'DC')) { if (seen.has(r.id)) continue; seen.add(r.id); out.push({ r, g, s: (roleScore(p, r) || { score: 0 }).score }); }
  return out.sort((x, y) => y.s - x.s).slice(0, n);
}

// ---------- pesquisas pré-definidas ----------
function scoutPresets(ctx) {
  const xiCA = ctx.xiCA || 120, d = SCOUT_DEFAULT();
  return [
    { k: 'cheap_wonder', ic: '💎', n: L('Wonderkids baratos', 'Cheap wonderkids'), d: L('Até 19 anos, potencial ≥ 150 e valor baixo.', 'Up to 19, potential ≥ 150 and a low value.'),
      F: { ...d, ageMax: 19, paMin: 150, valueMax: ctx.cheapValue || 3000000, sort: 'pa' } },
    { k: 'reach', ic: '📈', n: L('Wonderkids que vão lá chegar', 'Wonderkids likely to reach potential'), d: L('Até 20 anos, potencial ≥ 145 e alta probabilidade de o atingir (profissionalismo, determinação, ambição, lesões).', 'Up to 20, potential ≥ 145 and a high chance of reaching it (professionalism, determination, ambition, injuries).'),
      F: { ...d, ageMax: 20, paMin: 145, likelyMin: 70, sort: 'likely' } },
    { k: 'expiring', ic: '⏳', n: L('Contrato a terminar', 'Contract expiring'), d: L(`Contrato acaba nos próximos 6 meses e CA perto do teu XI (≥ ${Math.max(60, xiCA - 15)}).`, `Contract ends within 6 months and CA close to your XI (≥ ${Math.max(60, xiCA - 15)}).`),
      F: { ...d, endMonths: 6, caMin: Math.max(60, xiCA - 15), ageMax: 32, sort: 'ca' } },
    { k: 'interested', ic: '🤝', n: L('Interessados em vir (estimativa)', 'Likely to join (estimate)'), d: L('Alta probabilidade de aceitar vir para o teu clube, ao nível do teu XI. Estimativa: o fmsave não lê o “interessado” do jogo.', 'High chance of agreeing to join your club, at your XI’s level. An estimate: fmsave does not read the game’s “interested” flag.'),
      F: { ...d, joinMin: 70, caMin: Math.max(60, xiCA - 10), sort: 'join' }, needMyClub: true },
    { k: 'free', ic: '🆓', n: L('Jogadores livres', 'Free agents'), d: L('Sem clube: sem custo de transferência.', 'Without a club: no transfer fee.'), F: { ...d, free: true, caMin: Math.max(50, xiCA - 30), sort: 'ca' } },
    { k: 'bargain', ic: '🏷', n: L('Pechinchas', 'Bargains'), d: L(`Entre 21 e 29 anos, CA ao nível do teu XI e valor baixo.`, `Between 21 and 29, CA at your XI’s level and a low value.`),
      F: { ...d, ageMin: 21, ageMax: 29, caMin: Math.max(60, xiCA - 5), valueMax: ctx.bargainValue || 5000000, sort: 'ca' } },
    { k: 'upgrade', ic: '⬆', n: L('Reforços para a minha tática', 'Upgrades for my tactic'), d: L('Quem mais melhora o teu XI, posição a posição (adequação ao papel da tua tática).', 'Who improves your XI the most, position by position (fit for your tactic’s role).'),
      F: { ...d, caMin: Math.max(60, xiCA - 10), sort: 'gain' }, needSquad: true },
    { k: 'veteran', ic: '🎖', n: L('Veteranos a custo zero', 'Veterans on a free'), d: L('30 anos ou mais, qualidade alta e contrato a acabar em 12 meses (ou sem clube).', '30 or older, high quality and a contract ending within 12 months (or no club).'),
      F: { ...d, ageMin: 30, caMin: Math.max(60, xiCA), endMonths: 12, sort: 'ca' } },
    { k: 'hidden', ic: '🔍', n: L('Joias escondidas', 'Hidden gems'), d: L('Até 21 anos, potencial ≥ 140 em clubes pequenos (reputação baixa).', 'Up to 21, potential ≥ 140 at small clubs (low reputation).'),
      F: { ...d, ageMax: 21, paMin: 140, valueMax: ctx.cheapValue || 3000000, clubRepMax: 4000, sort: 'pa' } },
    { k: 'elite_pers', ic: '🧠', n: L('Personalidade de elite', 'Elite personality'), d: L('Até 23 anos, Profissionalismo ≥ 17 e Determinação ≥ 15: bons para o balneário e para evoluir.', 'Up to 23, Professionalism ≥ 17 and Determination ≥ 15: good for the dressing room and for development.'),
      F: { ...d, ageMax: 23, profMin: 17, detMin: 15, paMin: 120, sort: 'pa' } },
    { k: 'weak_spot', ic: '🎯', n: L('A minha posição mais fraca', 'My weakest position'), d: L('Jogadores naturais na posição onde o teu XI é mais fraco.', 'Natural players for the position where your XI is weakest.'),
      F: { ...d, caMin: Math.max(60, xiCA - 10), sort: 'gain' }, needSquad: true, weakSpot: true },
  ];
}
