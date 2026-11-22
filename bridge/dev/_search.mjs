// FM search from the bridge: node dev/_search.mjs "<name>" [unique_id]
import { call } from '../fmb-node.mjs';
const P = '825307168=98;1198877028=1;1719300978=1;1954115685=0', name = process.argv[2], want = process.argv[3];
const sleep = ms => new Promise(r => setTimeout(r, ms));
console.log('write', await call(`WRITE FM.UI.SearchReference|${P}|1851878757 ["System.String",${JSON.stringify(name)}]`));
for (let i = 0; i < 8; i++) {
  await sleep(500);
  const r = JSON.parse((await call(`READ FM.UI.SearchReference|${P}|1886547059`)).slice(3).replace(/[\u0000-\u001f]/g, ' '));
  const ids = (r || []).map(x => x[1].pairs.find(p => p[0] === 1685218662)[1]);
  if (!ids.length) continue;
  const q = ids.map(id => `FM.UI.PersonReference|825307168=28;1198877028=1;1685218662=${id}|1970170212`).join(' ;; ');
  const uids = (await call('READ ' + q)).slice(3).split('\u001e').map(x => x.replace(/"/g, ''));
  console.log('results:', ids.map((id, k) => `${id} (unique_id ${uids[k]})${want && uids[k] === want ? '  <== match' : ''}`).join('\n         '));
  process.exit(0);
}
console.log('no results');
