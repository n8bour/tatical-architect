// Formation moves on the active tactic: node dev/_formation.mjs show | move <slot> <toBits> <ip|oop> | ment <n> <label>
import { FMB, call } from '../fmb-node.mjs';
const c = await FMB.context(), P = `825307168=126;1198877028=1;1651271022=1;1685218662=${c.team}`;
const f = (o, k) => { const e = o && o.entries && o.entries.find(x => x[0] === k); return e ? e[2] : undefined; };
async function show() {
  const [slots, ment, name] = (await call(`READ FM.UI.TeamTacticReference|${P}|1919628150 ;; FM.UI.TeamTacticReference|${P}|1162170689 ;; FM.UI.TeamTacticReference|${P}|1851878757`)).slice(3).split('\u001e').map(x => JSON.parse(x.replace(/[\u0000-\u001f]/g, ' ')));
  console.log(name, '| mentality', JSON.stringify(ment && ment.entries.map(e => e[2])));
  for (const s of slots) { const r = s[1]; console.log(`  slot ${f(r, 1414752632)}  IP ${f(r, 1835167823)}  OOP ${f(r, 1668237639)}  person ${f(r, 1886157170)?.pairs?.at(-1)?.[1]}`); }
  return slots.map(s => s[1]);
}
const [cmd, a, b, d] = process.argv.slice(2);
const slots = await show();
if (cmd === 'move') {
  const ip = d !== 'oop', s = slots.find(r => +f(r, 1414752632) === +a), from = +f(s, ip ? 1835167823 : 1668237639);
  const R = (k, pos, slot) => [k, 'SI.Core.Record', { type: 'SI.Core.Record', entries: [[k, 'SI.Core.DynamicNumber', String(pos)], [1769491558, 'System.Boolean', ip ? 'True' : 'False'], ...(slot != null ? [[857762170, 'SI.Core.DynamicNumber', String(slot)]] : [])] }];
  const rec = { type: 'SI.Core.Record', entries: [R(929121399, from, +a), R(944851554, +b)] };
  console.log('move', a, from, '->', b, await call(`SENDJ 21 FM.UI.TeamTacticReference ${P} ${JSON.stringify(rec)}`));
} else if (cmd === 'ment') {
  console.log('mentality', await call(`WRITE FM.UI.TeamTacticReference|${P}|1162170689 ${JSON.stringify({ type: 'SI.Bindable.DynamicReference', entries: [[1886680684, 'SI.Core.DynamicNumber', a], [1903577654, 'System.String', b]] })}`));
}
if (cmd) { await new Promise(r => setTimeout(r, 800)); console.log('after:'); await show(); }
