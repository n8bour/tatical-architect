import { call } from '../fmb-node.mjs';
const P = '825307168=93;1198877028=1;1348620377=17439426;1685218662=402653436';
const [group, slot, option, label, kind = '5'] = process.argv.slice(2);
const rec = { type: 'SI.Bindable.DynamicReference', entries: [[1903577654, 'System.String', label], [1886680684, 'SI.Bindable.DynamicReference', { type: 'SI.Bindable.DynamicReference', entries: [
  [1114920816, 'SI.Core.DynamicNumber', group], [1883858041, 'SI.Core.DynamicNumber', slot], [1750627616, 'SI.Core.DynamicNumber', option], [1398883404, 'SI.Core.DynamicNumber', kind], [1279880290, 'SI.Core.DynamicNumber', '0']] }]] };
console.log(await call(`SENDJ 1633039705 FM.UI.SetPieceReference ${P} ${JSON.stringify(rec)}`));
await new Promise(r => setTimeout(r, 800));
const r = JSON.parse((await call(`READAT auto FM.UI.SetPieceReference|${P}|1867869768`)).slice(3).replace(/[\u0000-\u001f]/g, ' '));
const f = (o, k) => { const e = o && o.entries && o.entries.find(x => x[0] == k); return e ? e[2] : undefined; };
for (const g of [...f(r, 1750096198), ...f(r, 945304884)].map(x => x[1]))
  for (const s of (f(g, 1752528242) || []).map(x => x[1])) console.log('group', f(g, 1114920816), 'slot', f(s, 1883858041), f(s, 1851878757), '|', f(f(s, 1750627616), 1903577654));
