// ============================================================
// Navegação, render, arrastar-e-largar e ponte com a app instalada
// ============================================================
const NAV = [
  ['home', 'Início', '⌂'],
  ['team', 'A minha equipa', '☰'],
  ['scout', 'Prospeção', '🔭'],
  ['tactic', 'Tática', '◎'],
  ['training', 'Treino', '◷'],
  ['bible', 'Bíblia', '◆'],
  ['settings', 'Definições', '⚙'],
];
const TABS = {
  team: [['squad', 'Plantel'], ['gen', 'Gerar tática'], ['xi', 'Melhor XI & sistemas'], ['opp', 'Adversário']],
  tactic: [['analyze', 'Analisar'], ['sim', 'Campo animado'], ['plans', 'Situações de jogo'], ['setpieces', 'Bolas paradas'], ['sheet', 'Ficha para o FM27'], ['import', 'Importar']],
  bible: [['roles', 'Papéis'], ['ti', 'Instruções'], ['concepts', 'Conceitos & adversários'], ['btraining', 'Treino & Effectiveness']],
};

// Barra de topo ao estilo do FM: menu principal + estado + dados do jogo; por baixo, os separadores do ecrã (como no FM).
function renderNav() {
  const sq = players().length, op = oppPlayers().length, v = ST.view;
  const fb = (typeof FMB !== 'undefined' && FMB.state) || {};
  const stat = (on, label, tip) => `<span class="tb-stat" title="${esc(tip || label)}"><span class="dot ${on ? 'ok' : ''}"></span><span class="lbl">${label}</span></span>`;
  const saveTip = DESKTOP ? 'Save: ' + (SAVE_BUSY ? L('a ler…', 'reading…') : ST.auto.saveErr ? L('erro (ver Definições)', 'error (see Settings)') : ST.auto.save ? L('automático', 'automatic') : 'manual')
    + ' · F9: ' + (!ST.f9Found ? L('plugin não encontrado', 'plugin not found') : ST.auto.on ? L('automático', 'automatic') : 'manual') : '';
  const fmTip = fb.ok ? (fb.ready ? L('FM ligado (Tactical Bible Bridge)', 'FM connected (Tactical Bible Bridge)') : L('FM aberto, à espera do jogo', 'FM open, waiting for the game')) : L('FM não ligado (plugin opcional, ver Definições)', 'FM not connected (optional plugin, see Settings)');
  const main = NAV.filter(([id]) => id !== 'settings');
  const tabs = TABS[v] ? TABS[v].map(([k, n]) => `<button role="tab" aria-selected="${ST.tabs[v] === k}" class="${ST.tabs[v] === k ? 'on' : ''}" onclick="A.tab('${v}','${k}')">${n}</button>`).join('')
    : `<span class="tb-crumb">${(NAV.find(([id]) => id === v) || [, ''])[1]}</span>`;
  return `<div class="tb-row">
      <button class="tb-brand" onclick="A.go('home')" title="FM27 Tactical Bible v${VERSION}${DESKTOP ? ' · app' : ' · browser'}" aria-label="FM27 Tactical Bible"><span class="logo">FM27</span><span class="lbl">Tactical Bible</span></button>
      <nav class="tb-nav" aria-label="${L('Menu', 'Menu')}">${main.map(([id, label, ic]) => `<button class="nav-btn ${v === id ? 'active' : ''}" onclick="A.go('${id}')" title="${label}" aria-label="${label}"${v === id ? ' aria-current="page"' : ''}><span class="ic" aria-hidden="true">${ic}</span><span class="lbl">${label}</span></button>`).join('')}</nav>
      <div class="tb-right">
        <div class="tb-status">
          ${stat(sq, sq ? `${L(`${sq} jogadores`, `${sq} players`)}${ST.myClub ? ' · ' + esc(ST.myClub) : ''}` : 'Sem plantel', saveTip)}
          ${stat(op, op ? 'Adversário carregado' : 'Sem adversário')}
          ${DESKTOP ? stat(fb.ok && fb.ready, 'FM', fmTip) : ''}
        </div>
        ${dataCard()}
        <div class="seg lang-seg" role="group" aria-label="Idioma">${[['en', 'EN'], ['pt', 'PT']].map(([k, n]) => `<button class="${LANG === k ? 'on' : ''}" onclick="A.lang('${k}')" aria-pressed="${LANG === k}">${n}</button>`).join('')}</div>
        <button class="tb-icon ${v === 'settings' ? 'on' : ''}" onclick="A.go('settings')" title="Definições" aria-label="Definições">⚙</button>
      </div>
    </div>
    <div class="tb-sub" role="tablist">${tabs}</div>
    <div id="busy" aria-live="polite"></div>`;
}

