// ============================================================
// Navegação, render, arrastar-e-largar e ponte com a app instalada
// ============================================================
const NAV = [
  ['home', 'Início', '⌂'],
  ['team', 'A minha equipa', '☰'],
  ['tactic', 'Tática', '◎'],
  ['training', 'Treino', '◷'],
  ['bible', 'Bíblia', '◆'],
  ['settings', 'Definições', '⚙'],
];
const TABS = {
  team: [['squad', 'Plantel'], ['xi', 'Melhor XI & sugestões'], ['opp', 'Adversário']],
  tactic: [['analyze', 'Analisar'], ['sim', 'Campo animado'], ['sheet', 'Ficha para o FM27'], ['import', 'Importar']],
  bible: [['roles', 'Papéis'], ['ti', 'Instruções'], ['concepts', 'Conceitos & adversários'], ['btraining', 'Treino & Effectiveness']],
};

function renderNav() {
  const sq = players().length, op = oppPlayers().length;
  return `<div class="brand"><div class="logo">FM27</div><div><b>Tactical Bible</b><span>v${VERSION}${DESKTOP ? ' · app' : ' · browser'}</span></div></div>` +
    NAV.map(([id, label, ic]) => `<button class="nav-btn ${ST.view === id ? 'active' : ''}" onclick="A.go('${id}')"><span class="ic">${ic}</span>${label}</button>`).join('') +
    `<div class="aside-status">
      <div><span class="dot ${sq ? 'ok' : ''}"></span>${sq ? `${sq} jogadores${ST.myClub ? ' · ' + esc(ST.myClub) : ''}` : 'Sem plantel'}</div>
      <div><span class="dot ${op ? 'ok' : ''}"></span>${op ? 'Adversário carregado' : 'Sem adversário'}</div>
      ${DESKTOP ? `<div><span class="dot ${ST.auto.save && !ST.auto.saveErr ? 'ok' : ''}"></span>Save: ${SAVE_BUSY ? 'a ler…' : ST.auto.saveErr ? 'erro (ver Definições)' : ST.auto.save ? 'automático' : 'manual'}</div>
      <div><span class="dot ${ST.auto.on ? 'ok' : ''}"></span>F9 automático ${ST.auto.on ? 'ligado' : 'desligado'}</div>` : ''}
    </div>
    <div class="aside-foot">Arraste qualquer ficheiro (CSV, HTML, .fmf, .json) para a janela.</div>`;
}

function tabsHTML(view) {
  return `<div class="tabs" role="tablist">${TABS[view].map(([k, n]) => `<button role="tab" aria-selected="${ST.tabs[view] === k}" class="${ST.tabs[view] === k ? 'on' : ''}" onclick="A.tab('${view}','${k}')">${n}</button>`).join('')}</div>`;
}

function render(scrollTop) {
  const main = document.getElementById('main'); const y = main.scrollTop;
  document.getElementById('nav').innerHTML = renderNav();
  const v = ST.view, tab = ST.tabs[v];
  const isSim = v === 'tactic' && tab === 'sim';
  if (!isSim) cancelAnimationFrame(SIM.raf);
  let body;
  if (v === 'home') body = vHome();
  else if (v === 'team') body = tabsHTML('team') + (tab === 'squad' ? vSquad() : tab === 'xi' ? vXI() : vOpp());
  else if (v === 'tactic') body = tabsHTML('tactic') + (tab === 'analyze' ? vTactic() : tab === 'sim' ? vSim() : tab === 'sheet' ? vSheet() : vImport());
  else if (v === 'training') body = vTraining();
  else if (v === 'bible') body = tabsHTML('bible') + (tab === 'roles' ? vRoles() : tab === 'ti' ? vTI() : tab === 'concepts' ? vConcepts() : vBTraining());
  else body = vSettings();
  main.innerHTML = body;
  main.scrollTop = scrollTop ? 0 : y;
  if (isSim) simStart();
}

