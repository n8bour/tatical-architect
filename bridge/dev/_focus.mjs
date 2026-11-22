// Recruitment focus (scout assignment) item -1 = the one being created. node dev/_focus.mjs read | write
import { call } from '../fmb-node.mjs';
const P = '825307168=85;1198877028=1;1348620377=17439426;1769566810=402653436;1919314276=-1';
const props = { minAge: 1394625906, maxAge: 929909872, ca: 1346584898, pa: 1347436866, region: 1817143629, scouts: 1785818164, a: 1449617238, b: 1366773845, c: 1682798689, name: 1851878757 };
const short = (v) => Array.isArray(v) ? '[' + v.map(x => short(x[1])).join(', ') + ']' : v && v.entries ? '{' + v.entries.map(e => e[0] + ':' + short(e[2])).join(', ') + '}' : v && v.pairs ? 'ref' + v.pairs.at(-1)[1] : JSON.stringify(v);
async function read(at = '') {
  const ks = Object.keys(props);
  const r = (await call((at ? `READAT ${at} ` : 'READ ') + ks.map(k => `FM.UI.RecruitmentFocusItemReference|${P}|${props[k]}`).join(' ;; '))).slice(3).split('\u001e');
  ks.forEach((k, i) => { let v; try { v = short(JSON.parse(r[i].replace(/[\u0000-\u001f]/g, ' '))); } catch (e) { v = r[i].slice(0, 80); } console.log(' ', k.padEnd(7), v.slice(0, 160)); });
}
const W = (k, json) => call(`WRITE FM.UI.RecruitmentFocusItemReference|${P}|${props[k]} ${JSON.stringify(json)}`);
const num = (n) => ({ type: 'SI.Bindable.DynamicReference', entries: [[1886680684, 'SI.Core.DynamicNumber', String(n)], [1903577654, 'System.String', String(n)]] });
const range = (a, b) => ({ type: 'SI.Bindable.DynamicReference', entries: [[1298233430, 'SI.Core.DynamicNumber', String(a)], [1298755158, 'SI.Core.DynamicNumber', String(b)], [1397715049, 'System.Boolean', 'False']] });
if (process.argv[2] === 'write') {
  console.log(await W('minAge', num(16)), await W('maxAge', num(19)), await W('ca', range(40, 200)), await W('pa', range(140, 200)));
  await new Promise(r => setTimeout(r, 800));
}
console.log('read (root):'); await read();
