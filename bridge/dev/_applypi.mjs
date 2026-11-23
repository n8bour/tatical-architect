// Apply the possession template with player instructions on some slots. FMB_CORE=1 node dev/_applypi.mjs
import { FMB, evalIn } from '../fmb-node.mjs';
const t = evalIn('tacticFromTemplate')(evalIn('TEMPLATES').find(x => x.id === 'possession433'));
const set = { AML: ['Dribble more', 'Stay wider', 'Cross from byline', 'Shoot more'], AMR: ['Roam from position', 'Cross from deep', 'Dribble less'], ST: ['Take more risks', 'Shoot less', 'Get further forward'],
  DM: ['Tackle harder', 'Press more', 'Mark tighter'], DCL: ['Take fewer risks', 'Hold position'], MCL: ['Press less', 'Ease off tackles'] };
t.slots.forEach(s => { s.pis = set[s.ipCode] || []; });
console.log((await FMB.applyTactic(t, [], (x) => console.log('  ..', x))).join('\n'));
const slots = await FMB.readTacticSlots(await FMB.context());
console.log('slots:', slots.map(x => `${x.slot}=${x.ip}`).join(' '));
