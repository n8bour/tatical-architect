// End-to-end test of the app's corner-routine apply (FMB.readRoutine / applyRoutine / setInstruction) on the running FM.
// FM must show Tactics › Set Pieces › Corners. It gives FM's current players planned roles, applies, reads back, then
// RESTORES every slot's original instruction.   node bridge/dev/_routine.mjs [--keep]
import { FMB, sandbox } from '../fmb-node.mjs';
sandbox.shortName = (p) => p.name;
const c = await FMB.context();
await FMB.status(true);
console.log('plugin', FMB.state.version, 'routines supported:', FMB.canRoutines());
const before = await FMB.readRoutine(c);
if (!before) { console.log('Set Pieces screen not open in FM'); process.exit(1); }
const show = (r) => r.slots.map(s => `  g${s.group} ${s.name} person ${s.person}: ${s.label} (${s.option})`).join('\n');
console.log('kind', before.kind, 'side', before.side, '\n' + show(before));
// fake plan: roles for the players FM has in the slots, as the app's setPiecePlan would produce them
const att = before.kind === 5;
const keys = att ? ['far', 'near', 'centre', 'spot', 'edge', 'short', 'back0', 'back1', 'back2'] : ['zn', 'zc', 'zf', 'm0', 'm1', 'de', 'o0', 'o1'];
const personOf = {}, rows = [];
before.slots.forEach((s, i) => { if (i >= keys.length) return; const uid = 900000 + i; personOf[uid] = s.person;
  rows.push({ k: keys[i], n: keys[i], x: { p: { name: 'P' + s.person + '/' + keys[i], meta: { uid } } } }); });
const plan = att ? { attack: rows, defend: [] } : { attack: [], defend: rows };
console.log('\napply:\n' + (await FMB.applyRoutine(c, plan, before, personOf)).join('\n'));
await new Promise(r => setTimeout(r, 800));
const after = await FMB.readRoutine(c);
console.log('\nafter:\n' + show(after));
if (process.argv.includes('--keep')) process.exit(0);
for (const s of before.slots) {
  const now = after.slots.find(x => x.group === s.group && x.slot === s.slot);
  if (now && now.option !== s.option) await FMB.setInstruction(c, before.kind, s, { id: s.option, label: s.label });
}
await new Promise(r => setTimeout(r, 800));
const back = await FMB.readRoutine(c);
const same = back.slots.every(s => before.slots.find(x => x.group === s.group && x.slot === s.slot)?.option === s.option);
console.log('\nrestored:', same ? 'yes, identical to before' : 'NO\n' + show(back));
