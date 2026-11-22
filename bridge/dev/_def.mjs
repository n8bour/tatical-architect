import { call } from '../fmb-node.mjs';
const P = '825307168=93;1198877028=1;1348620377=17439426;1685218662=402653436';
const sleep = ms => new Promise(r => setTimeout(r, ms));
const f = (o, k) => { const e = o && o.entries && o.entries.find(x => x[0] == k); return e ? e[2] : undefined; };
export async function readView(v) {   // v: "1" attacking, "2" defending (engine context only), restores attacking
  if (v !== '1') { await call(`SETDATA 7600:4 ["SI.Core.DynamicNumber","${v}"]`); await sleep(300); }
  const r = JSON.parse((await call(`READAT auto FM.UI.SetPieceReference|${P}|1867869768`)).slice(3).replace(/[\u0000-\u001f]/g, ' '));
  if (v !== '1') await call(`SETDATA 7600:4 ["SI.Core.DynamicNumber","1"]`);
  return r;
}
export function show(r, opts) {
  console.log('kind', f(r, 1398883404), 'att/def', f(r, 1919054668));
  for (const g of [...f(r, 1750096198), ...f(r, 945304884)].map(x => x[1])) {
    const sl = (f(g, 1752528242) || []).map(x => x[1]);
    console.log(' group', f(g, 1114920816), sl.map(s => `${f(s, 1851878757)}=${f(f(s, 1750627616), 1903577654)}(${f(f(f(s, 1750627616), 1886680684), 1750627616)})`).join(' | ') || '(empty)');
    if (opts && sl[0]) console.log('   options:', (f(sl[0], 1833723970) || []).map(x => f(x[1], 1383036740)).map(i => f(i, 1903577654) + '=' + f(f(i, 1886680684), 1750627616)).join(' ; '));
  }
}
if (process.argv[2]) show(await readView(process.argv[2]), process.argv[3]);
