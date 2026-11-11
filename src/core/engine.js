// ============================================================
// Motor de análise — puro JS, sem dependências
// ============================================================

// ---------- Algoritmo húngaro (minimização, n linhas ≤ m colunas) ----------
function hungarian(cost) {
  const n = cost.length; if (!n) return [];
  const m = cost[0].length;
  const INF = 1e18, u = new Array(n + 1).fill(0), v = new Array(m + 1).fill(0);
  const p = new Array(m + 1).fill(0), way = new Array(m + 1).fill(0);
  for (let i = 1; i <= n; i++) {
    p[0] = i; let j0 = 0;
    const minv = new Array(m + 1).fill(INF), used = new Array(m + 1).fill(false);
    do {
      used[j0] = true; const i0 = p[j0]; let delta = INF, j1 = 0;
      for (let j = 1; j <= m; j++) if (!used[j]) {
        const cur = cost[i0 - 1][j - 1] - u[i0] - v[j];
        if (cur < minv[j]) { minv[j] = cur; way[j] = j0; }
        if (minv[j] < delta) { delta = minv[j]; j1 = j; }
      }
      for (let j = 0; j <= m; j++) { if (used[j]) { u[p[j]] += delta; v[j] -= delta; } else minv[j] -= delta; }
      j0 = j1;
    } while (p[j0] !== 0);
    do { const j1 = way[j0]; p[j0] = p[j1]; j0 = j1; } while (j0);
  }
  const res = new Array(n).fill(-1);
  for (let j = 1; j <= m; j++) if (p[j]) res[p[j] - 1] = j - 1;
  return res;
}

// ---------- Papéis válidos ----------
function rolesFor(group, phase, wideCB) {
  return ROLES.filter(r => r.phase === phase && r.groups.includes(group) && (r.wideCB ? wideCB : true));
}
const DEFAULT_ROLE = {
  IP:{ GK:'gk_ip', DC:'cb_ip', D_LR:'fb_ip', WB_LR:'wb_ip', DM:'dm_ip', MC:'cm_ip', M_LR:'wm_ip', AMC:'am_ip', AM_LR:'w_ip', ST:'cf_ip' },
  OOP:{ GK:'gk_oop', DC:'cb_oop', D_LR:'fb_oop', WB_LR:'wb_oop', DM:'dm_oop', MC:'cm_oop', M_LR:'wm_oop', AMC:'am_oop', AM_LR:'w_oop', ST:'cf_oop' },
};
function validRole(id, group, phase, wideCB) {
  const r = ROLE_BY_ID[id];
  return !!r && r.phase === phase && r.groups.includes(group) && (!r.wideCB || wideCB);
}

// ---------- Tática ----------
function slotInfo(formation, code) {
  const s = (FORMATIONS[formation] || []).find(z => z.code === code);
  return s ? { ...s, group: groupOf(code), side: sideOf(s.x), wideCB: isWideCB(code, formation) } : null;
}

// Emparelha cada posição IP com a posição OOP mais próxima (custo total mínimo)
function matchFormations(ipForm, oopForm) {
  const A = FORMATIONS[ipForm], B = FORMATIONS[oopForm];
  const cost = A.map(a => B.map(b => {
    if ((a.code === 'GK') !== (b.code === 'GK')) return 1e6;
    return Math.hypot(a.x - b.x, (a.y - b.y) * 1.1);
  }));
  const asg = hungarian(cost);
  return A.map((a, i) => ({ ipCode: a.code, oopCode: B[asg[i]].code }));
}

function newTactic(ipForm = '4-3-3 DM Wide', oopForm = '4-1-4-1 DM Wide') {
  const pairs = matchFormations(ipForm, oopForm);
  return {
    name: 'Nova tática', ment: 'Balanced', ipForm, oopForm,
    slots: pairs.map(p => {
      const a = slotInfo(ipForm, p.ipCode), b = slotInfo(oopForm, p.oopCode);
      return { ipCode: p.ipCode, oopCode: p.oopCode, ip: DEFAULT_ROLE.IP[a.group], oop: DEFAULT_ROLE.OOP[b.group], pid: null, pis: [] };
    }),
    ti: {}, ois: 0, source: 'manual',
  };
}

// Mudar formação mantendo papéis/jogadores quando possível
function retargetTactic(t, ipForm, oopForm) {
  const nt = newTactic(ipForm, oopForm);
  nt.name = t.name; nt.ment = t.ment; nt.ti = { ...t.ti }; nt.ois = t.ois; nt.source = t.source;
  const oldIP = FORMATIONS[t.ipForm];
  const newIP = FORMATIONS[ipForm];
  // emparelhar slots antigos → novos por proximidade da posição IP
  const cost = newIP.map(a => oldIP.map(b => ((a.code === 'GK') !== (b.code === 'GK')) ? 1e6 : Math.hypot(a.x - b.x, a.y - b.y)));
  const asg = hungarian(cost);
  nt.slots.forEach((s, i) => {
    const old = t.slots.find(o => o.ipCode === oldIP[asg[i]].code); if (!old) return;
    const a = slotInfo(ipForm, s.ipCode), b = slotInfo(oopForm, s.oopCode);
    if (validRole(old.ip, a.group, 'IP', a.wideCB)) s.ip = old.ip;
    if (validRole(old.oop, b.group, 'OOP', b.wideCB)) s.oop = old.oop;
    s.pid = old.pid; s.pis = old.pis || [];
  });
  return nt;
}

function tacticFromTemplate(tp) {
  const t = newTactic(tp.ip, tp.oop);
  t.name = tp.n; t.ment = tp.ment; t.ti = { ...tp.ti }; t.source = 'template';
  t.slots.forEach(s => {
    const r = tp.roles[s.ipCode]; if (!r) return;
    const a = slotInfo(tp.ip, s.ipCode), b = slotInfo(tp.oop, s.oopCode);
    if (validRole(r[0], a.group, 'IP', a.wideCB)) s.ip = r[0];
    if (validRole(r[1], b.group, 'OOP', b.wideCB)) s.oop = r[1];
  });
  return t;
}

// ---------- Plantel ----------
const POS_ALIASES = ['position','positions','posição','posicao','posições','pos.'];
const NAME_ALIASES = ['name','nome','player','jogador'];
const AGE_ALIASES = ['age','idade'];
// Colunas extra do FM View Exporter (FM27): guardadas para mostrar, não entram nos cálculos
const META_ALIASES = { 'best pos':'bestPos', 'best position':'bestPos', 'best role':'bestRole', 'ability':'ability', 'potential':'potential',
  'ability current':'ca', 'current ability':'ca', 'ca':'ca', 'ability potential':'pa', 'potential ability':'pa', 'pa':'pa',
  'unique id':'uid', 'uid':'uid', 'personality':'personality', 'club':'club', 'club name':'club', 'preferred foot':'foot', 'height':'height' };

