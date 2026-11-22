import { call } from '../fmb-node.mjs';
const P = '825307168=93;1198877028=1;1348620377=17439426;1685218662=402653436';
const N = (k, v) => [k, 'SI.Core.DynamicNumber', String(v)];
const DR = entries => ({ type: 'SI.Bindable.DynamicReference', entries });
const REC = entries => ({ type: 'SI.Core.Record', entries });
const flags = [N(1769491558, 0), N(857762170, 1), N(877999731, 0)];
// node _move.mjs <srcGroup> <srcSlot> group <dstGroup>   |   node _move.mjs <srcGroup> <srcSlot> slot <dstGroup> <dstSlot>
const [sg, ss, mode, dg, ds] = process.argv.slice(2);
const src = DR([N(1883858041, ss), N(1114920816, sg)]);
const dst = mode === 'group' ? DR([N(1114920816, dg)]) : DR([N(1883858041, ds), N(1114920816, dg)]);
const rec = REC([[929121399, 'SI.Core.Record', REC([[929121399, 'SI.Bindable.DynamicReference', src], ...flags])],
                 [944851554, 'SI.Core.Record', REC([[944851554, 'SI.Bindable.DynamicReference', dst], ...flags])]]);
console.log(await call(`SENDJ 1449477207 FM.UI.SetPieceReference ${P} ${JSON.stringify(rec)}`));
await new Promise(r => setTimeout(r, 800));
const r = JSON.parse((await call(`READAT auto FM.UI.SetPieceReference|${P}|1867869768`)).slice(3).replace(/[\u0000-\u001f]/g, ' '));
const f = (o, k) => { const e = o && o.entries && o.entries.find(x => x[0] == k); return e ? e[2] : undefined; };
for (const g of [...f(r, 1750096198), ...f(r, 945304884)].map(x => x[1])) {
  const sl = (f(g, 1752528242) || []).map(x => x[1]);
  console.log('group', f(g, 1114920816), sl.map(s => `${f(s, 1851878757)}=${f(f(s, 1750627616), 1903577654)}`).join(' | ') || '(empty)');
}
