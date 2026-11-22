import { call } from '../fmb-node.mjs';
const P = '825307168=93;1198877028=1;1348620377=17439426;1685218662=402653436';
const sleep = ms => new Promise(r => setTimeout(r, ms));
export async function state() {
  const t = (await call(`READAT auto FM.UI.SetPieceReference|${P}|1867869768 ;; FM.UI.SetPieceReference|${P}|1668375608 ;; FM.UI.SetPieceReference|${P}|1279880290`)).slice(3).split('\u001e').map(x => JSON.parse(x.replace(/[\u0000-\u001f]/g, ' ')));
  const g = k => t[0].entries.find(e => e[0] == k)[2];
  return `routine kind ${g(1398883404)} att/def ${g(1919054668)} | tab ${t[1].entries.map(e => e[2]).join(' ')} | 1279880290=${t[2]}`;
}
export const wui = (prop, json) => call(`WRITEUI FM.UI.SetPieceReference|-|${prop} ${JSON.stringify(json)}`);
export const tab = (n, l) => wui(1668375608, { type: 'SI.Bindable.DynamicReference', entries: [[1886680684, 'SI.Core.DynamicNumber', String(n)], [1903577654, 'System.String', l]] });
if (process.argv[2] === 'run') {
  console.log(await state());
  console.log(await tab(1, 'Shooting Range')); await sleep(1500); console.log(await state());
  console.log(await tab(2, 'Attacking Third')); await sleep(1500); console.log(await state());
  console.log(await tab(0, 'Corners')); await sleep(1500); console.log(await state());
}
