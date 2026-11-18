// ============================================================
// Tabela do plantel: papéis · habilidade e contrato · personalidade e ocultos
// ============================================================
const SQ_PERS = () => [['professionalism', L('Profissionalismo', 'Professionalism'), 'pers'], ['ambition', L('Ambição', 'Ambition'), 'pers'], ['pressure', L('Pressão', 'Pressure'), 'pers'],
  ['loyalty', L('Lealdade', 'Loyalty'), 'pers'], ['temperament', L('Temperamento', 'Temperament'), 'pers'], ['adaptability', L('Adaptabilidade', 'Adaptability'), 'pers'],
  ['cons', L('Consistência', 'Consistency'), 'hid'], ['imp', L('Jogos importantes', 'Important matches'), 'hid'], ['inj', L('Propensão a lesões', 'Injury proneness'), 'hid', true], ['vers', L('Versatilidade', 'Versatility'), 'hid']];
function sqVal(p, k) {
  if (k === 'name') return p.name; if (k === 'age') return p.age; if (k === 'ca') return p.ca; if (k === 'pa') return p.pa; if (k === 'value') return p.value; if (k === 'wage') return p.wage; if (k === 'end') return p.end || null;
  if (k === 'lk') return p.age != null && p.age <= 23 ? reachLikelihood(p) : null; if (k === 'rep') return p.rep;
  const d = SQ_PERS().find(x => x[0] === k); if (d) { const v = (p[d[2]] || {})[k]; return v == null ? null : v; }
  return null;
}
function squadTable(ps) {
  const mode = ST.sqMode || 'roles', S = ST.sqSort || { k: '', dir: 'desc' };
  const rich = ps.some(p => p.ca != null), richP = ps.some(p => p.pers && Object.keys(p.pers).length);
  const list = S.k ? [...ps].sort((a, b) => { const x = sqVal(a, S.k), y = sqVal(b, S.k); if (x == null && y == null) return 0; if (x == null) return 1; if (y == null) return -1; return (x > y ? 1 : x < y ? -1 : 0) * (S.dir === 'asc' ? 1 : -1); }) : ps;
  const th = (k, label) => `<th><button class="th-sort ${S.k === k ? 'on' : ''}" onclick="A.sqSort('${k}')">${label}${S.k === k ? (S.dir === 'asc' ? ' ▲' : ' ▼') : ''}</button></th>`;
  const badge = (v, inv) => v == null ? '<span class="muted">—</span>' : `<b class="v v${inv ? (v <= 6 ? 4 : v <= 10 ? 3 : v <= 14 ? 2 : 1) : (v >= 16 ? 4 : v >= 13 ? 3 : v >= 10 ? 2 : 1)}">${v}</b>`;
  const tabs = `<div class="seg" role="tablist" aria-label="${L('Colunas', 'Columns')}">${[['roles', L('Papéis', 'Roles')], ['ability', L('Habilidade e contrato', 'Ability and contract')], ['pers', L('Personalidade e ocultos', 'Personality and hidden')]].map(([k, n]) => `<button class="${mode === k ? 'on' : ''}" onclick="A.sqMode('${k}')" aria-pressed="${mode === k}">${n}</button>`).join('')}</div>`;
  const note = (mode === 'ability' && !rich) || (mode === 'pers' && !richP) ? `<div class="notice small" style="margin:8px 0">${L('Estes dados vêm do save: leia o plantel com “Ler o save agora” (fmsave) para ver CA/PA, personalidade, atributos ocultos, contrato e valor. O F9 e outros CSV só trazem o que a vista exportar.', 'This data comes from the save: read the squad with “Read the save now” (fmsave) to see CA/PA, personality, hidden attributes, contract and value. F9 and other CSVs only include what the view exports.')}</div>` : '';
  let head, row;
  if (mode === 'ability') {
    head = `${th('ca', 'CA')}${th('pa', 'PA')}${th('lk', L('Prob. PA', 'PA chance'))}${th('value', L('Valor', 'Value'))}${th('wage', L('Salário', 'Wage'))}${th('end', L('Contrato até', 'Contract until'))}<th>${L('Estatuto', 'Status')}</th>`;
    row = (p) => `<td><b>${p.ca ?? '—'}</b></td><td>${p.pa ?? '—'}${p.paRange ? '*' : ''}</td><td>${sqVal(p, 'lk') ?? '<span class="muted">—</span>'}</td><td>${fmtMoney(p.value)}</td><td>${fmtMoney(p.wage)}</td><td class="small">${fmtDate(p.end)}</td><td class="small">${esc(p.status || '—')}</td>`;
  } else if (mode === 'pers') {
    head = SQ_PERS().map(([k, n]) => th(k, n)).join('');
    row = (p) => SQ_PERS().map(([k, , g, inv]) => `<td>${badge((p[g] || {})[k], inv)}</td>`).join('');
  } else {
    head = `${rich ? th('ca', 'CA') : ''}<th>${L('Melhores papéis com bola', 'Best in-possession roles')}</th><th>${L('Melhores papéis sem bola', 'Best out-of-possession roles')}</th>`;
    row = (p) => {
      const groups = p.pos.length ? [...new Set(p.pos.filter(x => !x.includes(':')))] : Object.keys(GROUPS);
      const best = (phase) => groups.flatMap(g => rolesFor(g, phase, g === 'DC')).filter((r, i, a) => a.indexOf(r) === i).map(r => ({ r, s: (roleScore(p, r) || { score: 0 }).score })).sort((x, y) => y.s - x.s).slice(0, 3);
      return `${rich ? `<td><b>${p.ca ?? '—'}</b></td>` : ''}<td>${best('IP').map(x => `${fitBadge(x.s)} <span class="small">${esc(roleName(x.r))}</span>`).join('<br>')}</td><td>${best('OOP').map(x => `${fitBadge(x.s)} <span class="small">${esc(roleName(x.r))}</span>`).join('<br>')}</td>`;
    };
  }
  return `<div class="panel" style="margin-top:16px"><div class="row" style="justify-content:space-between;flex-wrap:wrap;gap:8px"><h3 style="margin:0">${L('Jogadores do plantel', 'Squad players')}</h3>${tabs}</div>
    ${note}<p class="small muted" style="margin:6px 0">${L('Clique num jogador para ver a ficha completa; clique no título de uma coluna para ordenar.', 'Click a player for the full card; click a column title to sort.')}</p>
    <div class="table-wrap" style="max-height:none"><table class="scout-table"><thead><tr>${th('name', L('Jogador', 'Player'))}<th>${L('Pos', 'Pos')}</th>${th('age', L('Idade', 'Age'))}${head}</tr></thead><tbody>
    ${list.map(p => `<tr class="click" onclick="A.squadOpen('${esc(p.id)}')"><td><b>${esc(p.name)}</b></td><td class="small">${esc(p.posRaw || '—')}</td><td>${p.age ?? '—'}</td>${row(p)}</tr>`).join('')}
    </tbody></table></div></div>`;
}