function normHeader(h) {
  return String(h || '').trim().toLowerCase().replace(/[_\-]+/g, ' ').replace(/\s+/g, ' ').replace(/[.:]$/, '')
    .replace(/^(attributes?|technical|mental|physical|goalkeeping|goalkeeper|gk attributes) /, '');
}
const POS_RATING = { gk:['GK'], dl:['D_LR','D_LR:L'], dc:['DC'], dr:['D_LR','D_LR:R'], wbl:['WB_LR','WB_LR:L'], wbr:['WB_LR','WB_LR:R'], dm:['DM'], dmc:['DM'],
  ml:['M_LR','M_LR:L'], mc:['MC'], mr:['M_LR','M_LR:R'], aml:['AM_LR','AM_LR:L'], amc:['AMC'], amr:['AM_LR','AM_LR:R'], st:['ST'], sc:['ST'] };

function headerToKey(h) {
  const n = normHeader(h);
  if (!n) return null;
  if (NAME_ALIASES.includes(n)) return 'name';
  if (POS_ALIASES.includes(n)) return 'position';
  if (AGE_ALIASES.includes(n)) return 'age';
  if (META_ALIASES[n]) return 'meta:' + META_ALIASES[n];
  if (/ code$/.test(n)) return null;                                      // colunas *_code do fmsave (há uma coluna de texto ao lado)
  const pr = n.match(/^(?:positions?|position rating|proficienc(?:y|ies)|pos) (gk|dl|dc|dr|wbl|wbr|dm|dmc|ml|mc|mr|aml|amc|amr|st|sc)$/);
  if (pr) return 'posr:' + pr[1];
  for (const [k, a] of Object.entries(ATTRS)) if (a.a.includes(n) || normHeader(a.n) === n) return k;
  return null;
}

function parseAttrValue(v) {
  const s = String(v ?? '').trim();
  if (!s || s === '-' || s === '–') return null;
  const m = s.match(/^(\d{1,2})\s*[-–]\s*(\d{1,2})$/);
  if (m) return (+m[1] + +m[2]) / 2;
  const n = parseFloat(s.replace(',', '.'));
  return isFinite(n) ? n : null;
}

// "D (RL), WB (R)", "AM (RLC), ST (C)", "DM, M (C)", "M/AM (R)", "GK"
function parsePositions(str) {
  const out = new Set();
  const s = String(str || '').toUpperCase().replace(/\s+/g, ' ');
  // dividir em partes no nível de topo (vírgulas fora de parênteses)
  const parts = []; let depth = 0, cur = '';
  for (const ch of s) { if (ch === '(') depth++; if (ch === ')') depth--; if (ch === ',' && depth === 0) { parts.push(cur); cur = ''; } else cur += ch; }
  if (cur.trim()) parts.push(cur);
  for (const part of parts) {
    const code = part.trim();
    const SHORT = { GK:['GK'], DC:['DC'], DCL:['DC'], DCR:['DC'], DL:['D_LR','D_LR:L'], DR:['D_LR','D_LR:R'], WBL:['WB_LR','WB_LR:L'], WBR:['WB_LR','WB_LR:R'],
      DM:['DM'], MC:['MC'], ML:['M_LR','M_LR:L'], MR:['M_LR','M_LR:R'], AMC:['AMC'], AML:['AM_LR','AM_LR:L'], AMR:['AM_LR','AM_LR:R'], ST:['ST'], SC:['ST'] };
    if (SHORT[code]) { SHORT[code].forEach(x => out.add(x)); continue; }
    const m = code.match(/^([A-Z/ ]+?)\s*(?:\(([RLC]+)\))?$/); if (!m) continue;
    const pref = m[1].split('/').map(x => x.trim()).filter(Boolean);
    const sides = (m[2] || '').split('');
    for (const p of pref) {
      if (p === 'GK') out.add('GK');
      else if (p === 'DM') out.add('DM');
      else if (p === 'ST' || p === 'S') out.add('ST');
      else if (p === 'D') { if (!sides.length || sides.includes('C')) out.add('DC'); sides.filter(x => x !== 'C').forEach(x => { out.add('D_LR'); out.add('D_LR:' + x); }); }
      else if (p === 'WB') { (sides.length ? sides : ['L','R']).filter(x => x !== 'C').forEach(x => { out.add('WB_LR'); out.add('WB_LR:' + x); }); }
      else if (p === 'M') { if (!sides.length || sides.includes('C')) out.add('MC'); sides.filter(x => x !== 'C').forEach(x => { out.add('M_LR'); out.add('M_LR:' + x); }); }
      else if (p === 'AM') { if (!sides.length || sides.includes('C')) out.add('AMC'); sides.filter(x => x !== 'C').forEach(x => { out.add('AM_LR'); out.add('AM_LR:' + x); }); }
    }
  }
  return [...out];
}

function tableFromText(text) {
  const t = String(text || '').replace(/\r/g, '').trim();
  if (!t) return [];
  if (/<table/i.test(t)) {
    const doc = new DOMParser().parseFromString(t, 'text/html');
    const tables = [...doc.querySelectorAll('table')];
    const table = tables.sort((a, b) => b.rows.length - a.rows.length)[0];
    return [...table.rows].map(r => [...r.cells].map(c => c.textContent.trim()));
  }
  const lines = t.split('\n').filter(l => l.trim() && !/^[\s|\-+=_]+$/.test(l));
  if (lines.some(l => (l.match(/\|/g) || []).length >= 3)) {
    return lines.map(l => l.split('|').map(x => x.trim())).map(r => { if (r[0] === '') r.shift(); if (r[r.length - 1] === '') r.pop(); return r; });
  }
  const head = lines[0];
  const delim = ['\t', ';', ','].sort((a, b) => head.split(b).length - head.split(a).length)[0];
  return lines.map(l => splitCSV(l, delim));
}
function splitCSV(line, d) {
  const out = []; let cur = '', q = false;
  for (let i = 0; i < line.length; i++) {
    const c = line[i];
    if (c === '"') { if (q && line[i + 1] === '"') { cur += '"'; i++; } else q = !q; }
    else if (c === d && !q) { out.push(cur.trim()); cur = ''; }
    else cur += c;
  }
  out.push(cur.trim()); return out;
}

