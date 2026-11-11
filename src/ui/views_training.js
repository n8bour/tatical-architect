// ============================================================
// Treino
// ============================================================
function vTraining() {
  const t = ST.tactic, au = auditTactic(t, players()), plan = trainingPlan(t, players(), au);
  return `<div class="top"><div><h2>Plano de treino para “${esc(t.name)}”</h2>
    <p class="muted small">Ligado às áreas de Tactical Understanding do FM27: treine primeiro o que a sua tática mais exige.</p></div></div>
  <div class="grid g2">
    <div class="panel"><h3>Prioridades de Tactical Understanding</h3>
      ${plan.areas.map((a, i) => `<div class="finding"><div class="sev ${i < 2 ? 'warn' : 'info'}">${i + 1}</div><div><b>${TRAINING.areas[a.k].n}</b><span class="small muted">${[...new Set(a.why)].map(esc).join(' · ')}</span></div></div>`).join('')}
      ${plan.cohesion ? '<div class="notice warn">Complexidade acima do que o XI absorve: inclua Team Cohesion e reduza instruções.</div>' : ''}
    </div>
    <div class="panel"><h3>Semana-tipo sugerida</h3>
      <table><tbody>
      ${[['Dia 1 (pós-jogo)','Recuperação'],['Dia 2', TRAINING.areas[(plan.areas[0] || { k: 'ip_shape' }).k].n],['Dia 3', TRAINING.areas[(plan.areas[1] || { k: 'oop_shape' }).k].n + ' + Individual'],['Dia 4', TRAINING.areas[(plan.areas[2] || { k: 'buildup' }).k].n],['Dia 5 (véspera)','Match Preparation (próximo adversário) + bolas paradas']]
        .map(([d, s]) => `<tr><td><b>${d}</b></td><td>${esc(s)}</td></tr>`).join('')}
      </tbody></table>
      <p class="small muted">Semanas com dois jogos: só Recuperação + Match Preparation.</p></div>
  </div>
  <div class="panel" style="margin-top:16px"><h3>Desenvolvimento individual do XI</h3>
    ${plan.indiv.length ? `<div class="table-wrap"><table><thead><tr><th>Jogador</th><th>Pos</th><th>IP</th><th>OOP</th><th>Role Training</th><th>Foco adicional</th></tr></thead><tbody>
      ${plan.indiv.map(x => `<tr><td><b>${esc(x.p.name)}</b></td><td>${x.slot}</td><td>${fitBadge(x.si)}</td><td>${fitBadge(x.so)}</td><td>${esc(x.role.name)}</td><td>${x.weak.map(k => `<span class="chip">${esc(attrName(k))} ${Math.round(x.p.a[k])}</span>`).join('')}</td></tr>`).join('')}
    </tbody></table></div>` : '<p class="muted">Atribua jogadores ao XI (Plantel → Analisador → Auto XI) para recomendações individuais.</p>'}
    ${plan.mentors.length && plan.youth.length ? `<h4 style="margin-top:12px">Mentoria</h4><p class="small">Mentores (Determinação ≥ 15, ≥ 26 anos): ${plan.mentors.slice(0, 4).map(p => esc(p.name)).join(', ')}. Jovens ≤ 21: ${plan.youth.slice(0, 8).map(p => esc(p.name)).join(', ')}.</p>` : ''}
  </div>`;
}
