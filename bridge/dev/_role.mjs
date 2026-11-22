// Set a slot's role: node dev/_role.mjs <slot> <roleBits> <name> <ip|oop>   (then reads back both role lists)
import { FMB, call } from '../fmb-node.mjs';
const c = await FMB.context(), P = `825307168=126;1198877028=1;1651271022=1;1685218662=${c.team}`;
const f = (o, k) => { const e = o && o.entries && o.entries.find(x => x[0] === k); return e ? e[2] : undefined; };
const [slot, bits, name, side] = process.argv.slice(2);
const N = (k, v) => [k, 'SI.Core.DynamicNumber', String(v)], DR = (entries) => ({ type: 'SI.Bindable.DynamicReference', entries });
const role = DR([N(1349481322, bits), [1464367445, 'SI.Bindable.DynamicReference', DR([N(1298233430, 14), N(1298755158, 14), [1397715049, 'System.Boolean', 'False']])], [1936487524, 'System.Boolean', 'False'], [909713440, 'System.String', name]]);
const rec = { type: 'SI.Core.Record', entries: [N(1414752632, slot), [1349481322, 'SI.Bindable.DynamicReference', DR([N(1886680684, 1), [1903577654, 'SI.Bindable.DynamicReference', role]])], [1265792339, 'System.Boolean', side === 'oop' ? 'False' : 'True']] };
console.log('send', await call(`SENDJ 1467503190 FM.UI.TeamTacticReference ${P} ${JSON.stringify(rec)}`));
await new Promise(r => setTimeout(r, 800));
const [ip, oop] = (await call(`READ FM.UI.TeamTacticReference|${P}|930166885 ;; FM.UI.TeamTacticReference|${P}|1798391651`)).slice(3).split('\u001e').map(x => JSON.parse(x.replace(/[\u0000-\u001f]/g, ' ')));
const s1 = ip.find(x => +f(x[1], 1414752632) === +slot)[1], s2 = oop.find(x => +f(x[1], 1414752632) === +slot)[1];
const ob = BigInt(f(s2, 1349481322)), mask = (1n << 20n) | (1n << 21n) | (1n << 22n);
console.log(`slot ${slot}: IP role ${f(s1, 1698852728)} | OOP role ${(ob & ~mask).toString()} (raw ${ob})`);
