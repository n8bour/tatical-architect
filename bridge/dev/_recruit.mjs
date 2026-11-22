// End-to-end test of the app's recruitment bridge code (FMB.findPerson / pushShortlist / createFocus / focusList).
// node dev/_recruit.mjs   — adds Yilver Riascos to the shortlist, creates "TB e2e" focus, then DELETES that focus.
import { FMB, FMB_IDS, call } from '../fmb-node.mjs';
const c = await FMB.context();
console.log('findPerson Yilver Riascos:', await FMB.findPerson('Yilver Riascos', '2000095557'));
console.log('findPerson wrong unique_id:', await FMB.findPerson('Yilver Riascos', '123'));
console.log('pushShortlist:', JSON.stringify(await FMB.pushShortlist([{ name: 'Yilver Riascos', uniq: '2000095557' }, { name: 'Nobody Atall', uniq: '1' }], (t) => console.log('  ', t))));
const made = await FMB.createFocus({ name: 'TB e2e', ageMin: 16, ageMax: 20, caStars: 1.5, paStars: 4, areas: [{ id: FMB_IDS.focus.nation(191), label: 'Colombia' }, { id: FMB_IDS.focus.nation(170), label: 'Spain' }] }, (t) => console.log('  ', t));
console.log('created:', made);
const P = FMB_IDS.focus.pairs(c, made.id), F = FMB_IDS.focus;
const r = await FMB.read([[ 'RecruitmentFocusItemReference', P, F.ageMin ], [ 'RecruitmentFocusItemReference', P, F.ageMax ], [ 'RecruitmentFocusItemReference', P, F.ca ], [ 'RecruitmentFocusItemReference', P, F.pa ], [ 'RecruitmentFocusItemReference', P, F.areas ]]);
console.log('read back: age', FMB.field(r[0], 1886680684), '-', FMB.field(r[1], 1886680684), '· CA', FMB.field(r[2], F.level), '· PA', FMB.field(r[3], F.level), '· areas', FMB.items(r[4]).length);
if (!process.argv.includes('--keep')) { await FMB.send(F.remove, 'ScoutingInfoReference', F.info(c), ['SI.Core.DynamicNumber', String(made.id)]); await new Promise(r => setTimeout(r, 700)); console.log('after delete:', (await FMB.focusList(c)).map(f => f.name).join(' | ')); }
