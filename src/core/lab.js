// ============================================================
// Laboratório: análise das experiências do FM Tactics Lab
// (github.com/lqf0624/fm-tactics-lab, MIT). Formato lido do código
// do servidor (fm27_tactics_server.py):
//   job = { id, name, status, completed, total, mode: 'single'|'comparison',
//           experiment_type?: 'save_opponents'|'setpiece_episode',
//           schedule?: { variant: { group, tested_side, opponent, venue } },
//           metrics: { variant|group: { n, wins, draws, losses, win_rate, win_interval, goals_for, goals_against, points } },
//           runs: [ { variant, seed, result: { score: [casa, fora] },
//                     statistics: { teams: [ { xg, shots, possession, passes, completed_passes, ... }, {...} ] } } ] }
// No modo livre a equipa testada é a 0; 'A' = referência, 'B' = tática nova.
// ============================================================

const T95 = [0, 12.71, 4.30, 3.18, 2.78, 2.57, 2.45, 2.36, 2.31, 2.26, 2.23, 2.20, 2.18, 2.16, 2.14, 2.13, 2.12, 2.11, 2.10, 2.09, 2.09, 2.08, 2.07, 2.07, 2.06, 2.06, 2.06, 2.05, 2.05, 2.05, 2.04];
const tcrit = (df) => df <= 0 ? NaN : df < T95.length ? T95[df] : 1.96 + 2.4 / df;
function meanCI(xs) {
  const n = xs.length; if (!n) return null;
  const m = xs.reduce((a, b) => a + b, 0) / n;
  if (n < 2) return { n, mean: m, sd: null, lo: null, hi: null };
  const sd = Math.sqrt(xs.reduce((a, x) => a + (x - m) * (x - m), 0) / (n - 1));
  const h = tcrit(n - 1) * sd / Math.sqrt(n);
  return { n, mean: m, sd, lo: m - h, hi: m + h };
}
// Wilson para taxas (vitórias)
function wilson(k, n) {
  if (!n) return null; const z = 1.959963984540054, p = k / n, den = 1 + z * z / n;
  const c = (p + z * z / (2 * n)) / den, h = z * Math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / den;
  return { p, lo: Math.max(0, c - h), hi: Math.min(1, c + h) };
}

// orienta cada jogo para a equipa testada: devolve {gf, ga, xg, xga, shots, shotsA, poss, pass}
function orientRun(job, run) {
  const sched = job.schedule && job.schedule[run.variant];
  const side = sched && sched.tested_side != null ? sched.tested_side : 0;
  const sc = (run.result && run.result.score) || [0, 0];
  const st = (run.statistics && run.statistics.teams) || [];
  const me = st[side] || {}, op = st[1 - side] || {};
  const num = (v) => (typeof v === 'number' && isFinite(v) ? v : null);
  return {
    group: sched ? sched.group : run.variant, seed: run.seed, opponent: sched ? sched.opponent_key || sched.opponent : null, venue: sched ? sched.venue : null,
    gf: sc[side], ga: sc[1 - side], xg: num(me.xg), xga: num(op.xg), shots: num(me.shots), shotsA: num(op.shots),
    poss: num(me.possession), pass: me.passes ? (me.completed_passes || 0) / me.passes : null,
  };
}

function summarize(rows) {
  const n = rows.length; if (!n) return null;
  const avg = (k) => { const v = rows.map(r => r[k]).filter(x => x != null); return v.length ? v.reduce((a, b) => a + b, 0) / v.length : null; };
  const w = rows.filter(r => r.gf > r.ga).length, d = rows.filter(r => r.gf === r.ga).length;
  return { n, w, d, l: n - w - d, win: wilson(w, n), pts: (3 * w + d) / n, gf: avg('gf'), ga: avg('ga'), xg: avg('xg'), xga: avg('xga'),
    shots: avg('shots'), shotsA: avg('shotsA'), poss: avg('poss'), pass: avg('pass'),
    gd: meanCI(rows.map(r => r.gf - r.ga)), xgd: rows.every(r => r.xg != null && r.xga != null) ? meanCI(rows.map(r => r.xg - r.xga)) : null };
}

