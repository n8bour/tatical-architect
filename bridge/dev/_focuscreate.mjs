// Create a recruitment focus (scout assignment) from the bridge. node dev/_focuscreate.mjs
import { call } from '../fmb-node.mjs';
const P = '825307168=85;1198877028=1;1348620377=17439426;1769566810=402653436;1919314276=-1';
const sleep = ms => new Promise(r => setTimeout(r, ms));
const W = (prop, json) => call(`WRITE FM.UI.RecruitmentFocusItemReference|${P}|${prop} ${JSON.stringify(json)}`);
const E = (ev, json) => call(`SENDJ ${ev} FM.UI.RecruitmentFocusItemReference ${P} ${JSON.stringify(json)}`);
const R = async (prop) => JSON.parse((await call(`READ FM.UI.RecruitmentFocusItemReference|${P}|${prop}`)).slice(3).replace(/[\u0000-\u001f]/g, ' '));
const num = (n) => ({ type: 'SI.Bindable.DynamicReference', entries: [[1886680684, 'SI.Core.DynamicNumber', String(n)], [1903577654, 'System.String', String(n)]] });
async function stepTo(prop, ev, target) {               // ability filters move in half-star steps: value = 10 + 2 × stars
  for (let i = 0; i < 12; i++) {
    const v = +(await R(prop)).entries.find(e => e[0] === 1298233430)[2];
    if (v === target) return v;
    await E(ev, ['SI.Core.DynamicNumber', v < target ? '1' : '0']); await sleep(250);
  }
}
console.log('ages', await W(1394625906, num(15)), await W(929909872, num(19)));
console.log('CA ->', await stepTo(1346584898, 1517250124, 12), ' PA ->', await stepTo(1347436866, 1431517038, 18));
console.log('area', await W(1817143629, { type: 'SI.Bindable.DynamicReference', entries: [[1903577654, 'System.String', 'South America (North)'], [1886680684, 'SI.Bindable.DynamicReference', { type: 'SI.Bindable.DynamicReference', entries: [[1685218662, 'SI.Core.DynamicNumber', '318767126']] }]] }), await E(0, null));
await sleep(400);
console.log('name', await W(1851878757, ['System.String', 'TB Wonderkids']));
await sleep(400);
for (const [k, p] of [['name', 1851878757], ['areas', 1986161206], ['ca', 1346584898], ['pa', 1347436866], ['min', 1394625906], ['max', 929909872]]) console.log(' ', k, JSON.stringify(await R(p)).slice(0, 220));
if (process.argv[2] === 'confirm') console.log('confirm', await E(55, null));