function parseSquad(text) {
  const rows = tableFromText(text);
  if (rows.length < 2) return { players: [], warnings: ['Não encontrei uma tabela com cabeçalho e linhas.'], mapped: [] };
  // encontrar a linha de cabeçalho: a que tem mais colunas reconhecidas
  let hi = 0, best = -1;
  rows.slice(0, 6).forEach((r, i) => { const c = r.map(headerToKey).filter(Boolean).length; if (c > best) { best = c; hi = i; } });
  const header = rows[hi].map(headerToKey);
  const mapped = rows[hi].map((h, i) => ({ h, k: header[i] }));
  const warnings = [];
  if (!header.includes('name')) warnings.push('Sem coluna de nome reconhecida ("Name"/"Nome"). Uso a primeira coluna.');
  const attrCols = header.filter(k => k && ATTRS[k]).length;
  if (attrCols < 10) warnings.push(`Só reconheci ${attrCols} atributos. Para análise completa exporte uma vista com todos os atributos.`);
  const players = [];
  rows.slice(hi + 1).forEach((r, idx) => {
    const p = { id: 'p' + idx + '_' + Math.random().toString(36).slice(2, 7), name: '', pos: [], posRaw: '', age: null, a: {}, meta: {} };
    r.forEach((v, i) => {
      const k = header[i];
      if (k && k.startsWith('meta:')) { if (v) p.meta[k.slice(5)] = v; }
      else if (k && k.startsWith('posr:')) { const n = parseAttrValue(v); if (n != null) (p.posR = p.posR || {})[k.slice(5)] = n; }
      else if (k === 'name') p.name = v;
      else if (k === 'position') { p.posRaw = v; p.pos = parsePositions(v); }
      else if (k === 'age') p.age = parseAttrValue(v);
      else if (k && ATTRS[k]) { const n = parseAttrValue(v); if (n != null) p.a[k] = n; }
    });
    if (!p.name) p.name = r[0];
    if (!p.pos.length && p.posR) {
      // proficiências 1–20 por posição (fmsave): ≥ 15 conta como posição natural; se nenhuma, a melhor
      const ent = Object.entries(p.posR); const top = Math.max(...ent.map(e => e[1]));
      const good = ent.filter(([, r]) => r >= 15 || (top < 15 && r === top)).map(([k]) => k);
      p.pos = [...new Set(good.flatMap(k => POS_RATING[k] || []))]; p.posRaw = good.map(k => k.toUpperCase()).join(', ');
    }
    if (!p.pos.length && p.meta.bestPos) { p.posRaw = p.meta.bestPos; p.pos = parsePositions(p.meta.bestPos); }
    if (p.meta.uid) p.id = 'u' + p.meta.uid;
    if (p.name && Object.keys(p.a).length >= 3) players.push(p);
  });
  const seen = {};
  players.forEach(p => { if (!p.meta.uid) { const base = 'n:' + p.name.trim().toLowerCase(); seen[base] = (seen[base] || 0) + 1; p.id = seen[base] > 1 ? base + '#' + seen[base] : base; } });
  if (!players.length) warnings.push('Nenhum jogador com atributos foi lido.');
  return { players, warnings, mapped };
}

// ---------- Adequação ----------
function roleScore(p, role) {
  if (!p || !role) return null;
  let s = 0, w = 0, have = 0, tot = 0;
  for (const k of role.key) { tot++; const v = p.a[k]; if (v != null) { s += v * 2; w += 2; have++; } }
  for (const k of role.pref) { tot++; const v = p.a[k]; if (v != null) { s += v; w += 1; have++; } }
  if (!w) return null;
  const avg = s / w;
  return { score: Math.round(Math.max(0, Math.min(100, (avg - 1) / 19 * 100))), coverage: have / tot };
}
const ADJ = { DM:['MC','DC'], MC:['DM','AMC'], AMC:['MC','ST'], ST:['AMC'], M_LR:['AM_LR','WB_LR'], AM_LR:['M_LR','ST'], D_LR:['WB_LR','DC'], WB_LR:['D_LR','M_LR'], DC:['DM','D_LR'], GK:[] };
function posFactor(p, group, side) {
  if (!p.pos || !p.pos.length) return 1;
  const isGK = p.pos.includes('GK');
  if (group === 'GK') return isGK ? 1 : 0.1;
  if (isGK && p.pos.length === 1) return 0.1;
  if (p.pos.includes(group)) {
    if ((side === 'L' || side === 'R') && ['D_LR','WB_LR','M_LR','AM_LR'].includes(group)) return p.pos.includes(group + ':' + side) ? 1 : 0.9;
    return 1;
  }
  if ((ADJ[group] || []).some(g => p.pos.includes(g))) return 0.82;
  return 0.6;
}
function slotScore(p, t, slot) {
  const a = slotInfo(t.ipForm, slot.ipCode), b = slotInfo(t.oopForm, slot.oopCode);
  const ri = roleScore(p, ROLE_BY_ID[slot.ip]), ro = roleScore(p, ROLE_BY_ID[slot.oop]);
  if (!ri && !ro) return 0;
  const base = ri && ro ? ri.score * 0.55 + ro.score * 0.45 : (ri || ro).score;
  return Math.round(base * Math.min(posFactor(p, a.group, a.side), posFactor(p, b.group, b.side)));
}
function autoPickXI(t, players) {
  if (!players.length) return t;
  const cost = t.slots.map(s => players.map(p => 100 - slotScore(p, t, s)));
  // garantir colunas ≥ linhas
  while (cost[0].length < cost.length) cost.forEach(r => r.push(1000));
  const asg = hungarian(cost);
  t.slots.forEach((s, i) => { s.pid = players[asg[i]] ? players[asg[i]].id : null; });
  return t;
}
function bestRolesForPlayer(p, group, phase, wideCB, n = 3) {
  return rolesFor(group, phase, wideCB).map(r => ({ r, s: (roleScore(p, r) || { score: 0 }).score })).sort((a, b) => b.s - a.s).slice(0, n);
}

// ---------- Auditoria ----------
function tiVal(t, k) { const d = TI_BY_KEY[k]; return t.ti[k] != null ? t.ti[k] : d.def; }
function tiLabel(k, v) { return TI_BY_KEY[k].opts[v]; }
function avgAttr(players, keys) {
  const vals = []; players.forEach(p => keys.forEach(k => { if (p && p.a[k] != null) vals.push(p.a[k]); }));
  return vals.length ? vals.reduce((a, b) => a + b, 0) / vals.length : null;
}

