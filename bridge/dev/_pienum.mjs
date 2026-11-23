// Player-instruction categories of the player selected in FM (his instructions panel must be open):
// for every category id step ‹ to the lowest option, then › up, reading the label the screen shows.
// node dev/_pienum.mjs [maxCat]    (needs SEQ 1 + DATAHOOK 1). Results are merged into %TEMP%/pi_cats.json
import { FMB, FMB_IDS, call } from '../fmb-node.mjs';
import fs from 'node:fs';
const c = await FMB.context(), P = FMB_IDS.tactic.pairs(c), SEQ = process.env.APPDATA + '/FM27TacticalBible/bridge_seq.txt';
const PREV = 1668445234, NEXT = 1668445233, sleep = ms => new Promise(r => setTimeout(r, ms));
const sel = JSON.parse((await call(`READAT auto FM.UI.TeamTacticReference|${P}|2001486437`)).slice(3).replace(/[\u0000-\u001f]/g, ' '));
const f = (k) => { const e = sel.entries.find(x => x[0] === k); return e ? e[2] : undefined; };
const slot = process.env.SLOT || f(1414752632), ipRole = f(1698852728), pos = f(1349481321);
const press = (ev, cat) => call(`SENDJ ${ev} FM.UI.TeamTacticReference ${P} ${JSON.stringify({ type: 'SI.Bindable.DynamicReference', entries: [[1414752632, 'SI.Core.DynamicNumber', String(slot)], [1449616730, 'SI.Core.DynamicNumber', String(cat)], [1265792339, 'System.Boolean', 'True']] })}`);
const size = () => fs.statSync(SEQ).size;
const NOISE = /^(True|False|This Season|FM\.UI\.|SI\.|\d+$|Dpad|RightStick|LeftStick|Submit|Tactics$|Overview$|Lst$|Loa$|FtT$|Inj$|Wnt$|Req$|Lng$|Unr$|tactic_icons)/;
function labels(pos0) { const n = size() - pos0, buf = Buffer.alloc(n), fd = fs.openSync(SEQ, 'r'); fs.readSync(fd, buf, 0, n, pos0); fs.closeSync(fd);
  return buf.toString('utf8').split('\n').map(l => l.match(/SetData (\S+) "(.{2,50})"\r?$/)).filter(Boolean).map(m => m[2]).filter(t => !NOISE.test(t)); }
const out = {}, max = +process.argv[2] || 14;
console.log(`slot ${slot} position ${pos} role ${ipRole}`);
for (let cat = 0; cat <= max; cat++) {
  let last = null; const seq = [];
  for (let i = 0; i < 7; i++) { const p0 = size(); await press(PREV, cat); await sleep(450); const l = labels(p0).at(-1); if (l) last = l; }
  if (last) seq.push(last);
  for (let i = 0; i < 7; i++) { const p0 = size(); await press(NEXT, cat); await sleep(450); const l = labels(p0).at(-1); if (l && l !== seq.at(-1)) seq.push(l); }
  console.log(String(cat).padStart(2), seq.length ? seq.join(' | ') : '(not for this player)');
  if (seq.length) out[cat] = seq;
}
const file = process.env.TEMP + '/pi_cats.json', all = fs.existsSync(file) ? JSON.parse(fs.readFileSync(file, 'utf8')) : {};
all[`${pos}:${ipRole}`] = out; fs.writeFileSync(file, JSON.stringify(all, null, 1));
