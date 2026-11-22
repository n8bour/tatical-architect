// List recruitment focuses (id + name). node dev/_focuslist.mjs
import { call } from '../fmb-node.mjs';
const S = '825307168=97;1198877028=1;1348620377=17439426;1769566810=402653436';
for (const p of [1145779256, 1953983593]) {
  const r = JSON.parse((await call(`READ FM.UI.ScoutingInfoReference|${S}|${p}`)).slice(3).replace(/[\u0000-\u001f]/g, ' '));
  const ids = (Array.isArray(r) ? r : []).map(x => x[1] && x[1].pairs ? x[1].pairs.find(q => q[0] === 1919314276)?.[1] : null).filter(x => x != null);
  if (!ids.length) { console.log(p, JSON.stringify(r).slice(0, 150)); continue; }
  const names = (await call('READ ' + ids.map(i => `FM.UI.RecruitmentFocusItemReference|825307168=85;1198877028=1;1348620377=17439426;1769566810=402653436;1919314276=${i}|1851878757`).join(' ;; '))).slice(3).split('\u001e');
  console.log(p, ids.map((i, k) => `${i}=${names[k]}`).join(' | '));
}