function render(scrollTop) {
  const main = document.getElementById('main'); const y = main.scrollTop;
  document.getElementById('nav').innerHTML = renderNav();
  const v = ST.view, tab = ST.tabs[v];
  const isSim = v === 'tactic' && tab === 'sim';
  if (!isSim) cancelAnimationFrame(SIM.raf);
  let body;
  if (v === 'home') body = vHome();
  else if (v === 'team') body = (tab === 'squad' ? vSquad() : tab === 'gen' ? vGen() : tab === 'xi' ? vXI() : vOpp());
  else if (v === 'tactic') body = (tab === 'analyze' ? vTactic() : tab === 'sim' ? vSim() : tab === 'plans' ? vGamePlans() : tab === 'setpieces' ? vSetPieces() : tab === 'sheet' ? vSheet() : vImport());
  else if (v === 'training') body = vTraining();
  else if (v === 'scout') body = vScout();
  else if (v === 'bible') body = (tab === 'roles' ? vRoles() : tab === 'ti' ? vTI() : tab === 'concepts' ? vConcepts() : vBTraining());
  else body = vSettings();
  main.innerHTML = body;
  document.querySelector('.app').classList.toggle('mini', !!ST.ui.navMini);
  paintBusy();
  translateDOM(document.getElementById('nav')); translateDOM(main);
  decoratePanels(v + '/' + (ST.tabs[v] || ''));          // depois de traduzir: o rótulo de acessibilidade já vem no idioma certo
  const ov = document.getElementById('dropov'); if (ov) ov.textContent = T('Largue o ficheiro — plantel (CSV/HTML/JSON), tática .fmf ou cópia .json');
  document.documentElement.lang = LANG;
  main.scrollTop = scrollTop ? 0 : y;
  if (isSim) simStart();
  if (v === 'bible') initClips(); else stopClips();
}

// ---------- actividade: tudo o que demora mostra que está a trabalhar ----------
// busyStart(chave, texto) → barra no topo + cartão com tempo decorrido; busyUpdate(chave, {p: 0–1, detail}); busyEnd(chave).
// Os botões com data-busy="chave" ficam bloqueados e com indicador enquanto a tarefa corre.
const BUSY = {};
let BUSY_TICK = 0;
function busyStart(key, label, opts = {}) { BUSY[key] = { label, start: Date.now(), p: opts.p ?? null, detail: opts.detail || '', slow: opts.slow || null, slowAfter: opts.slowAfter || 12 }; paintBusy(); }
function busyUpdate(key, patch) { if (BUSY[key]) { Object.assign(BUSY[key], patch); paintBusy(); } }
function busyEnd(key) { delete BUSY[key]; paintBusy(); }
async function withBusy(key, label, fn, opts) {
  busyStart(key, label, opts);
  await new Promise(r => requestAnimationFrame(() => setTimeout(r, 0)));   // deixa a janela desenhar o indicador antes de trabalhar
  try { return await fn((p, detail) => busyUpdate(key, { p, ...(detail != null ? { detail } : {}) })); }
  finally { busyEnd(key); }
}
function paintBusy() {
  const keys = Object.keys(BUSY), bar = document.getElementById('topbar'), box = document.getElementById('busy');
  if (!bar || !box) return;
  const one = keys.length === 1 && BUSY[keys[0]].p != null ? BUSY[keys[0]].p : null;
  bar.className = keys.length ? (one != null ? 'on det' : 'on') : '';
  bar.firstElementChild.style.width = one != null ? Math.round(one * 100) + '%' : '';
  box.innerHTML = keys.map(k => { const b = BUSY[k], s = Math.floor((Date.now() - b.start) / 1000);
    const extra = b.slow && s >= b.slowAfter ? b.slow : b.detail;
    const tip = `${b.label}${b.p != null ? ' — ' + Math.round(b.p * 100) + '%' : ''}${extra ? ' — ' + extra : ''}`;
    return `<div class="busy-item" role="status" title="${esc(tip)}"><span class="spin" aria-hidden="true"></span>${b.p != null ? `<span class="busy-pct">${Math.round(b.p * 100)}%</span>` : ''}<div class="busy-body"><b>${esc(b.label)}</b>
      ${b.p != null ? `<div class="busy-bar"><i style="width:${Math.round(b.p * 100)}%"></i></div>` : ''}
      <span class="small muted">${[b.p != null ? Math.round(b.p * 100) + '%' : '', s >= 1 ? s + ' s' : '', extra ? esc(extra) : ''].filter(Boolean).join(' · ') || 'a começar…'}</span></div></div>`; }).join('');
  box.className = keys.length ? 'on' : '';
  document.querySelectorAll('[data-busy]').forEach(el => { const on = keys.includes(el.dataset.busy); el.disabled = on; el.classList.toggle('is-busy', on); el.setAttribute('aria-busy', String(on)); });
  translateDOM(box);
  clearInterval(BUSY_TICK); if (keys.length) BUSY_TICK = setInterval(paintBusy, 500);
}

