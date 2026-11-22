// Enumerate team-instruction options with FM's Team Instructions panel open: for each instruction id press "previous"
// until the start, then "next" one step at a time; read the label FM's screen shows (SetData in the seq log).
// node dev/_tienum.mjs [ids…]   (needs SEQ 1 + DATAHOOK 1)
import { FMB, call } from '../fmb-node.mjs';
import fs from 'node:fs';
const c = await FMB.context(), P = `825307168=126;1198877028=1;1651271022=1;1685218662=${c.team}`;
const PREV = 1249276755, NEXT = 862931569, SEQ = process.env.APPDATA + '/FM27TacticalBible/bridge_seq.txt';
const sleep = ms => new Promise(r => setTimeout(r, ms));
const press = (ev, id) => call(`SENDJ ${ev} FM.UI.TeamTacticReference ${P} ["SI.Core.DynamicNumber","${id}"]`);
const size = () => fs.statSync(SEQ).size;
function labelSince(pos) {   // last short label FM's screen set after file position pos
  const fd = fs.openSync(SEQ, 'r'), n = size() - pos, buf = Buffer.alloc(Math.max(0, n)); fs.readSync(fd, buf, 0, n, pos); fs.closeSync(fd);
  const ls = buf.toString('utf8').split('\n').map(l => l.match(/SetData \S+ "(.{2,40})"\r?$/)).filter(Boolean).map(m => m[1])
    .filter(t => !/^(True|False|This Season|FM\.UI\.|Custom$|\d+$)/.test(t) && !/^\u0001/.test(t));
  return ls.at(-1) || null;
}
const ids = process.argv.slice(2).length ? process.argv.slice(2).map(Number) : Array.from({ length: 30 }, (_, i) => i + 1);
const out = {};
for (const id of ids) {
  for (let i = 0; i < 7; i++) await press(PREV, id);
  await sleep(450);
  await press(NEXT, id); await sleep(700);
  let pos = size(); await press(PREV, id); await sleep(700);
  const first = labelSince(pos); if (!first) { console.log(String(id).padStart(2), '(not on the visible tab)'); continue; }
  const opts = [first];
  for (let i = 0; i < 7; i++) {
    pos = size(); await press(NEXT, id); await sleep(700);
    const l = labelSince(pos); if (!l || l === opts.at(-1)) break; opts.push(l);
  }
  out[id] = opts; console.log(String(id).padStart(2), opts.join(' | '));
}
const f = process.env.TEMP + '/ti_options.json', old = fs.existsSync(f) ? JSON.parse(fs.readFileSync(f, 'utf8')) : {};
fs.writeFileSync(f, JSON.stringify(Object.assign(old, out), null, 1));