function complexityOf(t) {
  let cx = 0; const parts = [];
  TEAM_INSTRUCTIONS.forEach(d => { const v = tiVal(t, d.k); if (v !== d.def) { const c = Math.abs(v - d.def) * d.cx; cx += c; parts.push([d.n + ': ' + d.opts[v], c]); } });
  t.slots.forEach(s => {
    const r = ROLE_BY_ID[s.ip];
    if (r && ['fr','ocb','acb','f9'].includes(r.id)) { cx += 1; parts.push(['Papel exigente: ' + r.name, 1]); }
    if (s.pis && s.pis.length) { cx += s.pis.length * 0.5; parts.push([`${s.pis.length} instrução(ões) individuais (${s.ipCode})`, s.pis.length * 0.5]); }
  });
  if (t.ois) { cx += t.ois * 0.5; parts.push([`${t.ois} instruções de adversário`, t.ois * 0.5]); }
  const level = cx <= 6 ? 'Baixa' : cx <= 12 ? 'Média' : 'Alta';
  return { cx: Math.round(cx * 10) / 10, level, parts };
}

function auditTactic(t, squad) {
  const F = []; // findings
  const add = (cat, sev, title, text, fix) => F.push({ cat, sev, title, text, fix });
  const pmap = Object.fromEntries((squad || []).map(p => [p.id, p]));
  const S = t.slots.map(s => {
    const a = slotInfo(t.ipForm, s.ipCode), b = slotInfo(t.oopForm, s.oopCode);
    return { s, a, b, ip: ROLE_BY_ID[s.ip], oop: ROLE_BY_ID[s.oop], p: pmap[s.pid] || null };
  });
  const ipTag = (tag) => S.filter(x => x.ip && x.ip.tags.includes(tag));
  const oopTag = (tag) => S.filter(x => x.oop && x.oop.tags.includes(tag));
  const mi = MENTALITIES.indexOf(t.ment);
  const outfield = S.filter(x => x.a.group !== 'GK');

  // 1. papéis em falta / inválidos
  S.forEach(x => {
    if (!x.ip || !validRole(x.s.ip, x.a.group, 'IP', x.a.wideCB)) add('IP','crit',`Papel IP inválido em ${x.s.ipCode}`,'O papel não existe para esta posição.','Escolha um papel IP válido.');
    if (!x.oop || !validRole(x.s.oop, x.b.group, 'OOP', x.b.wideCB)) add('OOP','crit',`Papel OOP inválido em ${x.s.oopCode}`,'O papel não existe para esta posição.','Escolha um papel OOP válido.');
  });

  // 2. largura por flanco
  const wideSide = (x) => x.a.side !== 'C' ? x.a.side : (x.a.x < 50 ? 'L' : 'R');
  for (const side of ['L', 'R']) {
    const nm = side === 'L' ? 'esquerdo' : 'direito';
    const wide = ipTag('W').filter(x => wideSide(x) === side);
    if (!wide.length) add('IP','warn',`Sem largura no flanco ${nm}`,
      'Nenhum papel com bola fica aberto deste lado. O adversário fecha o meio sem custo e os ataques ficam previsíveis.',
      'Use um Winger/Wide Forward, um Wing-Back/Advanced Wing-Back, ou um Wide Central Midfielder deste lado.');
    const flankers = outfield.filter(x => x.a.side === side && ['D_LR','WB_LR','M_LR','AM_LR'].includes(x.a.group) || (x.a.wideCB && x.a.side === side));
    const fb = flankers.find(x => ['D_LR','WB_LR'].includes(x.a.group));
    const att = flankers.find(x => ['M_LR','AM_LR'].includes(x.a.group));
    if (fb && att && fb.ip && att.ip) {
      const fbIn = fb.ip.tags.includes('IN'), attIn = att.ip.tags.includes('IN');
      const fbW = fb.ip.tags.includes('W'), attW = att.ip.tags.includes('W');
      if (attIn && fbW) add('IP','good',`Flanco ${nm} complementar`,`${att.ip.name} por dentro + ${fb.ip.name} por fora: largura e ameaça interior sem conflito.`);
      else if (fbIn && attW) add('IP','good',`Flanco ${nm} complementar`,`${fb.ip.name} fecha por dentro, ${att.ip.name} dá largura — boa rest-defence neste lado.`);
      else if (fbW && attW && fb.ip.tags.includes('ADV') && att.ip.tags.includes('RUN')) add('IP','info',`Corredor ${nm} pode ficar congestionado`,
        `${fb.ip.name} e ${att.ip.name} querem ambos o corredor.`,'Ponha Supporting Runs em Overlap e/ou troque o extremo por Inside Forward/Inside Winger.');
    }
  }

  // 3. rest-defence
  const rd = outfield.filter(x => x.ip && (x.ip.tags.includes('RD') || x.ip.tags.includes('TOCB') || ((x.s.pis || []).includes('Hold position') && !x.ip.tags.includes('ADV')))).length;
  if (rd < 3) add('IP', mi >= 4 ? 'crit' : 'warn', `Rest-defence frágil (${rd} jogador${rd === 1 ? '' : 'es'})`,
    'Poucos jogadores ficam atrás da linha da bola. Cada perda pode virar contra-ataque com superioridade adversária.',
    'Garanta 3–4: dois centrais + DM/Half Back seguros, ou um lateral como Inside Full-Back/Inside Wing-Back.');
  else if (rd >= 6) add('IP','info',`Muitos jogadores atrás (${rd})`,'Estrutura muito conservadora com bola; pode faltar gente na frente.','Liberte um lateral (Wing-Back) ou um médio (Box-To-Box).');
  else add('IP','good',`Rest-defence equilibrada (${rd})`,'Número adequado de jogadores a proteger a transição.');

  // 4. ocupação da área
  const box = ipTag('BOX').length;
  if (box === 0) add('IP','crit','Ninguém ataca a área','Nenhum papel com ameaça de golo na área.','Avançado Centre Forward/Poacher, extremo Inside Forward, médio Attacking Midfielder/Box-To-Box.');
  else if (box === 1) add('IP','warn','Pouca presença na área (1)','Um só jogador a atacar a área é fácil de neutralizar.','Adicione um Inside Forward, Attacking Midfielder ou Box-To-Box.');
  else add('IP','good',`Presença na área (${box})`,'Vários jogadores a atacar a área.');

  // 5. profundidade
  const st = S.filter(x => x.a.group === 'ST');
  const runners = ipTag('RUN').filter(x => x.a.group !== 'ST' || !x.ip.tags.includes('DROP'));
  if (st.length && st.every(x => x.ip && x.ip.tags.includes('DROP'))) {
    const others = ipTag('RUN').filter(x => x.a.group !== 'ST').length + ipTag('BOX').filter(x => x.a.group !== 'ST').length;
    if (others < 2) add('IP','warn','Avançado(s) baixam e ninguém ataca o espaço',
      'False Nine/Deep-Lying Forward arrasta centrais mas o espaço criado fica vazio.',
      'Use Inside Forward, Channel Midfielder ou Second Striker para atacar a profundidade.');
  }
  if (!runners.length) add('IP','warn','Sem ataque à profundidade','Nenhum papel ataca as costas da defesa — fácil para o adversário subir a linha.','Channel Forward, Poacher, Inside Forward, Wide Forward ou Advanced Wing-Back.');
  if (st.length === 2 && st[0].ip && st[1].ip && st[0].ip.id === st[1].ip.id) add('IP','info','Dupla de avançados igual',
    `Dois ${st[0].ip.name}: tendem a ocupar o mesmo espaço.`,'Combine perfis: um que baixa (DLF/TF) e um que ataca o espaço (Channel Forward/Poacher).');

  // 6. criatividade
  const cr = ipTag('CR').filter(x => x.a.group !== 'GK' && x.a.group !== 'DC');
  if (!cr.length) add('IP','warn','Sem criador designado','Nenhum papel expressivo/organizador — a progressão depende só das instruções.','Deep-Lying Playmaker, Midfield Playmaker, Advanced Playmaker ou Playmaking Winger.');
  else if (cr.length >= 4) add('IP','warn',`Demasiados criadores (${cr.length})`,`${cr.map(x => x.ip.ab).join(', ')} pedem todos a bola.`,'Mantenha 2–3 e troque os restantes por papéis de movimento (corredores/finalizadores).');

  // 7. laterais
  const fbs = S.filter(x => ['D_LR','WB_LR'].includes(x.a.group));
  if (fbs.length === 2 && fbs.every(x => x.ip && x.ip.tags.includes('ADV')) && !outfield.some(x => x.ip && (x.ip.tags.includes('TOCB') || (x.a.group === 'DM' && x.ip.tags.includes('RD')))) && S.filter(x => x.a.group === 'DC').length < 3)
    add('IP','warn','Os dois laterais sobem com linha de 2','Com dois alas ofensivos e só 2 centrais, os flancos ficam expostos em transição.','Half Back no DM, um Inside Full-Back, ou um papel OOP Wide Covering no médio.');
  const acb = outfield.filter(x => x.ip && x.ip.id === 'acb');
  if (acb.length && outfield.filter(x => x.a.group === 'DC' && x.ip && x.ip.id !== 'acb').length < 2 && !outfield.some(x => x.ip && x.ip.tags.includes('TOCB')))
    add('IP','warn','Advanced Centre-Back sem cobertura','O ACB sobe para o meio-campo e fica só um central atrás.','Junte um Half Back ou Inside Full-Back, ou use linha de 3.');

  // 8. OOP: outlets e pressão
  const outl = oopTag('OUT').length;
  if (outl >= 3) add('OOP','crit',`Outlets a mais (${outl})`,'Três ou mais jogadores não defendem: o bloco fica em inferioridade.','Máximo 1–2 outlets; troque por Tracking/padrão.');
  const loe = tiVal(t,'loe') * 2, dl = tiVal(t,'dline'), trig = tiVal(t,'trigger'), dtr = tiVal(t,'def_trans'), atr = tiVal(t,'att_trans');
  const hp = oopTag('HP').length;
  if (loe >= 3 && hp <= 1) add('OOP','warn','Pressão alta sem pressionadores','Linha de pressão alta mas quase nenhum papel OOP assertivo.','Pressing CM/DM/FB, Tracking Centre Forward, Stopping CB.');
  if (loe <= 1 && hp >= 4) add('OOP','warn','Papéis de pressão num bloco baixo','Vários papéis assertivos com linha de pressão baixa: desorganiza o bloco.','Use Screening/Covering ou suba a Line of Engagement.');
  if (loe - dl >= 2) add('OOP','warn','Distância entre linhas (pressão alta, defesa baixa)',`LoE ${tiLabel('loe',loe/2)} com linha ${tiLabel('dline',dl)}: espaço enorme entre médios e defesa.`,'Aproxime as linhas (suba a defesa ou baixe a pressão).');
  if (dl - loe >= 2) add('OOP','warn','Linha alta sem pressão sobre a bola',`Linha ${tiLabel('dline',dl)} com LoE ${tiLabel('loe',loe/2)}: o portador tem tempo para lançar nas costas.`,'Suba a Line of Engagement ou baixe a linha defensiva.');
  const gk = S.find(x => x.a.group === 'GK');
  if (dl >= 3 && gk && gk.oop && gk.oop.id !== 'swk') add('OOP','warn','Linha alta sem Sweeper Keeper','Bolas nas costas da defesa ficam sem cobertura.','Mude o papel OOP do GR para Sweeper Keeper.');
  if (dl <= 1 && gk && gk.oop && gk.oop.id === 'swk') add('OOP','info','Sweeper Keeper com linha baixa','Pouco espaço para varrer; o GR sai sem necessidade.','Considere Goalkeeper ou Line-Holding Keeper.');
  if (atr === 2 && outl === 0 && !ipTag('RUN').length) add('Coerência','warn','Counter-Attack sem alvos','Transição de contra-ataque mas sem outlets nem corredores.','Adicione 1 Outlet OOP ou papéis de profundidade.');
  if (dtr === 2 && mi <= 1) add('Coerência','info','Counter-Press com mentalidade defensiva','Instrução e mentalidade puxam em sentidos opostos.','Escolha: Regroup para bloco baixo, ou suba a mentalidade.');
  if (dtr === 2 && tiVal(t,'width') >= 3) add('Coerência','info','Counter-Press com equipa muito larga','Jogadores longe uns dos outros na perda chegam tarde à pressão.','Balanced/Narrower na largura ou Regroup.');
  const trap = tiVal(t,'press_trap');
  if (trap === 2 && !S.some(x => x.oop && ['pfb','prwb','tw','twm','swcb'].includes(x.oop.id))) add('OOP','info','Pressing Trap Outside sem quem feche o corredor','Força para fora mas ninguém está instruído a apertar na ala.','Pressing Full-Back/Wing-Back ou Tracking Winger.');

  // 9. construção
  const bu = tiVal(t,'buildup');
  if (bu === 0) {
    if (gk && gk.ip && gk.ip.id === 'nngk') add('Coerência','warn','Play Through Press com No-Nonsense GK','O GR vai bater longo contra a intenção da equipa.','Ball-Playing Goalkeeper ou Goalkeeper.');
    if (S.some(x => x.ip && x.ip.id === 'nncb')) add('Coerência','warn','Play Through Press com No-Nonsense CB','Um central que alivia sempre quebra a construção curta.','Centre-Back ou Ball-Playing Centre-Back.');
  }
  if (bu === 2 && S.some(x => x.ip && ['bpgk','dlp'].includes(x.ip.id))) add('Coerência','info','Bypass Press desperdiça construtores','BPGK/DLP têm pouco impacto se a bola salta o meio-campo.','Balanced ou troque o papel.');
  if (tiVal(t,'goal_kicks') === 0 && gk && gk.ip && gk.ip.id === 'nngk') add('Coerência','warn','Goal Kicks curtos com No-Nonsense GK','Contradição directa.','Mixed/Long ou mude o papel.');
  const cs = tiVal(t,'crossing');
  if ((cs === 3) && !ipTag('AIR').length) add('Coerência','info','Cruzamentos altos sem referência aérea','Floated crosses sem Target Forward.','Low Crosses, ou um Target Forward/Centre Forward forte no ar.');
  if (tiVal(t,'reception') === 2 && tiVal(t,'tempo') >= 4) add('Coerência','info','Passe ao pé com ritmo muito alto','Exige Primeiro Toque e Decisões muito altos.','Baixe o tempo para Higher ou Balanced.');
  if (mi >= 5 && dl <= 1 && loe <= 1) add('Coerência','info','Mentalidade atacante com bloco baixo','Ataca com muitos mas defende muito atrás: equipa partida.','Suba linhas ou baixe a mentalidade.');
  if (mi <= 1 && loe >= 3) add('Coerência','info','Mentalidade defensiva com pressão alta','Mensagem contraditória.','Ajuste uma das duas.');

  // 10. distância IP↔OOP
  S.forEach(x => { if (x.a && x.b) { const d = Math.hypot(x.a.x - x.b.x, x.a.y - x.b.y); if (d > 38) add('Coerência','warn',`${x.s.ipCode} → ${x.s.oopCode}: transição longa`,`O jogador percorre uma grande distância entre a posição com e sem bola (${Math.round(d)}).`,'Escolha uma formação OOP mais próxima ou troque posições.'); } });

  // 11. complexidade
  const C = complexityOf(t);
  const xiPlayers = S.map(x => x.p).filter(Boolean);
  let cap = 12;
  if (xiPlayers.length >= 8) {
    const iq = avgAttr(xiPlayers, ['dec','ant','tea','pos','otb','cnt']);
    cap = iq < 10 ? 6 : iq < 12 ? 9 : iq < 14 ? 12 : 16;
    if (C.cx > cap) add('Complexidade','warn',`Tática complexa para este plantel (${C.cx} > ${cap})`,`Média tática do XI (Decisões/Antecipação/Trabalho de Equipa/…) ≈ ${iq.toFixed(1)}. No FM27 isto alonga a adaptação.`,'Remova instruções que não resolvem um problema concreto.');
    else add('Complexidade','good',`Complexidade adequada (${C.cx} ≤ ${cap})`,'O plantel deve conseguir absorver a tática.');
  } else if (C.cx > 12) add('Complexidade','warn',`Complexidade alta (${C.cx})`,'Muitas instruções fora do default: mais tempo de treino e jogadores mais capazes.','Comece simples; adicione instruções quando resolverem um problema identificado.');
  else add('Complexidade','good',`Complexidade ${C.level.toLowerCase()} (${C.cx})`,'Fácil de assimilar.');

  // 12. plantel
  if (xiPlayers.length) {
    S.forEach(x => {
      if (!x.p) return;
      const sc = slotScore(x.p, t, x.s);
      const ri = roleScore(x.p, x.ip), ro = roleScore(x.p, x.oop);
      if (sc < 50) {
        const alt = bestRolesForPlayer(x.p, x.a.group, 'IP', x.a.wideCB, 1)[0];
        add('Plantel','warn',`${x.p.name} pouco adequado em ${x.s.ipCode} (${sc})`,
          `${x.ip.name}: ${ri ? ri.score : '—'} · ${x.oop.name}: ${ro ? ro.score : '—'}.` + (Math.min(posFactor(x.p, x.a.group, x.a.side), posFactor(x.p, x.b.group, x.b.side)) < 1 ? ' Penalizado por jogar fora da posição natural (com ou sem bola).' : ''),
          alt && alt.r.id !== x.ip.id ? `Melhor papel IP para ele aqui: ${alt.r.name} (${alt.s}).` : 'Considere outro jogador.');
      }
    });
    const dcs = S.filter(x => x.a.group === 'DC' && x.p).map(x => x.p);
    const sp = avgAttr(dcs, ['pac','acc']);
    if (dl >= 3 && sp != null && sp < 12) add('Plantel','crit','Linha alta com centrais lentos',`Velocidade/Aceleração média dos centrais ≈ ${sp.toFixed(1)}.`,'Baixe a linha para Standard/Lower ou mude os centrais.');
    if (trig >= 3 || loe >= 3 || dtr === 2) {
      const ws = avgAttr(xiPlayers.filter((p, i) => true), ['wor','sta']);
      if (ws != null && ws < 12) add('Plantel','warn','Pressão exigente para o físico do XI',`Índice de Trabalho/Resistência médio ≈ ${ws.toFixed(1)}.`,'Reduza Trigger Press / LoE ou use Press Less nos Tactical Tweaks.');
    }
    if (bu === 0 && gk && gk.p) { const g = avgAttr([gk.p], ['pas','fir','cmp','kic']); if (g != null && g < 11) add('Plantel','warn','GR fraco com os pés para construção curta',`Passe/Primeiro Toque/Compostura/Pontapé ≈ ${g.toFixed(1)}.`,'Balanced/Bypass Press ou outro GR.'); }
    if (tiVal(t,'cross_eng') === 0) { const h = avgAttr(dcs, ['hea','jum','str']); if (h != null && h < 13) add('Plantel','warn','Convida cruzamentos sem centrais fortes no ar',`Cabeceamento/Impulsão/Força dos centrais ≈ ${h.toFixed(1)}.`,'Balanced ou Stop Crosses.'); }
    if (atr === 2) { const fast = xiPlayers.filter(p => (p.a.pac || 0) >= 14 || (p.a.acc || 0) >= 14).length; if (fast < 2) add('Plantel','warn','Contra-ataque sem velocidade',`Só ${fast} jogador(es) com Velocidade/Aceleração ≥ 14.`,'Hold Shape/Balanced, ou jogadores rápidos nos outlets.'); }
    if (cs === 3) { const air = xiPlayers.filter(p => (p.a.hea || 0) >= 14 && (p.a.jum || 0) >= 13).length; if (air < 2) add('Plantel','info','Cruzamentos altos com poucos jogadores aéreos',`${air} jogador(es) com Cabeceamento ≥ 14 e Impulsão ≥ 13.`,'Low Crosses.'); }
  }

  // pontuação por categoria
  const cats = ['IP','OOP','Coerência','Complexidade'].concat(xiPlayers.length ? ['Plantel'] : []);
  const pen = { crit: 25, warn: 10, info: 3, good: 0 };
  const scores = {};
  cats.forEach(c => { scores[c] = Math.max(0, 100 - F.filter(f => f.cat === c).reduce((a, f) => a + pen[f.sev], 0)); });
  const overall = Math.round(cats.reduce((a, c) => a + scores[c], 0) / cats.length);
  const order = { crit: 0, warn: 1, info: 2, good: 3 };
  F.sort((a, b) => order[a.sev] - order[b.sev]);
  return { findings: F, scores, overall, complexity: C, cap };
}

