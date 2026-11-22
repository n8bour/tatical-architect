// Which TeamReference property is the next fixture? node dev/_fixture.mjs
import { call } from '../fmb-node.mjs';
const T = '825307168=28;1198877028=1;1685218662=402653436';
const props = [1148676708, 1413904460, 1668247142, 1852130921, 1935767927];
const name = async (id) => (await call(`READ FM.UI.TeamReference|825307168=28;1198877028=1;1685218662=${id}|1851878757`)).slice(3).replace(/[\u0000-\u001f]/g, ' ').replace(/^.*\u0002|\u0001"$|"/g, '').slice(-30);
const r = (await call('READ ' + props.map(p => `FM.UI.TeamReference|${T}|${p}`).join(' ;; '))).slice(3).split('\u001e');
for (const [i, p] of props.entries()) {
  let v; try { v = JSON.parse(r[i].replace(/[\u0000-\u001f]/g, ' ')); } catch (e) { console.log(p, r[i].slice(0, 100)); continue; }
  const refs = Array.isArray(v) ? v.map(x => x[1]) : [v];
  const out = [];
  for (const f of refs.slice(0, 3)) {
    if (!f || !f.pairs) { out.push(JSON.stringify(f).slice(0, 60)); continue; }
    const pr = Object.fromEntries(f.pairs);
    out.push(`[${await name(pr[1718119764])} vs ${await name(pr[1717663572])}] ${f.pairs.map(q => q.join('=')).join(';')}`);
  }
  console.log(p, Array.isArray(v) ? `list(${v.length})` : 'one', '\n   ' + out.join('\n   '));
}