// ---------- dados do jogo: ler o save / F9 / abrir ficheiro — sempre à mão ----------
function relTime(ts) {
  const t0 = typeof ts === 'number' ? ts : Date.parse(ts); if (!t0) return ''; const s = (Date.now() - t0) / 1000;
  if (s < 90) return L('agora mesmo', 'just now'); const m = s / 60; if (m < 60) return L(`há ${Math.round(m)} min`, `${Math.round(m)} min ago`);
  const h = m / 60; if (h < 24) return L(`há ${Math.round(h)} h`, `${Math.round(h)} h ago`); const d = Math.round(h / 24); return L(`há ${d} dia(s)`, `${d} day(s) ago`);
}
function dataButtons(size = '') {
  if (DESKTOP) return `<button class="btn primary ${size}" data-busy="save" onclick="A.saveNow()" title="${L('Ler o plantel do último save (Ctrl+Shift+L)', 'Read the squad from your latest save (Ctrl+Shift+L)')}">📥 ${L('Ler o save agora', 'Read the save now')}</button>
    <button class="btn ${size}" data-busy="f9" onclick="A.autoNow()" title="${L('Carregar a última exportação F9 do FM View Exporter', 'Load the latest F9 export from FM View Exporter')}">⌨ ${L('Carregar último F9', 'Load latest F9')}</button>`;
  return `<button class="btn primary ${size}" onclick="A.pickFile()" title="${L('CSV/HTML/JSON do plantel (Ctrl+Shift+L)', 'Squad CSV/HTML/JSON (Ctrl+Shift+L)')}">📂 ${L('Abrir ficheiro do plantel', 'Open squad file')}</button>`;
}
function dataCard() {
  const at = ST.squad && ST.squad.at, sq = (ST.squad && ST.squad.players || []).length;
  const when = sq ? L(`Plantel lido ${relTime(at)}`, `Squad read ${relTime(at)}`) : L('Ainda sem plantel', 'No squad yet');
  const drop = L('Também pode arrastar qualquer ficheiro (CSV, HTML, .fmf, .json) para a janela.', 'You can also drag any file (CSV, HTML, .fmf, .json) into the window.');
  return `<div class="data-card" role="group" aria-label="${L('Dados do jogo', 'Game data')}">
    ${DESKTOP ? `<button class="btn primary sm data-btn" data-busy="save" onclick="A.saveNow()" title="${esc(L('Ler o save agora (Ctrl+Shift+L)', 'Read the save now (Ctrl+Shift+L)') + ' — ' + when)}" aria-label="${L('Ler o save agora', 'Read the save now')}"><span class="ic" aria-hidden="true">📥</span><span class="lbl">${L('Ler o save agora', 'Read the save now')}</span></button>
      <button class="btn sm data-btn" data-busy="f9" onclick="A.autoNow()" title="${L('Carregar último F9', 'Load latest F9')}" aria-label="${L('Carregar último F9', 'Load latest F9')}"><span class="ic" aria-hidden="true">⌨</span><span class="lbl">${L('Último F9', 'Latest F9')}</span></button>`
    : `<button class="btn primary sm data-btn" onclick="A.pickFile()" title="${esc(L('Abrir ficheiro do plantel (Ctrl+Shift+L)', 'Open squad file (Ctrl+Shift+L)') + ' — ' + when + '. ' + drop)}" aria-label="${L('Abrir ficheiro do plantel', 'Open squad file')}"><span class="ic" aria-hidden="true">📂</span><span class="lbl">${L('Abrir ficheiro', 'Open file')}</span></button>`}
  </div>`;
}