// ---------- Sugestões a partir do plantel ----------
function styleFit(players, style) {
  const top = [...players].sort((a, b) => avgAttr([b], Object.keys(ATTRS)) - avgAttr([a], Object.keys(ATTRS))).slice(0, 16);
  const v = avgAttr(top, STYLE_PROFILES[style].a);
  return v == null ? 50 : Math.round(Math.max(0, Math.min(100, (v - 1) / 19 * 100)));
}
function suggestTactics(players) {
  if (players.length < 11) return [];
  return TEMPLATES.map(tp => {
    const t = autoPickXI(tacticFromTemplate(tp), players);
    const pmap = Object.fromEntries(players.map(p => [p.id, p]));
    const xi = t.slots.map(s => ({ s, p: pmap[s.pid], sc: pmap[s.pid] ? slotScore(pmap[s.pid], t, s) : 0 }));
    const xiAvg = Math.round(xi.reduce((a, x) => a + x.sc, 0) / xi.length);
    // profundidade: melhor suplente por posição
    const used = new Set(t.slots.map(s => s.pid));
    const bench = players.filter(p => !used.has(p.id));
    const depth = Math.round(xi.reduce((a, x) => a + Math.max(0, ...bench.map(p => slotScore(p, t, x.s))), 0) / xi.length);
    const fit = styleFit(players, tp.style);
    const audit = auditTactic(t, players);
    const total = Math.round(xiAvg * 0.55 + fit * 0.2 + depth * 0.15 + audit.overall * 0.1);
    const weakest = [...xi].sort((a, b) => a.sc - b.sc).slice(0, 2);
    const strongest = [...xi].sort((a, b) => b.sc - a.sc).slice(0, 3);
    return { tp, t, xiAvg, depth, fit, audit, total, weakest, strongest };
  }).sort((a, b) => b.total - a.total);
}

