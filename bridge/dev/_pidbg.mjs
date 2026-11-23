// raw strings after each press of a category: node dev/_pidbg.mjs <slot> <cat> [presses]
import { FMB, FMB_IDS, call } from '../fmb-node.mjs';
import fs from 'node:fs';
const c = await FMB.context(), P = FMB_IDS.tactic.pairs(c), SEQ = process.env.APPDATA + '/FM27TacticalBible/bridge_seq.txt';
const [slot, cat, n = 6] = process.argv.slice(2);
const sleep = ms => new Promise(r => setTimeout(r, ms));
const press = (ev) => call(`SENDJ ${ev} FM.UI.TeamTacticReference ${P} ${JSON.stringify({ type: 'SI.Bindable.DynamicReference', entries: [[1414752632, 'SI.Core.DynamicNumber', slot], [1449616730, 'SI.Core.DynamicNumber', cat], [1265792339, 'System.Boolean', 'True']] })}`);
for (let i = 0; i < +n; i++) {
  const p0 = fs.statSync(SEQ).size; await press(1668445233); await sleep(700);
  const n0 = fs.statSync(SEQ).size - p0, buf = Buffer.alloc(n0), fd = fs.openSync(SEQ, 'r'); fs.readSync(fd, buf, 0, n0, p0); fs.closeSync(fd);
  const s = buf.toString('utf8').split('\n').map(l => l.match(/^(\S+) SetData (\S+) "(.{2,60})"\r?$/)).filter(Boolean).map(m => `${m[2]}=${m[3]}`).filter(x => !/=(True|False|This Season|FM\.UI|SI\.|\d+$|Lst$|Loa$|FtT$|Inj$|Wnt$|Req$|Lng$|Unr$)/.test(x));
  console.log(`press ${i + 1}:`, [...new Set(s)].join(' | ').slice(0, 300));
}
