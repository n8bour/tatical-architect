// Set some team instructions (app key=appOptionIndex) through FMB_IDS.tactic.ti. FMB_CORE=1 node dev/_tiset.mjs dline=4 press_trap=0 …
import { FMB, FMB_IDS, evalIn } from '../fmb-node.mjs';
const c = await FMB.context(), T = FMB_IDS.tactic, P = T.pairs(c), TI = evalIn('TI_BY_KEY');
for (const a of process.argv.slice(2)) {
  const [k, v] = a.split('='), [id, count, map] = T.ti[k], target = map[+v];
  for (let i = 0; i < count; i++) { await FMB.send(T.tiPrev, 'TeamTacticReference', P, ['SI.Core.DynamicNumber', String(id)]); await new Promise(r => setTimeout(r, 150)); }
  for (let i = 0; i < target; i++) { await FMB.send(T.tiNext, 'TeamTacticReference', P, ['SI.Core.DynamicNumber', String(id)]); await new Promise(r => setTimeout(r, 150)); }
  console.log(TI[k].n.padEnd(28), '→', TI[k].opts[+v]);
}