// ---------- Plano de treino ----------
function trainingPlan(t, players, audit) {
  const areas = [];
  const push = (k, why, w) => { const e = areas.find(a => a.k === k); if (e) { e.w += w; e.why.push(why); } else areas.push({ k, w, why: [why] }); };
  push('ip_shape','Base de qualquer tática (formação IP).',2); push('oop_shape','Base de qualquer tática (formação OOP).',2);
  const v = (k) => tiVal(t, k), d = (k) => TI_BY_KEY[k].def;
  if (v('passing') !== d('passing') || v('reception') !== d('reception') || v('tempo') !== d('tempo')) push('passing','Estilo de passe fora do default.',2);
  if (v('buildup') !== d('buildup') || v('goal_kicks') !== d('goal_kicks') || v('gk_target') !== d('gk_target')) push('buildup','Instruções de construção definidas.',2);
  if (v('patience') !== d('patience') || v('crossing') !== d('crossing') || v('long_shots') !== d('long_shots') || v('dribbling') !== d('dribbling')) push('chance','Instruções de último terço definidas.',2);
  if (v('trigger') !== d('trigger')) push('trigger','Trigger Press fora do default.',2 + Math.abs(v('trigger') - d('trigger')));
  if (v('press_trap') !== d('press_trap')) push('trap','Pressing Trap definida.',2);
  if (v('loe') !== d('loe')) push('loe','Line of Engagement fora do default.',1 + Math.abs(v('loe') - d('loe')));
  (audit.findings || []).filter(f => f.sev === 'crit' || f.sev === 'warn').forEach(f => {
    if (f.cat === 'OOP') push('oop_shape','Problema detectado: ' + f.title, 1);
    if (f.cat === 'IP') push('ip_shape','Problema detectado: ' + f.title, 1);
  });
  areas.sort((a, b) => b.w - a.w);
  const pmap = Object.fromEntries((players || []).map(p => [p.id, p]));
  const indiv = t.slots.map(s => {
    const p = pmap[s.pid]; if (!p) return null;
    const ri = ROLE_BY_ID[s.ip], ro = ROLE_BY_ID[s.oop];
    const si = (roleScore(p, ri) || { score: 0 }).score, so = (roleScore(p, ro) || { score: 0 }).score;
    const focusRole = si <= so ? ri : ro;
    const weak = [...ri.key, ...ro.key].filter((k, i, a) => a.indexOf(k) === i && p.a[k] != null).sort((a, b) => p.a[a] - p.a[b]).slice(0, 2);
    return { p, slot: s.ipCode, role: focusRole, si, so, weak };
  }).filter(Boolean);
  const mentors = (players || []).filter(p => (p.a.det || 0) >= 15 && (p.age || 0) >= 26);
  const youth = (players || []).filter(p => p.age != null && p.age <= 21);
  const cohesion = audit && audit.complexity && audit.complexity.cx > (audit.cap || 12);
  return { areas, indiv, mentors, youth, cohesion };
}