// emparelha A e B pela mesma semente (e mesmo adversário/campo, no modo save)
function pairedDiff(rows) {
  const key = (r) => [r.seed, r.opponent || '', r.venue || ''].join('|');
  const A = new Map(), B = new Map();
  rows.forEach(r => { if (r.group === 'A') A.set(key(r), r); else if (r.group === 'B') B.set(key(r), r); });
  const pairs = [...A.keys()].filter(k => B.has(k)).map(k => [A.get(k), B.get(k)]);
  if (!pairs.length) return null;
  const dGD = meanCI(pairs.map(([a, b]) => (b.gf - b.ga) - (a.gf - a.ga)));
  const hasXG = pairs.every(([a, b]) => a.xg != null && b.xg != null && a.xga != null && b.xga != null);
  const dXG = hasXG ? meanCI(pairs.map(([a, b]) => (b.xg - b.xga) - (a.xg - a.xga))) : null;
  const dPts = meanCI(pairs.map(([a, b]) => ((b.gf > b.ga ? 3 : b.gf === b.ga ? 1 : 0) - (a.gf > a.ga ? 3 : a.gf === a.ga ? 1 : 0))));
  // o xG tem muito menos ruído do que os golos: é a métrica principal do veredicto
  const main = dXG || dGD, unit = dXG ? 'xG' : L('golos', 'goals');
  let verdict, tone;
  if (!main || main.n < 6 || main.lo == null) { verdict = L('Poucos jogos para concluir. Corre pelo menos 30 por tática.', 'Too few matches to conclude. Run at least 30 per tactic.'); tone = 'info'; }
  else if (main.lo > 0) { verdict = L(`B é melhor do que A: +${main.mean.toFixed(2)} ${unit} de saldo por jogo (95%: ${main.lo.toFixed(2)} a ${main.hi.toFixed(2)}).`, `B is better than A: +${main.mean.toFixed(2)} ${unit} difference per match (95%: ${main.lo.toFixed(2)} to ${main.hi.toFixed(2)}).`); tone = 'good'; }
  else if (main.hi < 0) { verdict = L(`A é melhor do que B: ${main.mean.toFixed(2)} ${unit} de saldo por jogo com B (95%: ${main.lo.toFixed(2)} a ${main.hi.toFixed(2)}).`, `A is better than B: ${main.mean.toFixed(2)} ${unit} difference per match with B (95%: ${main.lo.toFixed(2)} to ${main.hi.toFixed(2)}).`); tone = 'crit'; }
  else { verdict = L(`Sem diferença clara (${main.mean >= 0 ? '+' : ''}${main.mean.toFixed(2)} ${unit}; 95%: ${main.lo.toFixed(2)} a ${main.hi.toFixed(2)}).`, `No clear difference (${main.mean >= 0 ? '+' : ''}${main.mean.toFixed(2)} ${unit}; 95%: ${main.lo.toFixed(2)} to ${main.hi.toFixed(2)}).`); tone = 'warn'; }
  // jogos necessários para separar uma diferença do tamanho observada (mín. 0,15) com 95%
  let needN = null;
  if (main && main.sd) { const target = Math.max(0.15, Math.abs(main.mean)); needN = Math.ceil(Math.pow(1.96 * main.sd / target, 2)); }
  return { n: pairs.length, dGD, dXG, dPts, verdict, tone, needN, unit };
}

function analyzeJob(job) {
  const runs = (job.runs || []).filter(r => r && r.result && Array.isArray(r.result.score));
  const rows = runs.map(r => orientRun(job, r));
  const groups = [...new Set(rows.map(r => r.group))].sort();
  const per = Object.fromEntries(groups.map(g => [g, summarize(rows.filter(r => r.group === g))]));
  const comparison = groups.includes('A') && groups.includes('B') ? pairedDiff(rows) : null;
  // por adversário (modo save)
  let byOpp = null;
  if (job.schedule) {
    const opps = [...new Set(rows.map(r => r.opponent).filter(Boolean))];
    byOpp = opps.map(o => ({ key: o, name: (Object.values(job.schedule).find(s => (s.opponent_key || s.opponent) === o) || {}).opponent || o,
      A: summarize(rows.filter(r => r.opponent === o && r.group === 'A')), B: summarize(rows.filter(r => r.opponent === o && r.group === 'B')) }));
  }
  const names = job.variant_names || {};
  return { id: job.id, name: job.name || job.id, status: job.status, completed: job.completed, total: job.total, mode: job.mode, type: job.experiment_type || 'free', per, comparison, byOpp, names, rows };
}

// pacote de exportação (/api/jobs/{id}/export) ou o próprio job
function jobFromAny(d) {
  if (!d || typeof d !== 'object') return null;
  if (d.summary && Array.isArray(d.summary.runs)) return Object.assign({}, d.summary, { _scenarios: d.scenarios || null });
  if (Array.isArray(d.runs) && (d.metrics || d.status)) return d;
  return null;
}
