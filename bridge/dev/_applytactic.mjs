// Apply an app template to FM's ACTIVE tactic through the app's FMB.applyTactic. FMB_CORE=1 node dev/_applytactic.mjs [templateId]
import { FMB, sandbox, evalIn } from '../fmb-node.mjs';
sandbox.slotInfo = sandbox.slotInfo || (() => ({ x: 0 }));
const tp = evalIn("TEMPLATES").find(t => t.id === (process.argv[2] || 'possession433')) || sandbox.TEMPLATES[0];
const t = evalIn("tacticFromTemplate")(tp);
console.log('template', tp.id, t.ipForm, '/', t.oopForm, t.ment, '| slots', t.slots.map(s => `${s.ipCode}:${s.ip}/${s.oop}`).join(' '));
const lines = await FMB.applyTactic(t, [], (x) => console.log('  ..', x), { create: process.argv.includes('--new') });
console.log(lines.join('\n'));
const after = await FMB.readTacticSlots(await FMB.context());
console.log('FM now:', after.map(s => `${s.slot}:${s.ip} ip=${s.ipRole} oop=${s.oopRole}`).join(' | '));