// ---------- ficheiros: rota única para tudo o que entra ----------
function ingestJSON(d, name) {
  if (d.kind === 'fm27tb-tactic' && d.tactic) { ST.tactic = d.tactic; save(); A.go('tactic', 'analyze'); toast('Tática importada'); }
  else if (d.kind === 'fm27tb-all') { Object.assign(ST, { tactic: d.tactic || ST.tactic, squad: d.squad || ST.squad, opp: d.opp || ST.opp, saved: d.saved || [], myClub: d.myClub || ST.myClub }); save(); render(true); toast('Dados restaurados'); }
  else if (Array.isArray(d) || Array.isArray(d.players)) {
    // JSON de outras ferramentas (ex.: FMLibreEditor): lista de objectos → tabela
    const flat = (o, pre = '', out = {}) => { Object.entries(o || {}).forEach(([k, v]) => { if (v && typeof v === 'object' && !Array.isArray(v)) flat(v, pre + k + '_', out); else out[pre + k] = Array.isArray(v) ? v.join(', ') : v; }); return out; };
    const rows = (Array.isArray(d) ? d : d.players).map(r => flat(r)); const keys = [...new Set(rows.flatMap(r => Object.keys(r)))];
    const csv = [keys.join('\t')].concat(rows.map(r => keys.map(k => String(r[k] ?? '').replace(/\t/g, ' ')).join('\t'))).join('\n');
    ingestSquadText(csv, name); render();
  } else toast('Ficheiro JSON não reconhecido');
}
async function ingestFile(f) {
  const n = f.name.toLowerCase();
  if (n.endsWith('.fmf')) { ST.tabs.tactic = 'import'; ST.view = 'tactic'; render(true); return A.fmf(f); }
  const txt = await f.text();
  if (n.endsWith('.json')) { try { return ingestJSON(JSON.parse(txt), f.name); } catch (e) { return toast('JSON inválido'); } }
  const where = ingestSquadText(txt, f.name);
  if (where) { ST.view = 'team'; ST.tabs.team = where === 'opp' ? 'opp' : 'squad'; }
  render(true);
}
function setupDnD() {
  let depth = 0; const ov = document.getElementById('dropov');
  window.addEventListener('dragenter', (e) => { if (![...e.dataTransfer.types].includes('Files')) return; depth++; ov.classList.add('on'); });
  window.addEventListener('dragleave', () => { depth = Math.max(0, depth - 1); if (!depth) ov.classList.remove('on'); });
  window.addEventListener('dragover', (e) => e.preventDefault());
  window.addEventListener('drop', (e) => { e.preventDefault(); depth = 0; ov.classList.remove('on'); [...e.dataTransfer.files].forEach(ingestFile); });
}

// ---------- app instalada: apanha o último F9 do FM View Exporter ----------
let POLLING = false;
async function desktopPoll(force) {
  if (!DESKTOP || (!ST.auto.on && !force) || POLLING) return;
  POLLING = true;
  try {
    const r = await window.__TAURI__.core.invoke('latest_export', { customDir: ST.auto.dir || null, since: ST.auto.mtime || null });
    if (r && r.content) {
      ST.auto.mtime = r.modified; ST.auto.last = r.name;
      const where = ingestSquadText(r.content, r.name + ' (F9)');
      save(); if (where) render();
    } else if (force) toast('Nenhuma exportação nova encontrada');
  } catch (e) { if (force) toast('Não consegui ler a pasta de exportação'); }
  POLLING = false;
}
// ---------- app instalada: lê o save (.fm) com o fmsave sempre que o jogo grava ----------
let SAVE_BUSY = false;
async function savePoll(force) {
  if (!DESKTOP || (!ST.auto.save && !force) || SAVE_BUSY) return;
  let info = null;
  try { info = await window.__TAURI__.core.invoke('latest_save', { customDir: ST.auto.saveDir || null }); } catch (e) { return; }
  ST.lastSave = info;
  if (!info) { if (force) toast('Não encontrei saves do FM27 — indique a pasta em Definições'); return; }
  if (!force && info.modified <= (ST.auto.saveMtime || 0)) return;
  SAVE_BUSY = true; ST.auto.saveMtime = info.modified;           // marca já, para não repetir em ciclo se falhar
  toast('A ler o save ' + info.name + '…'); render();
  try {
    const r = await window.__TAURI__.core.invoke('export_save', { path: info.path });
    ST.auto.saveErr = ''; ST.auto.saveLast = `${r.save} · ${new Date(r.modified * 1000).toLocaleString('pt-PT')}`;
    ingestSquadText(r.content, r.save + ' (save)', { force: 'squad' });
  } catch (e) { ST.auto.saveErr = String(e); toast('Não consegui ler o save — veja Definições'); }
  SAVE_BUSY = false; save(); render();
}

async function desktopDirs() {
  if (!DESKTOP) return [];
  try { return await window.__TAURI__.core.invoke('export_dirs', { customDir: ST.auto.dir || null }); } catch (e) { return []; }
}

window.addEventListener('DOMContentLoaded', () => {
  load(); applyTheme(); render(); setupDnD();
  if (DESKTOP) { desktopPoll(); setInterval(desktopPoll, 3000); savePoll(); setInterval(savePoll, 10000); }
});