// ---------- Inspector de .fmf ----------
function hex(bytes) { return [...bytes].map(b => b.toString(16).padStart(2, '0')).join(' '); }
function entropy(u8) {
  const c = new Array(256).fill(0); u8.forEach(b => c[b]++);
  let e = 0; c.forEach(n => { if (n) { const p = n / u8.length; e -= p * Math.log2(p); } }); return e;
}
async function inflateFrom(u8, offset, format) {
  if (typeof DecompressionStream === 'undefined') return null;
  try {
    const ds = new DecompressionStream(format);
    const writer = ds.writable.getWriter();
    writer.write(u8.slice(offset)).catch(() => {}); writer.close().catch(() => {});
    const reader = ds.readable.getReader(); const chunks = []; let total = 0;
    try { while (true) { const { done, value } = await reader.read(); if (done) break; chunks.push(value); total += value.length; if (total > 64e6) break; } } catch (e) { /* dados extra após o stream */ }
    if (!total) return null;
    const out = new Uint8Array(total); let o = 0; chunks.forEach(c => { out.set(c, o); o += c.length; }); return out;
  } catch (e) { return null; }
}
function extractStrings(u8, min = 4) {
  const out = []; let cur = '';
  for (let i = 0; i < u8.length; i++) { const b = u8[i]; if (b >= 32 && b < 127) cur += String.fromCharCode(b); else { if (cur.length >= min) out.push(cur); cur = ''; } }
  if (cur.length >= min) out.push(cur);
  // UTF-16LE
  cur = '';
  for (let i = 0; i + 1 < u8.length; i += 2) { const b = u8[i], z = u8[i + 1]; if (z === 0 && b >= 32 && b < 127) cur += String.fromCharCode(b); else { if (cur.length >= min) out.push(cur); cur = ''; } }
  if (cur.length >= min) out.push(cur);
  // UTF-8 com acentos
  try { const txt = new TextDecoder('utf-8', { fatal: false }).decode(u8); (txt.match(/[\p{L}\p{N} '\-]{5,}/gu) || []).forEach(s => out.push(s.trim())); } catch (e) {}
  return [...new Set(out.map(s => s.trim()).filter(s => s.length >= min))];
}
// Contentor FM27 (.fmf), mapeado a partir de ficheiros reais:
//   [02 01][tag 4B invertida][08 00 00] por secção
//   secção 'afe.' (".efa"): u32 tamanho (inclui o próprio campo) + blocos [u32 16][u32 16][nonce 16][tag 16][cifrado]
//   secção 'fmf.' (".fmf"): frame zstd com o índice (nome do arquivo, ficheiros, offsets, tamanhos)
// Os blocos são frames zstd cifradas com cifra autenticada (padrão AES-GCM) — não legíveis sem a chave do jogo.
function parseFmfContainer(u8) {
  const dv = new DataView(u8.buffer, u8.byteOffset, u8.byteLength);
  const sections = []; let p = 0;
  while (p + 9 <= u8.length && u8[p] === 0x02 && u8[p + 1] === 0x01) {
    const tag = String.fromCharCode(...u8.slice(p + 2, p + 6));
    const start = p + 9;
    if (u8[start] === 0x28 && u8[start + 1] === 0xb5 && u8[start + 2] === 0x2f && u8[start + 3] === 0xfd) {
      sections.push({ tag, name: tag.split('').reverse().join(''), off: p, start, len: u8.length - start, kind: 'zstd' }); break;
    }
    const len = start + 4 <= u8.length ? dv.getUint32(start, true) : 0;
    if (!len || start + len > u8.length) break;
    sections.push({ tag, name: tag.split('').reverse().join(''), off: p, start, len, kind: 'blob' });
    p = start + len;
  }
  const blocks = [];
  sections.filter(x => x.kind === 'blob').forEach(x => {
    const marks = [];
    for (let i = x.start + 4; i + 8 <= x.start + x.len; i++) if (dv.getUint32(i, true) === 16 && dv.getUint32(i + 4, true) === 16) { marks.push(i); i += 7; }
    marks.forEach((m, k) => { const end = k + 1 < marks.length ? marks[k + 1] : x.start + x.len; blocks.push({ off: m, size: end - m, cipher: end - m - 40 }); });
  });
  const recognized = sections.length >= 2 && sections.some(x => x.kind === 'zstd') && blocks.length > 0;
  const largest = blocks.reduce((a, b) => (!a || b.size > a.size ? b : a), null);
  return { recognized, sections, blocks, largest };
}

async function inspectFmf(file) {
  const buf = new Uint8Array(await file.arrayBuffer());
  const r = { name: file.name, size: buf.length, head: hex(buf.slice(0, 32)), headAscii: [...buf.slice(0, 32)].map(b => b >= 32 && b < 127 ? String.fromCharCode(b) : '.').join(''), entropy: entropy(buf).toFixed(2), streams: [], magic: [], strings: [], matches: { roles: [], formations: [], instructions: [], mentality: [] } };
  const sigs = [];
  for (let i = 0; i < buf.length - 4; i++) {
    const a = buf[i], b = buf[i + 1];
    if (a === 0x78 && (b === 0x01 || b === 0x5e || b === 0x9c || b === 0xda) && ((a << 8) | b) % 31 === 0) sigs.push([i, 'deflate']);
    else if (a === 0x1f && b === 0x8b && buf[i + 2] === 0x08) sigs.push([i, 'gzip']);
    else if (a === 0x28 && b === 0xb5 && buf[i + 2] === 0x2f && buf[i + 3] === 0xfd) r.magic.push(`zstd @${i}`);
    else if (a === 0x04 && b === 0x22 && buf[i + 2] === 0x4d && buf[i + 3] === 0x18) r.magic.push(`lz4 @${i}`);
    if (sigs.length > 40) break;
  }
  const blobs = [buf];
  for (const [off, fmt] of sigs.slice(0, 40)) {
    const out = await inflateFrom(buf, off, fmt);
    if (out && out.length > 32) { r.streams.push({ off, fmt, len: out.length }); blobs.push(out); }
  }
  const strs = new Set(); blobs.forEach(b => extractStrings(b).forEach(s => strs.add(s)));
  r.strings = [...strs];
  const low = r.strings.map(s => s.toLowerCase());
  const has = (term) => low.some(s => s.includes(term.toLowerCase()));
  r.matches.roles = [...new Set(ROLES.map(x => x.name))].filter(has);
  r.matches.formations = Object.keys(FORMATIONS).map(f => f.split(' ')[0]).filter((f, i, a) => a.indexOf(f) === i).filter(has);
  r.matches.instructions = TEAM_INSTRUCTIONS.map(x => x.n).filter(has);
  r.matches.mentality = MENTALITIES.filter(has);
  r.decoded = r.matches.roles.length + r.matches.instructions.length > 0;
  r.container = parseFmfContainer(buf);
  return r;
}
