// ============================================================
// Prospeção (scouting) — todos os jogadores do save
// ============================================================
const SCOUT = { db: null, nations: {}, date: null, save: '', ms: 0, cached: false, cache: null, cacheKey: '' };
const PAGE = 50;
function scoutState() { if (!ST.scout) ST.scout = { F: SCOUT_DEFAULT(), preset: '', page: 0, tab: 'results', open: null }; if (!ST.scout.F) ST.scout.F = SCOUT_DEFAULT(); return ST.scout; }
const fmtMoney = (v) => v == null ? '—' : new Intl.NumberFormat(LOCALE(), { notation: 'compact', maximumFractionDigits: 1 }).format(v);
const fmtDate = (d) => d ? new Date(d).toLocaleDateString(LOCALE(), { year: 'numeric', month: 'short' }) : '—';
function pctile(list, q) { const v = list.filter(x => x != null).sort((a, b) => a - b); return v.length ? v[Math.min(v.length - 1, Math.floor(v.length * q))] : null; }
function scoutCtx() {
  const squad = players(), pm = Object.fromEntries(squad.map(p => [p.id, p]));
  const xi = ST.tactic.slots.map(s => pm[s.pid]).filter(Boolean);
  const cas = xi.map(p => +(p.meta && p.meta.ca) || null).filter(Boolean);
  const xiCA = cas.length ? Math.round(cas.reduce((a, b) => a + b, 0) / cas.length) : 120;
  const db = SCOUT.db;
  if (db && !SCOUT._val) SCOUT._val = { cheapValue: pctile(db.players.filter(p => p.age <= 19 && p.pa >= 150).map(p => p.value), 0.35) || 3000000 };
  const bargainValue = db ? pctile(db.players.filter(p => p.age >= 21 && p.age <= 29 && p.ca >= xiCA - 5).map(p => p.value), 0.3) : null;
  // reputação do meu clube: lida na própria base de dados, nos jogadores do meu clube
  let myRep = null;
  if (db && ST.myClub) { if (SCOUT._myRepFor !== ST.myClub) { const mc = ST.myClub.toLowerCase(); const r = db.players.filter(p => p.club && p.club.toLowerCase() === mc && p.clubRep != null).map(p => p.clubRep); SCOUT._myRep = r.length ? r.sort((a, b) => a - b)[Math.floor(r.length / 2)] : null; SCOUT._myRepFor = ST.myClub; } myRep = SCOUT._myRep; }
  return { date: SCOUT.date, myClub: ST.myClub || '', tactic: ST.tactic, squad, xiCA, cheapValue: SCOUT._val ? SCOUT._val.cheapValue : null, bargainValue, myRep };
}
// casa mais fraca do meu XI (para a pesquisa "A minha posição mais fraca")
function weakestSlot() {
  const pm = Object.fromEntries(players().map(p => [p.id, p])); let w = null;
  ST.tactic.slots.forEach(s => { const p = pm[s.pid]; const sc = p ? slotScore(p, ST.tactic, s) : 0; const a = slotInfo(ST.tactic.ipForm, s.ipCode); if (a.group !== 'GK' && (!w || sc < w.sc)) w = { s, sc, group: a.group }; });
  return w;
}
function scoutResults() {
  const S = scoutState(), key = JSON.stringify(S.F) + '|' + (SCOUT.db ? SCOUT.db.players.length : 0) + '|' + ST.tactic.slots.map(s => s.pid + s.ip).join();
  if (SCOUT.cacheKey === key && SCOUT.cache) return SCOUT.cache;
  SCOUT.cache = scoutFilter(SCOUT.db, S.F, scoutCtx()); SCOUT.cacheKey = key; return SCOUT.cache;
}
const GROUP_OPTS = () => [['', L('Todas as posições', 'All positions')], ...Object.entries(GROUPS).map(([k, v]) => [k, v])];
function scoutFilters() {
  const F = scoutState().F, nat = Object.entries(SCOUT.nations).sort((a, b) => a[1].localeCompare(b[1]));
  const inp = (k, label, ph, type = 'number', step = '') => `<label class="f">${label}<input type="${type}" ${step ? `step="${step}"` : ''} value="${esc(F[k] ?? '')}" placeholder="${esc(ph || '')}" onchange="A.scoutF('${k}', this.value)"></label>`;
  const pair = (a, b, label) => `<div class="f"><span>${label}</span><div class="pair">${a}${b}</div></div>`;
  const mini = (k, ph) => `<input type="number" value="${esc(F[k] ?? '')}" placeholder="${ph}" onchange="A.scoutF('${k}', this.value)" aria-label="${ph}">`;
  const sel = (k, opts, label) => `<label class="f">${label}${selectHTML(opts, F[k] ?? '', `A.scoutF('${k}', this.value)`)}</label>`;
  const chk = (k, label) => `<label class="row small"><input type="checkbox" ${F[k] ? 'checked' : ''} onchange="A.scoutF('${k}', this.checked)"> ${label}</label>`;
  const attrOpts = [['', '—'], ...Object.keys(ATTRS).map(k => [k, attrName(k)]).sort((a, b) => a[1].localeCompare(b[1]))];
  const roleOpts = [['', L('— nenhum —', '— none —')], ...ROLES.filter(r => r.phase === 'IP').map(r => [r.id, `${roleName(r)} (${r.ab})`]).sort((a, b) => a[1].localeCompare(b[1]))];
  return `<div class="panel scout-filters">
    <div class="row" style="justify-content:space-between"><h3 style="margin:0">${L('Filtros', 'Filters')}</h3><button class="btn sm" onclick="A.scoutReset()">${L('Limpar', 'Clear')}</button></div>
    ${inp('q', L('Nome', 'Name'), L('procurar…', 'search…'), 'text')}
    <details open><summary>${L('Jogador', 'Player')}</summary>
      ${pair(mini('ageMin', L('mín', 'min')), mini('ageMax', L('máx', 'max')), L('Idade', 'Age'))}
      ${pair(mini('caMin', L('mín', 'min')), mini('caMax', L('máx', 'max')), L('Habilidade actual (CA)', 'Current ability (CA)'))}
      ${pair(mini('paMin', L('mín', 'min')), mini('paMax', L('máx', 'max')), L('Potencial (PA)', 'Potential (PA)'))}
      ${sel('pos', GROUP_OPTS(), L('Posição', 'Position'))}
      ${sel('posLevel', [['acc', L('competente ou natural (≥ 15)', 'accomplished or natural (≥ 15)')], ['nat', L('só natural (≥ 18)', 'natural only (≥ 18)')]], L('Nível na posição', 'Position level'))}
      ${sel('foot', [['', L('qualquer', 'any')], ['L', L('esquerdo forte', 'strong left')], ['R', L('direito forte', 'strong right')], ['B', L('ambos os pés', 'both feet')]], L('Pé', 'Foot'))}
      ${inp('heightMin', L('Altura mínima (cm)', 'Minimum height (cm)'), '185')}
    </details>
    <details open><summary>${L('Clube, contrato e valor', 'Club, contract and value')}</summary>
      ${inp('club', L('Clube', 'Club'), L('nome do clube', 'club name'), 'text')}
      ${nat.length ? sel('league', [['', L('qualquer', 'any')], ...nat.map(([k, v]) => [k, v])], L('País da liga do clube', 'Club’s league country')) + sel('nation', [['', L('qualquer', 'any')], ...nat.map(([k, v]) => [k, v])], L('Nacionalidade', 'Nationality')) : ''}
      ${inp('valueMax', L('Valor máximo (moeda do save)', 'Maximum value (save currency)'), '5000000')}
      ${inp('wageMax', L('Salário semanal máximo', 'Maximum weekly wage'), '20000')}
      ${inp('endMonths', L('Contrato acaba em até (meses)', 'Contract ends within (months)'), '6')}
      ${inp('clubRepMax', L('Reputação máxima do clube (1–10000)', 'Maximum club reputation (1–10000)'), '4000')}
      ${scoutCtx().myRep != null ? inp('joinMin', L('Probabilidade de querer vir (estimativa, 0–100)', 'Chance he would join you (estimate, 0–100)'), '70') : `<p class="small muted">${L('Para estimar quem quer vir, carregue o seu plantel (a app usa a reputação do seu clube).', 'To estimate who would join, load your squad (the app uses your club’s reputation).')}</p>`}
      ${chk('free', L('Só jogadores livres', 'Free agents only'))}
      ${chk('excludeMine', L('Excluir o meu clube', 'Exclude my club'))}
      ${sel('loan', [['any', L('incluir emprestados', 'include loaned players')], ['exclude', L('excluir emprestados', 'exclude loaned players')], ['only', L('só emprestados', 'loaned players only')]], L('Empréstimos', 'Loans'))}
    </details>
    <details><summary>${L('Personalidade e atributos ocultos', 'Personality and hidden attributes')}</summary>
      ${inp('profMin', L('Profissionalismo mínimo', 'Minimum professionalism'), '15')}
      ${inp('ambMin', L('Ambição mínima', 'Minimum ambition'), '14')}
      ${inp('detMin', L('Determinação mínima', 'Minimum determination'), '15')}
      ${inp('consMin', L('Consistência mínima', 'Minimum consistency'), '14')}
      ${inp('impMin', L('Jogos importantes mínimo', 'Minimum big-match temperament'), '14')}
      ${inp('injMax', L('Propensão a lesões máxima', 'Maximum injury proneness'), '8')}
      ${inp('likelyMin', L('Probabilidade de atingir o potencial (0–100)', 'Chance of reaching potential (0–100)'), '70')}
    </details>
    <details ${F.attrs.length ? 'open' : ''}><summary>${L('Atributos mínimos', 'Minimum attributes')} ${F.attrs.length ? `(${F.attrs.length})` : ''}</summary>
      ${F.attrs.map((x, i) => `<div class="pair attr-row">${selectHTML(attrOpts, x.k, `A.scoutAttr(${i}, 'k', this.value)`)}<input type="number" min="1" max="20" value="${esc(x.min ?? '')}" onchange="A.scoutAttr(${i}, 'min', this.value)" aria-label="${L('mínimo', 'minimum')}"><button class="btn sm" onclick="A.scoutAttrDel(${i})" aria-label="${L('Remover', 'Remove')}">×</button></div>`).join('')}
      <button class="btn sm" onclick="A.scoutAttrAdd()">+ ${L('Atributo', 'Attribute')}</button>
    </details>
    <details ${F.role ? 'open' : ''}><summary>${L('Papel', 'Role')}</summary>
      ${sel('role', roleOpts, L('Adequação ao papel', 'Role fit'))}
      ${F.role ? inp('roleMin', L('Adequação mínima (0–100)', 'Minimum fit (0–100)'), '65') : ''}
    </details>
  </div>`;
}
function scoutTable(list, extra) {
  const S = scoutState(), F = S.F, stars = new Set((ST.shortlist || []).map(x => x.id));
  const th = (k, label) => `<th><button class="th-sort ${F.sort === k ? 'on' : ''}" onclick="A.scoutSort('${k}')">${label}${F.sort === k ? (F.dir === 'asc' ? ' ▲' : ' ▼') : ''}</button></th>`;
  const ext = extra === 'join' ? [L('Vir?', 'Join?'), p => p._jn ?? '—'] : extra === 'likely' ? [L('Prob.', 'Chance'), p => p._lk ?? reachLikelihood(p)] : extra === 'role' ? [L('Papel', 'Role'), p => p._rs] : extra === 'gain' ? [L('Ganho no XI', 'XI gain'), p => p._gain != null ? `${p._gain > 0 ? '+' : ''}${p._gain} <span class="muted small">${esc(p._gainSlot || '')}</span>` : '—'] : null;
  return `<div class="table-wrap" style="max-height:none"><table class="scout-table"><thead><tr><th aria-label="${L('Lista de observação', 'Shortlist')}">☆</th>${th('name', L('Nome', 'Name'))}${th('age', L('Idade', 'Age'))}<th>${L('Pos', 'Pos')}</th><th>${L('Clube', 'Club')}</th>${th('ca', 'CA')}${th('pa', 'PA')}${th('value', L('Valor', 'Value'))}${th('wage', L('Salário', 'Wage'))}${th('end', L('Contrato', 'Contract'))}${ext ? `<th>${ext[0]}</th>` : ''}</tr></thead><tbody>
    ${list.map(p => `<tr class="click" onclick="A.scoutOpen('${esc(p.id)}')"><td><button class="star ${stars.has(p.id) ? 'on' : ''}" onclick="event.stopPropagation();A.scoutStar('${esc(p.id)}')" aria-label="${stars.has(p.id) ? L('Tirar da lista', 'Remove from shortlist') : L('Pôr na lista', 'Add to shortlist')}">${stars.has(p.id) ? '★' : '☆'}</button></td>
      <td><b>${esc(p.name)}</b>${p.loan ? ` <span class="chip">${L('emprest.', 'loan')}</span>` : ''}</td><td>${p.age ?? '—'}</td><td class="small">${esc(p.posRaw || '')}</td><td class="small">${p.club ? esc(p.club) : `<span class="chip ok">${L('livre', 'free')}</span>`}</td>
      <td><b>${p.ca ?? '—'}</b></td><td>${p.pa ?? '—'}${p.paRange ? '*' : ''}</td><td>${p.value == null && p.club ? `<span class="muted" title="${esc(L('Valor desconhecido: o fmsave não o conseguiu ler deste jogador.', 'Unknown value: fmsave could not read it for this player.'))}">?</span>` : fmtMoney(p.value)}</td><td>${fmtMoney(p.wage)}</td><td class="small">${fmtDate(p.end)}</td>${ext ? `<td>${ext[1](p) ?? '—'}</td>` : ''}</tr>`).join('')}
  </tbody></table></div>`;
}
function scoutCard(p, opts = {}) {
  const own = !!opts.own, close = own ? 'A.squadClose()' : 'A.scoutClose()';
  const ctx = scoutCtx(), jn = own ? null : joinLikelihood(p, ctx), g = ctx.squad.length ? bestGain(p, ST.tactic, ctx.squad) : null, roles = bestRolesAny(p, 5), lk = p.age != null && p.age <= 23 ? reachLikelihood(p) : null;
  const grp = (c) => Object.keys(ATTRS).filter(k => ATTRS[k].c === c && p.a[k] != null);
  const cell = (k) => { const v = Math.round(p.a[k]); return `<div class="attr"><span>${esc(attrName(k))}</span><b class="v v${v >= 16 ? 4 : v >= 13 ? 3 : v >= 10 ? 2 : 1}">${v}</b></div>`; };
  const gkR = p.posR ? (p.posR.gk ?? 0) : 0, isGK = gkR >= 15 && gkR >= Math.max(0, ...Object.entries(p.posR || {}).filter(([k]) => k !== 'gk').map(([, v]) => v));   // só se a melhor posição for GR
  const pers = [['professionalism', L('Profissionalismo', 'Professionalism')], ['ambition', L('Ambição', 'Ambition')], ['pressure', L('Pressão', 'Pressure')], ['loyalty', L('Lealdade', 'Loyalty')], ['temperament', L('Temperamento', 'Temperament')], ['adaptability', L('Adaptabilidade', 'Adaptability')]];
  const hid = [['cons', L('Consistência', 'Consistency')], ['imp', L('Jogos importantes', 'Important matches')], ['inj', L('Propensão a lesões', 'Injury proneness')], ['vers', L('Versatilidade', 'Versatility')]];
  const mySlot = own ? ST.tactic.slots.find(s => s.pid === p.id) : null;
  const ownNote = own ? (mySlot ? L(`Titular em <b>${esc(mySlot.ipCode)}</b> na tática actual (adequação ${slotScore(p, ST.tactic, mySlot)}).`, `Starts at <b>${esc(mySlot.ipCode)}</b> in the current tactic (fit ${slotScore(p, ST.tactic, mySlot)}).`)
    : g ? L(`Não está no XI. Melhor casa: <b>${esc(g.code)}</b> (${g.his} contra ${g.cur} de ${esc(g.starter || 'ninguém')}).`, `Not in the XI. Best slot: <b>${esc(g.code)}</b> (${g.his} vs ${g.cur} for ${esc(g.starter || 'nobody')}).`) : '') : '';
  return `<div class="modal-back" onclick="if(event.target===this)${close}"><div class="modal" role="dialog" aria-modal="true" aria-label="${esc(p.name)}">
    <div class="row" style="justify-content:space-between;align-items:flex-start"><div><h2 style="margin:0">${esc(p.name)}</h2>
      <p class="small muted" style="margin:.2em 0 0">${p.age ?? '—'} ${L('anos', 'years')} · ${p.club ? esc(p.club) : L('sem clube', 'no club')}${SCOUT.nations[p.nat] ? ' · ' + esc(SCOUT.nations[p.nat]) : ''} · ${esc(p.posRaw || '')}${p.height ? ' · ' + p.height + ' cm' : ''}${p.foot ? ' · ' + (p.foot === 'L' ? L('esquerdino', 'left-footed') : p.foot === 'R' ? L('destro', 'right-footed') : L('ambos os pés', 'two-footed')) : ''}</p></div>
      <div class="row">${own ? '' : `<button class="btn ${(ST.shortlist || []).some(x => x.id === p.id) ? 'primary' : ''}" onclick="A.scoutStar('${esc(p.id)}')">${(ST.shortlist || []).some(x => x.id === p.id) ? '★ ' + L('Na lista', 'On shortlist') : '☆ ' + L('Pôr na lista', 'Add to shortlist')}</button>`}<button class="btn" onclick="${close}" aria-label="${L('Fechar', 'Close')}">✕</button></div></div>
    <div class="kpis" style="margin-top:12px"><div><span>CA</span><b>${p.ca ?? '—'}</b></div><div><span>PA</span><b>${p.pa ?? '—'}${p.paRange ? ` <span class="small muted">(${p.paRange[0]}–${p.paRange[1]})</span>` : ''}</b></div><div><span>${L('Valor', 'Value')}</span><b>${p.value == null && p.club ? L('desconhecido', 'unknown') : fmtMoney(p.value)}</b></div><div><span>${L('Salário/semana', 'Wage/week')}</span><b>${fmtMoney(p.wage)}</b></div><div><span>${L('Contrato até', 'Contract until')}</span><b>${fmtDate(p.end)}</b></div>${lk != null ? `<div><span>${L('Prob. de atingir o PA', 'Chance to reach PA')}</span><b>${lk}</b></div>` : ''}</div>
    ${jn && !(ctx.myClub && p.club && p.club.toLowerCase() === ctx.myClub.toLowerCase()) ? `<div class="notice small" style="margin-top:10px"><b>${L('Probabilidade de querer vir', 'Chance he would join you')}: ${jn.score}/100</b> <span class="muted">(${L('estimativa', 'estimate')})</span> — ${esc(jn.why.join(' · '))}</div>` : ''}
    ${own ? (ownNote ? `<div class="notice small" style="margin-top:10px">${ownNote}</div>` : '') : g ? `<div class="notice small" style="margin-top:10px">${g.gain > 0 ? L(`No teu XI entraria em <b>${esc(g.code)}</b>: adequação ${g.his} contra ${g.cur} de ${esc(g.starter || 'ninguém')} (<b>+${g.gain}</b>).`, `In your XI he would play at <b>${esc(g.code)}</b>: fit ${g.his} vs ${g.cur} for ${esc(g.starter || 'nobody')} (<b>+${g.gain}</b>).`) : L(`Não melhoraria o teu XI (melhor casa: ${esc(g.code)}, ${g.his} contra ${g.cur}).`, `He would not improve your XI (best slot: ${esc(g.code)}, ${g.his} vs ${g.cur}).`)}</div>` : ''}
    <div class="grid g3 attrs-grid" style="margin-top:12px">${(isGK ? ['G', 'M', 'F'] : ['T', 'M', 'F']).map(c => `<div><h4>${{ T: L('Técnicos', 'Technical'), M: L('Mentais', 'Mental'), F: L('Físicos', 'Physical'), G: L('Guarda-redes', 'Goalkeeping') }[c]}</h4>${grp(c).map(cell).join('')}</div>`).join('')}</div>
    <div class="grid g2" style="margin-top:12px;align-items:start">
      <div><h4>${L('Melhores papéis', 'Best roles')}</h4>${roles.map(x => `<div class="row small" style="justify-content:space-between"><span>${esc(roleName(x.r))} <span class="muted">(<span>${esc(GROUPS[x.g] || x.g)}</span>)</span></span>${fitBadge(x.s)}</div>`).join('')}</div>
      <div><h4>${L('Personalidade e ocultos', 'Personality and hidden')}</h4>${pers.filter(([k]) => p.pers[k] != null).map(([k, n]) => `<div class="row small" style="justify-content:space-between"><span>${n}</span><b>${p.pers[k]}</b></div>`).join('')}${hid.filter(([k]) => p.hid[k] != null).map(([k, n]) => `<div class="row small" style="justify-content:space-between"><span>${n}</span><b>${p.hid[k]}</b></div>`).join('')}</div>
    </div>
    <p class="small muted" style="margin-bottom:0">${L('Dados lidos do save pelo fmsave. Alguns campos (personalidade, valor, salário) estão marcados pelo fmsave como ainda não confirmados no jogo.', 'Data read from the save by fmsave. Some fields (personality, value, wage) are flagged by fmsave as not yet confirmed against the game.')}</p>
  </div></div>`;
}
function vScout() {
  const S = scoutState();
  if (!SCOUT.db) {
    return `<div class="top"><div><h2>${L('Prospeção', 'Scouting')}</h2><p class="muted small" style="margin:0">${L('Todos os jogadores do teu save, com atributos, CA/PA, personalidade, contrato e valor.', 'Every player in your save, with attributes, CA/PA, personality, contract and value.')}</p></div></div>
      <div class="panel empty"><h3>${L('Carregar a base de dados do save', 'Load the save’s database')}</h3>
      ${DESKTOP ? `<p>${L('A app corre o fmsave sobre uma cópia do último save e lê todos os jogadores. A primeira leitura pode demorar 1–3 minutos; depois fica guardada até o save mudar.', 'The app runs fmsave on a copy of your latest save and reads every player. The first read can take 1–3 minutes; after that it is kept until the save changes.')}</p>
        <button class="btn primary" data-busy="scout" onclick="A.scoutLoad()">${L('Ler todos os jogadores do save', 'Read every player in the save')}</button>
        ${SCOUT.err ? `<div class="notice warn small" style="margin-top:10px">${esc(saveErrText(SCOUT.err))}</div>` : ''}`
      : `<p>${L('No browser: abra o', 'In the browser: open')} <a href="https://rhiever.github.io/fmsave/app/" target="_blank" rel="noopener">fmsave online</a>${L(', largue o save, descarregue o CSV com todos os jogadores e arraste-o para esta janela.', ', drop your save in, download the CSV with every player and drag it into this window.')}</p>`}
      ${(ST.shortlist || []).length ? `<p class="small">${L(`Tens ${ST.shortlist.length} jogador(es) na lista de observação.`, `You have ${ST.shortlist.length} player(s) on your shortlist.`)}</p>` : ''}</div>`;
  }
  const ctx = scoutCtx(), presets = scoutPresets(ctx);
  const res = scoutResults(), pages = Math.max(1, Math.ceil(res.length / PAGE)), page = Math.min(S.page || 0, pages - 1);
  const extra = S.F.sort === 'join' || S.F.joinMin ? 'join' : S.F.sort === 'likely' || S.F.likelyMin ? 'likely' : S.F.role ? 'role' : S.F.sort === 'gain' ? 'gain' : null;
  const open = S.open ? SCOUT.db.players.find(p => p.id === S.open) : null;
  const list = (ST.shortlist || []).map(x => SCOUT.db.players.find(p => p.id === x.id) || x);
  return `<div class="top"><div><h2>${L('Prospeção', 'Scouting')}</h2><p class="muted small" style="margin:0">${L(`${SCOUT.db.players.length.toLocaleString(LOCALE())} jogadores · ${esc(SCOUT.save)} · data do jogo ${SCOUT.date ? fmtDate(SCOUT.date) : '—'}${SCOUT.dateGuess ? ' (estimada)' : ''} · ${SCOUT.cached ? 'da cache' : 'lido em ' + (SCOUT.ms / 1000).toFixed(1) + ' s'}`, `${SCOUT.db.players.length.toLocaleString(LOCALE())} players · ${esc(SCOUT.save)} · game date ${SCOUT.date ? fmtDate(SCOUT.date) : '—'}${SCOUT.dateGuess ? ' (estimated)' : ''} · ${SCOUT.cached ? 'from cache' : 'read in ' + (SCOUT.ms / 1000).toFixed(1) + ' s'}`)}</p></div>
    <div class="row">${DESKTOP ? `<button class="btn" data-busy="scout" onclick="A.scoutLoad(true)">${L('Actualizar do save', 'Refresh from save')}</button>` : ''}<button class="btn" onclick="A.scoutExport()">${L('Exportar CSV', 'Export CSV')}</button></div></div>
  <div class="plan-tabs" role="tablist" aria-label="${L('Pesquisas pré-definidas', 'Preset searches')}">${presets.map(p => `<button class="${S.preset === p.k ? 'on' : ''}" onclick="A.scoutPreset('${p.k}')" title="${esc(p.d)}" ${(p.needSquad && !ctx.squad.length) || (p.needMyClub && ctx.myRep == null) ? `disabled title="${esc(L('Carregue primeiro o seu plantel.', 'Load your squad first.'))}"` : ''}><span aria-hidden="true">${p.ic}</span> ${esc(p.n)}</button>`).join('')}</div>
  ${S.preset ? `<p class="small muted" style="margin:8px 0 0">${esc((presets.find(p => p.k === S.preset) || {}).d || '')}${S.weak ? ' ' + L(`Posição: ${S.weak}.`, `Position: ${S.weak}.`) : ''} ${L('Pode afinar com os filtros.', 'You can refine it with the filters.')}</p>` : ''}
  <div class="grid scout-grid" style="margin-top:12px">
    ${scoutFilters()}
    <div class="panel" style="min-width:0">
      <div class="row" style="justify-content:space-between;flex-wrap:wrap"><div class="seg" role="tablist"><button class="${S.tab !== 'list' ? 'on' : ''}" onclick="A.scoutTab('results')">${L('Resultados', 'Results')} (${res.length.toLocaleString(LOCALE())})</button><button class="${S.tab === 'list' ? 'on' : ''}" onclick="A.scoutTab('list')">★ ${L('Lista de observação', 'Shortlist')} (${list.length})</button></div>
        ${S.tab !== 'list' && pages > 1 ? `<div class="row small"><button class="btn sm" ${page <= 0 ? 'disabled' : ''} onclick="A.scoutPage(${page - 1})" aria-label="${L('Anterior', 'Previous')}">‹</button><span>${page + 1} / ${pages}</span><button class="btn sm" ${page >= pages - 1 ? 'disabled' : ''} onclick="A.scoutPage(${page + 1})" aria-label="${L('Seguinte', 'Next')}">›</button></div>` : ''}</div>
      ${S.tab === 'list' ? (list.length ? scoutTable(list, null) : `<p class="small muted">${L('Ainda não há ninguém na lista: use ☆ nos resultados.', 'Nobody on the list yet: use ☆ in the results.')}</p>`)
        : res.length ? scoutTable(res.slice(page * PAGE, page * PAGE + PAGE), extra) : `<p class="small muted">${L('Nenhum jogador com estes filtros. Alargue a pesquisa.', 'No players match these filters. Widen the search.')}</p>`}
    </div>
  </div>
  ${open ? scoutCard(open) : ''}`;
}
