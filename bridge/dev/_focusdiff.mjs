// send one event to recruitment focus item -1 and show which properties changed. node dev/_focusdiff.mjs <ev> <value>
import { call } from '../fmb-node.mjs';
const P = '825307168=85;1198877028=1;1348620377=17439426;1769566810=402653436;1919314276=-1';
const ps = '1113732427 1164531812 1182102576 1229407795 1231700801 1245930093 1298483809 1333342809 1345542742 1346584898 1347436866 1366773845 1394625906 1397965364 1430999649 1432896057 1447777393 1449617238 1500468303 1633973107 1682798689 1750627946 1751477554 1765357145 1768829306 1785744201 1785818164 1785869637 1802126202 1817143629 1850636904 1851878757 1865630771 1902275159 1934782831 1934840397 1952526706 1984185648 1986161206 2021281094 540310070 860242543 913788738 929909872'.split(' ');
const snap = async () => (await call('READ ' + ps.map(p => `FM.UI.RecruitmentFocusItemReference|${P}|${p}`).join(' ;; '), 30000)).slice(3).split('\u001e');
const [ev, val] = process.argv.slice(2);
const a = await snap();
console.log(await call(`SENDJ ${ev} FM.UI.RecruitmentFocusItemReference ${P} ["SI.Core.DynamicNumber","${val}"]`));
await new Promise(r => setTimeout(r, 700));
const b = await snap();
ps.forEach((p, i) => { if (a[i] !== b[i]) console.log(p, '\n   before', a[i].slice(0, 300), '\n   after ', b[i].slice(0, 300)); });
