// ============================================================
// Estado, persistência e utilitários
// ============================================================
const VERSION = '3.1.0';

const STORE_KEY = 'fm27tb.v1';

const esc = (s) => String(s ?? '').replace(/[&<>"']/g, c => ({ '&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;' }[c]));

const fitColor = (s) => s == null ? 'var(--muted)' : s >= 70 ? '#1c8a4d' : s >= 58 ? '#5a9a2e' : s >= 48 ? '#b7791f' : '#c2412d';

const attrName = (k) => (ATTRS[k] ? ATTRS[k].n : k);

const EMPTY_SQUAD = () => ({ players: [], source: '', warnings: [], mapped: [], at: null });
const ST = {
  view: 'home', tabs: { team: 'squad', tactic: 'analyze', bible: 'roles' },
  tactic: newTactic(), squad: EMPTY_SQUAD(), opp: EMPTY_SQUAD(), myClub: null,
  saved: [], sel: null, pitchMode: 'IP', rq: '', rg: '', rp: '', fmf: null, sugg: null, theme: 'auto',
  auto: { on: true, dir: '', mtime: 0, last: '', save: true, saveDir: '', saveMtime: 0, saveErr: '', saveLast: '' },   // carregamento automático (app instalada)
  lastSave: null,
};
const DESKTOP = typeof window !== 'undefined' && !!(window.__TAURI__ && window.__TAURI__.core);

function save() {
  try { localStorage.setItem(STORE_KEY, JSON.stringify({ tactic: ST.tactic, squad: ST.squad, opp: ST.opp, myClub: ST.myClub, saved: ST.saved, theme: ST.theme, auto: ST.auto, tabs: ST.tabs })); } catch (e) {}
}

function load() {
  try {
    const d = JSON.parse(localStorage.getItem(STORE_KEY) || 'null');
    if (!d) { ST.tactic = tacticFromTemplate(TEMPLATES[0]); ST.saved = [JSON.parse(JSON.stringify(ST.tactic))]; return; }
    if (d.tactic && FORMATIONS[d.tactic.ipForm] && FORMATIONS[d.tactic.oopForm]) ST.tactic = d.tactic;
    if (d.squad && Array.isArray(d.squad.players)) ST.squad = d.squad;
    if (Array.isArray(d.saved)) ST.saved = d.saved;
    if (d.theme) ST.theme = d.theme;
    if (d.opp && Array.isArray(d.opp.players)) ST.opp = d.opp;
    if (d.myClub) ST.myClub = d.myClub;
    if (d.auto) ST.auto = Object.assign(ST.auto, d.auto);
    if (d.tabs) ST.tabs = Object.assign(ST.tabs, d.tabs);
    [ST.tactic, ...ST.saved].forEach(t => { if (t && t.ti && t.ti.loe > 2) t.ti.loe = 2; if (t && t.ti && t.ti.short_gk > 1) t.ti.short_gk = 1; });
  } catch (e) {}
}

function toast(msg) { const t = document.getElementById('toast'); t.textContent = msg; t.classList.add('on'); clearTimeout(t._h); t._h = setTimeout(() => t.classList.remove('on'), 2200); }

function applyTheme() { if (ST.theme === 'auto') document.documentElement.removeAttribute('data-theme'); else document.documentElement.setAttribute('data-theme', ST.theme); }

function players() { return ST.squad.players || []; }
function oppPlayers() { return ST.opp.players || []; }

// Carrega uma exportação (CSV/HTML/texto) e decide se é o teu plantel ou o do adversário pelo clube
function ingestSquadText(text, sourceName, opts = {}) {
  const r = parseSquad(String(text || '').replace(/^\uFEFF/, ''));
  if (!r.players.length) { toast('Não encontrei jogadores em ' + sourceName); return null; }
  const mc = majorityClub(r.players);
  const pack = { players: r.players, warnings: r.warnings, mapped: r.mapped, source: sourceName, at: new Date().toISOString() };
  let target = opts.force || 'squad';
  if (!opts.force && mc.club && mc.share < 0.6) { toast('Exportação com vários clubes (pesquisa?) ignorada para o plantel'); return null; }
  if (!opts.force && mc.club && ST.myClub && mc.club !== ST.myClub) target = 'opp';
  if (target === 'squad') {
    ST.squad = pack; if (mc.club) ST.myClub = mc.club; ST.sugg = null;
    const ids = new Set(r.players.map(p => p.id)); ST.tactic.slots.forEach(s => { if (s.pid && !ids.has(s.pid)) s.pid = null; });
    if (ST.tactic.slots.every(s => !s.pid) && r.players.length >= 11) autoPickXI(ST.tactic, r.players);
    toast(`Plantel${mc.club ? ' do ' + mc.club : ''}: ${r.players.length} jogadores`);
  } else {
    ST.opp = pack; toast(`Adversário${mc.club ? ' ' + mc.club : ''}: ${r.players.length} jogadores — relatório pronto`);
  }
  save();
  return target;
}

function pById(id) { return players().find(p => p.id === id) || null; }

function download(name, text, type = 'application/json') {
  try {
    const b = new Blob([text], { type }); const a = document.createElement('a');
    a.href = URL.createObjectURL(b); a.download = name; document.body.appendChild(a); a.click(); a.remove();
    setTimeout(() => URL.revokeObjectURL(a.href), 2000);
  } catch (e) { navigator.clipboard && navigator.clipboard.writeText(text).then(() => toast('Copiado para a área de transferência')); }
}

function makeSample() {
  let seed = 7; const rnd = () => (seed = (seed * 9301 + 49297) % 233280) / 233280;
  const first = ['Rui','Tiago','Bruno','Diogo','Nuno','André','Pedro','João','Gonçalo','Miguel','Rafael','Hugo','Luís','Vasco','Duarte','Tomás','Martim','Simão','Afonso','Henrique','Leandro','Fábio','Ivo','Caio'];
  const last = ['Matos','Ferraz','Lobo','Teixeira','Pinheiro','Carvalho','Sardinha','Valente','Brito','Quaresma','Antunes','Seixas','Cordeiro','Maia','Rebelo','Viana','Barros','Neves','Pacheco','Leal','Morais','Bastos','Paiva','Rocha'];
  const prof = [
    ['GK', { han:14, ref:15, ovo:13, pos:13, cnt:13, agi:13, aer:13, cmd:12, com:12, kic:12, pas:11, fir:10, cmp:12, dec:12, tro:13, ant:12, acc:10 }],
    ['GK', { han:12, ref:13, ovo:11, pos:12, cnt:11, agi:12, aer:12, cmd:11, com:11, kic:11, pas:9, fir:9, cmp:10, dec:10, tro:10, ant:10 }],
    ['D (C)', { hea:15, mar:14, tck:14, pos:14, str:15, jum:15, cmp:12, dec:12, cnt:13, ant:13, pac:11, acc:10, pas:11, bra:14, agg:12 }],
    ['D (C)', { hea:13, mar:13, tck:14, pos:13, str:12, jum:12, cmp:14, dec:13, cnt:12, ant:13, pac:14, acc:14, pas:14, vis:12, fir:12, tec:12 }],
    ['D (C)', { hea:14, mar:12, tck:13, pos:12, str:14, jum:14, cmp:10, dec:11, cnt:11, ant:11, pac:12, acc:11, pas:9, bra:13 }],
    ['D (RL), WB (R)', { tck:13, mar:12, pos:12, cro:13, dri:12, wor:15, sta:15, acc:15, pac:15, tea:13, otb:12, ant:12, pas:12, cnt:11, dec:11 }],
    ['D (L), WB (L)', { tck:12, mar:12, pos:11, cro:14, dri:13, wor:14, sta:15, acc:14, pac:14, tea:13, otb:13, tec:12, pas:12, fir:12, dec:11 }],
    ['D (RL)', { tck:14, mar:14, pos:14, cnt:13, ant:13, tea:13, cro:10, pas:12, cmp:13, dec:13, str:13, hea:13, wor:12, sta:13, acc:12 }],
    ['DM, M (C)', { tck:15, pos:15, ant:14, cnt:14, dec:14, tea:15, mar:13, pas:13, cmp:13, str:14, wor:15, sta:14, fir:12, vis:11 }],
    ['DM, M (C)', { pas:16, fir:15, tec:15, cmp:15, dec:15, vis:16, tea:14, ant:13, pos:12, tck:10, wor:12, sta:12, bal:13 }],
    ['M (C)', { pas:13, tck:12, otb:14, tea:14, wor:16, sta:16, dri:12, fin:12, fir:13, lon:13, ant:13, cmp:12, dec:12, acc:13, str:13 }],
    ['M (C), AM (C)', { pas:15, fir:15, tec:16, vis:15, cmp:14, dec:14, otb:14, tea:12, dri:14, fla:15, ant:13, lon:14, agi:14 }],
    ['M (C)', { pas:12, tck:13, dec:12, tea:13, fir:12, wor:14, sta:14, pos:12, ant:12, cmp:11 }],
    ['AM (RL), ST (C)', { dri:15, fin:14, fir:14, tec:15, otb:14, cmp:13, acc:16, agi:15, pac:15, fla:14, pas:12, lon:12, ant:12, dec:12, wor:12, sta:13 }],
    ['AM (R), M (R)', { cro:15, dri:15, tec:14, acc:16, agi:14, pac:16, fir:13, pas:12, otb:13, fla:13, wor:12, sta:13, tea:11 }],
    ['AM (L)', { dri:14, fin:12, fir:13, tec:14, otb:12, cmp:12, acc:14, agi:14, pac:13, pas:13, vis:13, fla:13, wor:11, sta:12 }],
    ['ST (C)', { fin:15, fir:14, hea:14, tec:13, cmp:14, otb:15, acc:14, str:14, ant:14, dri:12, pac:13, cnt:12, wor:12, tea:12, dec:12 }],
    ['ST (C)', { hea:16, str:16, jum:15, bra:14, fin:13, cmp:12, bal:14, fir:12, otb:12, tea:13, agg:13, ant:12, pac:10, acc:10 }],
    ['AM (C), ST (C)', { fin:13, fir:13, otb:14, cmp:12, ant:13, acc:14, dri:13, tec:13, dec:12, pas:12, lon:12, agi:13, wor:13 }],
    ['D (C), DM', { mar:13, tck:13, pos:13, ant:13, cmp:13, cnt:12, dec:13, tea:13, pas:12, str:13, jum:13, hea:13, fir:11 }],
    ['WB (L), M (L)', { cro:13, dri:13, tec:12, otb:13, tea:12, wor:14, acc:15, pac:15, sta:14, fir:12, pas:11, tck:10, mar:9 }],
    ['M (R), AM (R)', { pas:13, cro:13, tea:14, wor:15, dec:12, sta:15, fir:12, tec:12, otb:12, tck:11, pos:11, acc:13 }],
  ];
  return prof.map((pr, i) => {
    const a = {}; Object.keys(ATTRS).forEach(k => { a[k] = 6 + Math.round(rnd() * 5); });
    if (!pr[0].startsWith('GK')) ['aer','cmd','com','ecc','han','kic','ovo','pun','ref','tro','thr'].forEach(k => a[k] = 1 + Math.round(rnd() * 3));
    Object.entries(pr[1]).forEach(([k, v]) => a[k] = Math.max(1, Math.min(20, v + Math.round(rnd() * 2 - 1))));
    a.det = 8 + Math.round(rnd() * 10);
    return { id: 'sample' + i, name: `${first[i % first.length]} ${last[(i * 7) % last.length]}`, posRaw: pr[0], pos: parsePositions(pr[0]), age: 18 + Math.round(rnd() * 15), a };
  });
}

