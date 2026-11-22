// Snapshot every TeamTacticReference prop FM's screen uses (from the seq log) to a file; diff two snapshots.
// node dev/_tsnap.mjs save <file> | node dev/_tsnap.mjs diff <a> <b>
import { FMB, call } from '../fmb-node.mjs';
import fs from 'node:fs';
const [cmd, a, b] = process.argv.slice(2);
if (cmd === 'diff') {
  const A = JSON.parse(fs.readFileSync(a, 'utf8')), B = JSON.parse(fs.readFileSync(b, 'utf8'));
  for (const k of Object.keys(B)) if (A[k] !== B[k]) console.log(k, '\n  before', String(A[k]).slice(0, 400), '\n  after ', String(B[k]).slice(0, 400));
  process.exit(0);
}
const seq = fs.readFileSync(process.env.APPDATA + '/FM27TacticalBible/bridge_seq.txt', 'utf8');
const props = [...new Set([...seq.matchAll(/TeamTacticReference\|[^|]*\|(\d+)/g)].map(m => m[1]))];
const c = await FMB.context(), P = `825307168=126;1198877028=1;1651271022=1;1685218662=${c.team}`;
const out = {};
for (let i = 0; i < props.length; i += 50) {
  const ch = props.slice(i, i + 50);
  const r = (await call('READ ' + ch.map(p => `FM.UI.TeamTacticReference|${P}|${p}`).join(' ;; '), 30000)).slice(3).split('\u001e');
  ch.forEach((p, k) => out[p] = r[k]);
}
fs.writeFileSync(a, JSON.stringify(out));
console.log('saved', props.length, 'props');