// ---------- painéis que fecham e abrem (estado guardado por ecrã e título) ----------
function decoratePanels(scope) {
  document.querySelectorAll('#main .panel').forEach(p => {
    const first = p.firstElementChild; if (!first || p.classList.contains('empty') || p.hasAttribute('data-fixed')) return;
    const h = /^H[23]$/.test(first.tagName) ? first : (first.classList.contains('row') || first.classList.contains('score-ring') ? first.querySelector('h2,h3') : null);
    if (!h || !p.children[1]) return;                       // sem corpo, nada para recolher
    const key = scope + '|' + h.textContent.replace(/\s+/g, ' ').trim().slice(0, 60);
    const titleText = h.textContent.replace(/\s+/g, ' ').trim().replace(/^\d+(?=\D)/, '');
    const btn = document.createElement('button');
    btn.type = 'button'; btn.className = 'pc-toggle'; btn.textContent = '▾';
    const set = (c) => { p.classList.toggle('collapsed', c); btn.textContent = c ? '▸' : '▾'; btn.setAttribute('aria-expanded', String(!c)); btn.title = c ? L('Expandir', 'Expand') : L('Recolher', 'Collapse'); btn.setAttribute('aria-label', btn.title + ': ' + T(titleText)); };
    btn.onclick = (e) => { e.stopPropagation(); const c = !p.classList.contains('collapsed'); set(c); if (c) ST.ui.collapsed[key] = 1; else delete ST.ui.collapsed[key]; save(); };
    h.prepend(btn); h.classList.add('pc-title'); h.onclick = (e) => { if (e.target === h) btn.click(); };
    p.classList.add('collapsible'); set(!!ST.ui.collapsed[key]);
  });
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
  const txt = await withBusy('file', L('A ler ', 'Reading ') + f.name, () => f.text(), { detail: (f.size / 1048576).toFixed(1) + ' MB' });
  busyStart('file', L('A interpretar ', 'Parsing ') + f.name); await new Promise(r => setTimeout(r, 30));
  try { return ingestParsed(txt, f, n); } finally { busyEnd('file'); }
}
function ingestParsed(txt, f, n) {
  // exportação do fmsave com todos os jogadores (milhares de linhas): vai para a prospeção, não para o plantel
  if (!n.endsWith('.json') && txt.length > 100000 && /ability_current/.test(txt.slice(0, 5000)) && (txt.match(/\n/g) || []).length > 400) return A.scoutIngest(txt, { save: f.name });
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
      busyStart('f9', 'A carregar a exportação F9', { detail: r.name }); await new Promise(res => setTimeout(res, 30));
      let where; try { where = ingestSquadText(r.content, r.name + ' (F9)'); } finally { busyEnd('f9'); }
      save(); if (where) render();
    } else if (force) toast(L('Nenhuma exportação nova encontrada', 'No new export found'));
  } catch (e) { if (force) toast(L('Não consegui ler a pasta de exportação', 'Could not read the export folder')); }
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
  busyStart('save', L('A ler o save ', 'Reading save ') + info.name, { detail: (info.size / 1048576).toFixed(0) + ' MB · ' + L('lê uma cópia, o save não é alterado', 'reads a copy, the save is not changed'),
    slow: L('na primeira leitura o fmsave é descarregado e o save inteiro é analisado — pode demorar 1–2 minutos', 'on the first read fmsave is downloaded and the whole save is analysed — it can take 1–2 minutes'), slowAfter: 12 }); render();
  try {
    const r = await window.__TAURI__.core.invoke('export_save', { path: info.path });
    ST.auto.saveErr = ''; ST.auto.saveLast = `${r.save} · ${new Date(r.modified * 1000).toLocaleString('pt-PT')}`;
    ingestSquadText(r.content, r.save + ' (save)', { force: 'squad' });
  } catch (e) { ST.auto.saveErr = String(e); toast('Não consegui ler o save — veja Definições'); }
  SAVE_BUSY = false; busyEnd('save'); save(); render();
}

async function desktopDirs() {
  if (!DESKTOP) return [];
  try { const d = await window.__TAURI__.core.invoke('export_dirs', { customDir: ST.auto.dir || null }); return Array.isArray(d) ? d : []; } catch (e) { return []; }
}

document.addEventListener('click', (e) => {
  const a = e.target.closest && e.target.closest('a[href^="https://"]');
  if (!a || !DESKTOP) return;
  e.preventDefault(); window.__TAURI__.core.invoke('open_url', { url: a.href }).catch(() => toast(a.href));
});
window.addEventListener('keydown', (e) => {
  if (e.key === 'Escape' && ST.scout && ST.scout.open) { A.scoutClose(); return; }
  if (e.key === 'Escape' && ST.squadOpen) { A.squadClose(); return; }
  if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === 'l') { e.preventDefault(); if (DESKTOP) A.saveNow(); else A.pickFile(); return; } if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'b') { e.preventDefault(); A.navMini(); } });
window.addEventListener('DOMContentLoaded', () => {
  load(); LANG = ST.lang || 'en'; applyTheme(); render(); setupDnD();
  if (DESKTOP) {
    desktopPoll(); setInterval(desktopPoll, 3000); savePoll(); setInterval(savePoll, 10000);
    const chk = () => desktopDirs().then(d => { const f = d.some(x => x[1]); if (f !== ST.f9Found) { ST.f9Found = f; render(); } });
    chk(); setInterval(chk, 30000);
  }
});
