// ============================================================
// Laboratório — ligação ao FM Tactics Lab (motor original do FM27)
// ============================================================
function labState() { if (!ST.lab) ST.lab = { port: 8765, status: null, jobs: [], job: null, err: '', busy: false }; return ST.lab; }
async function labCall(method, path, body, timeoutMs) {
  const L = labState();
  const r = await window.__TAURI__.core.invoke('lab_request', { port: L.port, method, path, body: body == null ? null : JSON.stringify(body), timeoutMs: timeoutMs || 4000 });
  let data = r.body; try { data = JSON.parse(r.body); } catch (e) {}
  if (r.status >= 400 || r.status === 0) throw new Error((data && data.error) || ('HTTP ' + r.status));
  return data;
}
let LAB_POLL = 0;
function labPoll() {
  clearTimeout(LAB_POLL); const L = labState();
  if (!L.job || !['queued', 'running'].includes(L.job.status)) { busyEnd('lab-job'); return; }
  busyStart('lab-job', 'Motor do FM27: ' + (L.job.name || 'experiência'), { p: (L.job.completed || 0) / Math.max(1, L.job.total || 1),
    detail: `${L.job.completed || 0}/${L.job.total || 0} jogos${L.job.eta_seconds ? ' · faltam ~' + Math.ceil(L.job.eta_seconds / 60) + ' min' : ''}` });
  if (ST.view !== 'lab') return;
  LAB_POLL = setTimeout(async () => { try { L.job = await labCall('GET', '/api/jobs/' + L.job.id);
    const it = L.jobs.find(j => j.id === L.job.id); if (it) Object.assign(it, { status: L.job.status, completed: L.job.completed, total: L.job.total });
    if (!['queued', 'running'].includes(L.job.status)) toast('Experiência concluída no motor do FM27');
    render(); labPoll(); } catch (e) { L.err = e.message; render(); } }, 3000);
}
const pct = (x) => x == null ? '—' : (x * 100).toFixed(0) + '%';
const f2 = (x) => x == null ? '—' : x.toFixed(2);
function ciBar(ci, unit) {
  if (!ci || ci.lo == null) return '<span class="muted small">—</span>';
  const span = Math.max(1, Math.abs(ci.lo), Math.abs(ci.hi)), X = (v) => 50 + v / span * 46;
  const col = ci.lo > 0 ? 'var(--good)' : ci.hi < 0 ? 'var(--crit)' : 'var(--warn)';
  return `<div class="cibar" title="95%: ${f2(ci.lo)} a ${f2(ci.hi)} ${unit}"><i class="zero"></i><b style="left:${X(ci.lo)}%;width:${X(ci.hi) - X(ci.lo)}%;background:${col}"></b><em style="left:${X(ci.mean)}%"></em></div>
    <div class="small"><b>${ci.mean >= 0 ? '+' : ''}${f2(ci.mean)}</b> ${unit} <span class="muted">(95%: ${f2(ci.lo)} a ${f2(ci.hi)})</span></div>`;
}
function variantCard(label, s, name) {
  if (!s) return '';
  return `<div class="panel lab-var"><div class="row" style="justify-content:space-between"><h3 style="margin:0">${label === 'A' ? 'A · referência' : label === 'B' ? 'B · nova' : esc(label)}</h3><span class="muted small">${esc(name || '')}</span></div>
    <div class="kpis">
      <div><span>Jogos</span><b>${s.n}</b></div><div><span>V-E-D</span><b>${s.w}-${s.d}-${s.l}</b></div>
      <div><span>Vitórias</span><b>${pct(s.win && s.win.p)}</b><em>${s.win ? pct(s.win.lo) + '–' + pct(s.win.hi) : ''}</em></div>
      <div><span>Pontos/jogo</span><b>${f2(s.pts)}</b></div>
      <div><span>Golos</span><b>${f2(s.gf)} : ${f2(s.ga)}</b></div><div><span>xG</span><b>${f2(s.xg)} : ${f2(s.xga)}</b></div>
      <div><span>Remates</span><b>${s.shots == null ? '—' : s.shots.toFixed(1)} : ${s.shotsA == null ? '—' : s.shotsA.toFixed(1)}</b></div><div><span>Posse</span><b>${s.poss == null ? '—' : s.poss.toFixed(0) + '%'}</b></div>
    </div></div>`;
}
function vLabResults(job) {
  const a = analyzeJob(job), L = labState(), running = ['queued', 'running'].includes(job.status);
  const groups = Object.keys(a.per);
  const free = !job.experiment_type || job.experiment_type === 'free';
  return `<div class="panel"><div class="row" style="justify-content:space-between;align-items:flex-start">
      <div><h2 style="margin:0">${esc(a.name)}</h2><p class="small muted" style="margin:.2em 0 0">${a.type === 'save_opponents' ? 'Contra adversários reais do save' : a.mode === 'comparison' ? 'Comparação A/B com as mesmas sementes' : 'Tática única'} · ${a.completed || 0}/${a.total || a.rows.length} jogos · estado: ${esc(job.status || 'importado')}</p></div>
      <div class="row">${DESKTOP && L.status ? `${running ? `<button class="btn" onclick="A.labCancel('${esc(job.id)}')">Cancelar</button>` : ''}
        ${free && !running && job.id ? `${selectHTML([[50, '+50 jogos'], [100, '+100 jogos'], [200, '+200 jogos']], L.more || 100, "A.labSet('more',+this.value)")}<button class="btn primary" data-busy="lab-more" onclick="A.labMore('${esc(job.id)}')">Correr mais</button>` : ''}
        ${job.id ? groups.filter(g => g === 'A' || g === 'B').map(g => `<button class="btn" data-busy="lab-fmf" onclick="A.labFmf('${esc(job.id)}','${g}')">.fmf ${g}</button>`).join('') : ''}` : ''}</div></div>
    ${running ? `<div class="progress"><i style="width:${(job.completed || 0) / Math.max(1, job.total || 1) * 100}%"></i></div><p class="small muted">A correr no motor do FM27… ${job.matches_per_minute ? job.matches_per_minute + ' jogos/min' : ''}${job.eta_seconds ? ' · faltam ~' + Math.ceil(job.eta_seconds / 60) + ' min' : ''}</p>` : ''}
  </div>
  ${a.comparison ? `<div class="panel verdict ${a.comparison.tone}"><h3 style="margin:0 0 6px">Veredicto (${a.comparison.n} pares de jogos com a mesma semente)</h3><p style="margin:0 0 10px">${esc(a.comparison.verdict)}</p>
      <div class="grid g3"><div><span class="small muted">Saldo de xG por jogo (B − A)</span>${ciBar(a.comparison.dXG, 'xG')}</div><div><span class="small muted">Saldo de golos por jogo (B − A)</span>${ciBar(a.comparison.dGD, 'golos')}</div><div><span class="small muted">Pontos por jogo (B − A)</span>${ciBar(a.comparison.dPts, 'pts')}</div></div>
      ${a.comparison.needN && a.comparison.tone === 'warn' ? `<p class="small" style="margin:10px 0 0">Para separar uma diferença deste tamanho são precisos ~<b>${a.comparison.needN}</b> pares de jogos${a.comparison.needN > a.comparison.n ? ` (faltam ${a.comparison.needN - a.comparison.n})` : ''}.</p>` : ''}</div>` : ''}
  <div class="grid g2 lab-vars">${groups.map(g => variantCard(g, a.per[g], (a.names && a.names[g]) || '')).join('')}</div>
  ${a.byOpp && a.byOpp.length ? `<div class="panel"><h3>Por adversário</h3><div class="table-wrap" style="max-height:none"><table><thead><tr><th>Adversário</th>${groups.map(g => `<th>${g}: pts/jogo</th><th>${g}: xG saldo</th>`).join('')}</tr></thead><tbody>
      ${a.byOpp.map(o => `<tr><td>${esc(o.name)}</td>${groups.map(g => `<td>${o[g] ? f2(o[g].pts) : '—'}</td><td>${o[g] && o[g].xgd ? f2(o[g].xgd.mean) : '—'}</td>`).join('')}</tr>`).join('')}</tbody></table></div></div>` : ''}`;
}
function vLab() {
  const L = labState();
  const howto = `<ol class="clean small"><li>Abra o <b>FM27 Tactics Lab</b> (o Steam tem de estar aberto com a conta do FM27).</li><li>Lá, carregue o save ou monte as equipas, defina a tática <b>A</b> (referência) e <b>B</b> (alteração) e corra 30–100 jogos com “comparar”.</li><li>Aqui aparecem os resultados com intervalo de confiança e veredicto; pode correr mais jogos ou exportar o .fmf da vencedora.</li></ol>`;
  const left = DESKTOP ? `<div class="panel grid">
      <h3>Ligação</h3>
      <div class="row"><span class="dot ${L.status ? 'ok' : ''}"></span><b>${L.status ? 'Ligado ao Tactics Lab' : 'Desligado'}</b>${L.status ? `<span class="small muted">motor ${esc(L.status.engine || '')} · ${L.status.roles || '?'} papéis</span>` : ''}</div>
      ${L.err ? `<div class="notice warn small">${esc(L.err)}</div>` : ''}
      <div class="row"><label class="f" style="flex:1">Porta<input type="number" value="${L.port}" min="1024" max="65535" onchange="A.labSet('port',+this.value)"></label><button class="btn primary" data-busy="lab-connect" onclick="A.labConnect()">${L.status ? 'Actualizar' : 'Ligar'}</button></div>
      ${L.status ? `<button class="btn sm" data-busy="lab-cat" onclick="A.labCatalog()">Exportar catálogo do motor (diagnóstico)</button>` : ''}
    </div>
    <div class="panel"><div class="row" style="justify-content:space-between"><h3 style="margin:0">Experiências</h3>${L.status ? '<button class="btn sm" onclick="A.labJobs()">↻</button>' : ''}</div>
      ${L.jobs.length ? L.jobs.map(j => { const st = { complete: 'concluída', running: 'a correr', queued: 'em fila', error: 'erro', cancelled: 'cancelada' }[j.status] || j.status;
        return `<button class="jobitem ${L.job && L.job.id === j.id ? 'on' : ''}" onclick="A.labOpen('${esc(j.id)}')"><b>${esc(j.name || j.id)}</b><span class="st ${esc(j.status)}">${esc(st)}</span><span class="small">${j.completed || 0}/${j.total || 0} jogos</span><div class="small muted">${j.created ? new Date(j.created * 1000).toLocaleString('pt-PT') : ''}</div></button>`; }).join('') : `<p class="small muted">${L.status ? 'Ainda não há experiências no Tactics Lab.' : 'Ligue-se para ver as experiências.'}</p>`}
    </div>` : `<div class="panel"><h3>Ligação directa</h3><p class="small">Na <b>app instalada</b> (Windows) a app liga-se ao Tactics Lab, lista as experiências, corre mais jogos e exporta .fmf. No browser, importe o JSON exportado de uma experiência.</p></div>`;
  return `<div class="top"><div><h2>Laboratório — motor do FM27</h2><p class="muted small" style="margin:0">Testa táticas com o motor de jogo original através do <a href="https://github.com/lqf0624/fm-tactics-lab" target="_blank" rel="noopener">FM Tactics Lab</a> (MIT) e decide com estatística, não com palpites.</p></div>
    <label class="btn">Importar experiência (.json)<input type="file" accept=".json" hidden onchange="A.labImport(this)"></label></div>
  <div class="grid" style="grid-template-columns:minmax(280px,340px) 1fr;gap:16px;align-items:start">
    <div class="grid">${left}<div class="panel"><h3>Como usar</h3>${howto}</div></div>
    <div class="grid">${L.job ? vLabResults(L.job) : `<div class="panel empty"><h3>Sem experiência aberta</h3><p>${DESKTOP ? 'Ligue-se ao Tactics Lab e escolha uma experiência à esquerda' : 'Importe o JSON de uma experiência'}.</p></div>`}</div>
  </div>`;
}
