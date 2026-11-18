// ============================================================
// Situações de jogo — planos B/C a partir da tática actual
// ============================================================
function vGamePlans() {
  const ps = players(), k = ST.planK || 'chase';
  const r = gamePlan(ST.tactic, ps, k, { opp: oppPlayers() }); ST._plan = r;
  const sc = SCENARIOS();
  const delta = (a, b) => { const d = b - a; return `<span class="${d > 0 ? 'up' : d < 0 ? 'down' : 'muted'}">${a} → <b>${b}</b> ${d ? `(${d > 0 ? '+' : ''}${d})` : ''}</span>`; };
  const grpC = (kind) => r.changes.filter(c => c.kind === kind);
  const tiRows = grpC('ti'), roleRows = grpC('role'), piRows = grpC('pi'), ment = grpC('ment')[0];
  const sec = (title, body) => body ? `<div class="plan-sec"><h4>${title}</h4>${body}</div>` : '';
  const tbl = (rows) => `<table><tbody>${rows.join('')}</tbody></table>`;
  return `<div class="top"><div><h2>${L('Situações de jogo', 'Game states')}</h2><p class="muted small" style="margin:0">${L(`Planos a partir de “${esc(tName(ST.tactic))}”${ps.length ? ' e do teu plantel' : ''}. No FM27 pode guardar até 3 táticas: use a base como A e estes planos como B e C (por exemplo, B = Precisamos de golos, C = Segurar o resultado).`, `Plans built from “${esc(tName(ST.tactic))}”${ps.length ? ' and your squad' : ''}. FM27 lets you keep up to 3 tactics: use the base as A and these plans as B and C (e.g. B = We need goals, C = Protect the lead).`)}</p></div>
    <div class="row"><button class="btn primary" onclick="A.planSave()">${L('Guardar como tática', 'Save as tactic')}</button><button class="btn" onclick="A.planUse()">${L('Usar agora', 'Use now')}</button><button class="btn" onclick="A.planCopy()">${L('Copiar como texto', 'Copy as text')}</button></div></div>
  <div class="plan-tabs" role="tablist">${sc.map(s => `<button role="tab" aria-selected="${s.k === k}" class="${s.k === k ? 'on' : ''}" onclick="A.planK('${s.k}')"><span aria-hidden="true">${s.ic}</span> ${esc(s.n)}</button>`).join('')}</div>
  ${!ps.length ? `<div class="notice small" style="margin:10px 0">${L('Sem plantel carregado: os ajustes seguem só a estrutura da tática; as substituições e as instruções individuais precisam do plantel.', 'No squad loaded: adjustments follow the tactic’s structure only; substitutions and player instructions need the squad.')}</div>` : ''}
  <div class="grid" style="grid-template-columns:minmax(0,1.6fr) minmax(280px,1fr);gap:16px;align-items:start;margin-top:12px">
    <div class="panel">
      <p class="small" style="margin-top:0"><b>${esc(r.sc.n)}.</b> ${esc(r.sc.d)}</p>
      ${sec(L('Mentalidade', 'Mentality'), ment ? `<p class="small"><b>${esc(ment.from)} → ${esc(ment.to)}</b> — ${esc(ment.why)}</p>` : `<p class="small muted">${L('Mantém-se.', 'Unchanged.')}</p>`)}
      ${sec(L('Instruções de equipa', 'Team instructions'), tiRows.length ? tbl(tiRows.map(c => `<tr><td style="width:30%">${esc(c.label)}</td><td style="width:28%"><span class="muted">${esc(c.from)}</span> → <b>${esc(c.to)}</b></td><td class="small muted">${esc(c.why)}</td></tr>`)) : `<p class="small muted">${L('Sem mudanças.', 'No changes.')}</p>`)}
      ${sec(L('Papéis', 'Roles'), roleRows.length ? tbl(roleRows.map(c => `<tr><td style="width:18%"><b>${esc(c.code)}</b> <span class="chip ${c.phase === 'IP' ? 'ip' : 'oop'}">${c.phase}</span></td><td style="width:20%">${esc(c.player || '')}</td><td style="width:30%"><span class="muted">${esc(c.from)}</span> → <b>${esc(c.to)}</b></td><td class="small muted">${esc(c.why)}</td></tr>`)) : `<p class="small muted">${L('Os papéis actuais já servem este cenário (nenhuma troca melhorava o conjunto).', 'The current roles already suit this scenario (no swap improved the whole).')}</p>`)}
      ${sec(L('Instruções individuais', 'Player instructions'), piRows.length ? tbl(piRows.map(c => `<tr><td style="width:18%"><b>${esc(c.code)}</b></td><td style="width:20%">${esc(c.player)}</td><td style="width:30%">${c.removed ? `<s>${esc(c.to)}</s>` : `<b>${esc(c.to)}</b>`}</td><td class="small muted">${esc(c.why)}</td></tr>`)) : '')}
      ${sec('Tactical Tweaks', r.tweaks.length ? `<ul class="clean small">${r.tweaks.map(x => `<li><b>${esc(x.n)}</b> — ${esc(x.why)}</li>`).join('')}</ul>` : '')}
      ${sec(L('Substituições sugeridas', 'Suggested substitutions'), r.subs.length ? tbl(r.subs.map(x => `<tr><td style="width:18%"><b>${esc(x.code)}</b></td><td style="width:42%"><span class="muted">${esc(x.out)}</span> → <b>${esc(x.in)}</b></td><td class="small muted">${esc(x.why)}</td></tr>`)) : '')}
      ${sec(L('Instruções de adversário', 'Opposition instructions'), r.ois.length ? tbl(r.ois.map(x => `<tr><td style="width:30%"><b>${esc(x.p)}</b> <span class="muted small">${esc(x.code)}</span></td><td>${x.ois.map(o => `<span class="chip">${esc(o)}</span>`).join(' ')}</td></tr>`)) : '')}
      ${r.notes.length ? `<div class="plan-sec">${r.notes.map(n => `<p class="small">💡 ${esc(n)}</p>`).join('')}</div>` : ''}
    </div>
    <div class="grid">
      <div class="panel"><h3>${L('Impacto', 'Impact')}</h3>
        <table><tbody>
          <tr><td>${L('Compromisso ofensivo', 'Attacking commitment')}</td><td>${delta(r.intentBefore.att, r.intentAfter.att)}</td></tr>
          <tr><td>${L('Segurança defensiva', 'Defensive security')}</td><td>${delta(r.intentBefore.sec, r.intentAfter.sec)}</td></tr>
          <tr><td colspan="2" class="small muted" style="padding-top:10px">${L('Sinergia entre papéis (coerência)', 'Role synergy (coherence)')}</td></tr>
          ${['build', 'att', 'def', 'trans'].map(ph => `<tr><td class="small">${esc(SYN_PHASES()[ph])}</td><td class="small">${delta(r.before.phases[ph].score, r.after.phases[ph].score)}</td></tr>`).join('')}
          <tr><td class="small">${L('Diagnóstico', 'Diagnosis')}</td><td class="small">${delta(r.auditBefore.overall, r.auditAfter.overall)}</td></tr>
        </tbody></table>
        <p class="small muted" style="margin-bottom:0">${L('Compromisso e segurança medem a intensidade (mentalidade, ritmo, pressão, quem fica atrás). A sinergia mede se os papéis continuam coerentes. Um plano para marcar troca segurança por compromisso — é o preço.', 'Commitment and security measure intensity (mentality, tempo, pressing, who stays back). Synergy measures whether the roles stay coherent. A plan to score trades security for commitment — that is the price.')}</p>
      </div>
      <div class="panel"><h3>${L('No jogo', 'In the game')}</h3><ol class="clean small">
        <li>${L('Guarde este plano como tática (botão acima) e recrie-o no FM27 como tática B ou C, a partir da Ficha.', 'Save this plan as a tactic (button above) and recreate it in FM27 as tactic B or C, using the Sheet.')}</li>
        <li>${L('Durante o jogo, mude de tática no ecrã de táticas — ou use os Tactical Tweaks indicados para uma mudança rápida.', 'During the match, switch tactics on the tactics screen — or use the listed Tactical Tweaks for a quick change.')}</li>
        <li>${L('As instruções individuais só mudam quando a tática B/C é aplicada.', 'Player instructions only change when tactic B/C is applied.')}</li></ol></div>
    </div>
  </div>`;
}
