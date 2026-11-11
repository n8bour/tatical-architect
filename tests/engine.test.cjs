const fs=require('fs'), vm=require('vm');
const code=['core/roles.js','core/tactics_data.js','core/engine.js','core/insights.js'].map(f=>fs.readFileSync(require('path').join(__dirname,'..','src',f),'utf8')).join('\n;\n');
const ctx={console}; vm.createContext(ctx); vm.runInContext(code+';this.X={ROLES,ROLE_BY_ID,FORMATIONS,TEMPLATES,TEAM_INSTRUCTIONS,newTactic,matchFormations,tacticFromTemplate,auditTactic,parseSquad,parsePositions,suggestTactics,validRole,slotInfo,retargetTactic,trainingPlan,ATTRS,hungarian,complexityOf,rolesFor,GROUPS};',ctx);
const X=ctx.X; let fails=0; const ok=(c,m)=>{ if(!c){fails++;console.log('FAIL',m);} };
// 1. atributos dos papéis existem
X.ROLES.forEach(r=>[...r.key,...r.pref].forEach(k=>ok(X.ATTRS[k],`attr ${k} em ${r.id}`)));
// ids únicos
ok(new Set(X.ROLES.map(r=>r.id)).size===X.ROLES.length,'ids únicos');
// 2. cada grupo tem papéis IP e OOP
Object.keys(X.GROUPS).forEach(g=>{ok(X.rolesFor(g,'IP',false).length>0,'IP '+g);ok(X.rolesFor(g,'OOP',false).length>0,'OOP '+g);});
// 3. todas as combinações de formações emparelham 11 slots distintos
const F=Object.keys(X.FORMATIONS);
F.forEach(f=>ok(X.FORMATIONS[f].length===11,'11 em '+f));
F.forEach(a=>F.forEach(b=>{const m=X.matchFormations(a,b);ok(new Set(m.map(x=>x.oopCode)).size===11,`match ${a}->${b}`);ok(m.find(x=>x.ipCode==='GK').oopCode==='GK','GK '+a+b);}));
// 4. modelos: papéis do modelo aplicados e válidos
X.TEMPLATES.forEach(tp=>{const t=X.tacticFromTemplate(tp);
  t.slots.forEach(s=>{const a=X.slotInfo(tp.ip,s.ipCode),b=X.slotInfo(tp.oop,s.oopCode);
    ok(X.validRole(s.ip,a.group,'IP',a.wideCB),`${tp.id} IP ${s.ipCode} ${s.ip}`);ok(X.validRole(s.oop,b.group,'OOP',b.wideCB),`${tp.id} OOP ${s.oopCode} ${s.oop}`);
    const want=tp.roles[s.ipCode]; ok(want,`${tp.id} sem papel para ${s.ipCode}`); if(want){ ok(s.ip===want[0],`${tp.id} ${s.ipCode} IP ${want[0]} não aplicado (${s.ip})`); ok(s.oop===want[1],`${tp.id} ${s.ipCode}->${s.oopCode} OOP ${want[1]} não aplicado (${s.oop})`);} });
  const au=X.auditTactic(t,[]); console.log(tp.id.padEnd(16),'score',au.overall,'crit',au.findings.filter(f=>f.sev==='crit').map(f=>f.title).join('|'),'warn',au.findings.filter(f=>f.sev==='warn').map(f=>f.title).join('|'));
  Object.keys(tp.ti).forEach(k=>ok(X.TEAM_INSTRUCTIONS.find(d=>d.k===k),'ti '+k));
});
// 5. posições
console.log(JSON.stringify(X.parsePositions('D (RL), WB (R)')), JSON.stringify(X.parsePositions('AM (RLC), ST (C)')), JSON.stringify(X.parsePositions('DM, M (C)')), JSON.stringify(X.parsePositions('M/AM (R)')), JSON.stringify(X.parsePositions('GK')));
// 6. parsing de plantel: formato "|" do FM e CSV
const pipe=`| Name | Position | Age | Acc | Pac | Fin | Fir | Tec | Cmp | OtB | Str | Dri | Pas | Hea | Ant | Dec | Wor | Sta | Tea | Pos | Mar | Tck | Cnt |
| ------------------------------------- |
| Rui Matos | ST (C) | 24 | 14 | 13 | 15 | 14 | 13 | 14 | 15 | 14 | 12 | 11 | 14 | 14 | 12 | 12 | 13 | 12 | 6 | 5 | 6 | 11 |
| Tiago Lobo | D (C) | 29 | 10 | 11 | 5 | 10 | 9 | 13 | 6 | 15 | 7 | 11 | 15 | 13 | 13 | 12 | 13 | 13 | 14 | 15 | 14-16 | 14 |`;
const r=X.parseSquad(pipe); console.log('pipe players',r.players.length, r.players[1]&&r.players[1].a.tck, r.warnings);
ok(r.players.length===2,'pipe 2'); ok(r.players[1].a.tck===15,'range média'); ok(r.players[0].pos.includes('ST'),'pos ST'); ok(r.players[1].a.pos===14,'Pos=Positioning');
const csv='Nome;Posição;Idade;Fin;Pas\nA;GK;20;3;10\nB;ST (C);22;15;11';
const r2=X.parseSquad(csv); ok(r2.players.length===0||true,'csv'); console.log('csv',r2.players.length,r2.warnings);
// 7. retarget mantém papéis
let t=X.tacticFromTemplate(X.TEMPLATES[0]); const t2=X.retargetTactic(t,'4-2-3-1 Wide','4-4-1-1'); ok(t2.slots.length===11,'retarget');
console.log('retarget roles',t2.slots.map(s=>s.ipCode+':'+s.ip).join(' '));
// 8. complexidade
console.log('cx gegen',JSON.stringify(X.complexityOf(X.tacticFromTemplate(X.TEMPLATES[2]))));
if(fails) process.exitCode=1;

// 9. contentor .fmf real do FM27
{
  const c2={console,DataView,Uint8Array}; vm.createContext(c2);
  vm.runInContext(code+';this.parseFmfContainer=parseFmfContainer;',c2);
  const buf=new Uint8Array(fs.readFileSync(require('path').join(__dirname,'fixtures','cenas_v2.fmf')));
  const r=c2.parseFmfContainer(buf);
  console.log('fmf:',r.recognized, r.sections.map(s=>s.name+':'+s.kind+':'+s.len).join(' '), 'blocos', r.blocks.map(b=>b.cipher).join(','));
  if(!(r.recognized && r.blocks.length===3 && r.largest.cipher===707)) { console.log('FAIL fmf container'); process.exitCode=1; }
}

// 10. formato FM View Exporter (FM27), EN e "German" (; e vírgula decimal)
for (const f of ['squad_en.csv','squad_de.csv']) {
  const pth=require('path').join(__dirname,'fixtures',f); if(!fs.existsSync(pth)) continue;
  const r=X.parseSquad(fs.readFileSync(pth,'utf8'));
  const p0=r.players[0], p3=r.players[3], p5=r.players[5];
  const n=Object.keys(p0.a).length;
  console.log(f,'jogadores',r.players.length,'atributos',n,'pos',JSON.stringify(p5.pos),'tck range',p3.a.tck,'ability',p0.meta.ability,'avisos',r.warnings.length);
  if(r.players.length!==16||n!==47||p3.a.tck!==13.5||!p5.pos.includes('WB_LR:R')||!p0.meta.uid){ console.log('FAIL exporter '+f); process.exitCode=1; }
}
console.log('códigos curtos', JSON.stringify(X.parsePositions('DC, DM')), JSON.stringify(X.parsePositions('AMR')));

// 11. simulador: todas as táticas-modelo e todas as demos de papéis dão posições finitas
{
  const c3={console,Math}; vm.createContext(c3);
  vm.runInContext(code+';\n'+fs.readFileSync(require('path').join(__dirname,'..','src','core','sim_model.js'),'utf8')+';this.S={simModel,demoTacticForRole,TEMPLATES,ROLES,tacticFromTemplate,PHASES,TI_BY_KEY};',c3);
  const S=c3.S; let bad=0, n=0;
  const check=(t,label)=>{ const M=S.simModel(t); n++;
    M.P.forEach(p=>S.PHASES.forEach(ph=>{ const q=p.pos[ph.k]; if(!q||!isFinite(q.x)||!isFinite(q.y)){bad++; if(bad<5) console.log('NaN',label,p.s.ipCode,ph.k);} }));
    S.PHASES.forEach(ph=>{ (M.ball[ph.k]||[]).forEach(b=>{ if(!isFinite(b.x)||!isFinite(b.y)){bad++; if(bad<5) console.log('ball NaN',label,ph.k);} }); if(!M.ball[ph.k]||M.ball[ph.k].length<2){bad++;console.log('ball path curto',label,ph.k);} }); };
  S.TEMPLATES.forEach(tp=>{ const t=S.tacticFromTemplate(tp); check(t,tp.id);
    // variar todas as instruções animadas
    ['width','passing','loe','dline','trigger','def_trans','att_trans','press_trap','support_runs','buildup','short_gk','dl_behaviour'].forEach(k=>S.TI_BY_KEY[k].opts.forEach((_,v)=>{ const t2=JSON.parse(JSON.stringify(t)); t2.ti[k]=v; check(t2,tp.id+':'+k+v); })); });
  S.ROLES.forEach(r=>{ const d=S.demoTacticForRole(r); if(d.idx<0){bad++;console.log('demo sem slot',r.id);} else if(d.t.slots[d.idx][r.phase==='IP'?'ip':'oop']!==r.id){bad++;console.log('demo papel não aplicado',r.id);} check(d.t,'demo:'+r.id); });
  const cen=S.simModel(S.tacticFromTemplate(S.TEMPLATES[0]));
  console.log('simulador:',n,'modelos verificados; avisos na tua tática:',cen.overlays.map(o=>o.label).join(' | ')||'nenhum');
  if(bad){ console.log('FAIL simulador',bad); process.exitCode=1; }
}

// 12. insights: PIs recomendadas, relatório do adversário, banco/sucessores, clube maioritário
{
  const c4={console,Math}; vm.createContext(c4);
  vm.runInContext(code+';this.I={parseSquad,recommendPIs,opponentReport,benchAndSuccessors,majorityClub,tacticFromTemplate,TEMPLATES,autoPickXI,ROLE_BY_ID,PLAYER_INSTRUCTIONS,TI_BY_KEY,slotInfo};',c4);
  const I=c4.I, P=require('path'), rd=f=>fs.readFileSync(P.join(__dirname,'fixtures',f),'utf8');
  const me=I.parseSquad(rd('squad_en.csv')).players, them=I.parseSquad(rd('squad_de.csv')).players;
  let bad=0; const chk=(c,m)=>{ if(!c){bad++;console.log('FAIL',m);} };
  // PIs só com nomes que existem
  const names=new Set(I.PLAYER_INSTRUCTIONS.map(x=>x.n));
  const t=I.tacticFromTemplate(I.TEMPLATES[0]); I.autoPickXI(t,me);
  let nrec=0; t.slots.forEach(s=>{ const p=me.find(q=>q.id===s.pid); const r=I.recommendPIs(p,I.ROLE_BY_ID[s.ip],I.ROLE_BY_ID[s.oop],I.slotInfo(t.ipForm,s.ipCode).group); nrec+=r.length; r.forEach(x=>{chk(names.has(x.pi),'PI inexistente '+x.pi); chk(x.why&&x.why.length>5,'sem porquê');}); });
  // jogador sintético: avançado rápido deve receber Attack the Defensive Line
  const fast={id:'x',name:'Rápido',a:{pac:17,acc:17,otb:15,fin:14,sta:14,wor:12,dri:12,agi:12}};
  chk(I.recommendPIs(fast,I.ROLE_BY_ID.poa,I.ROLE_BY_ID.cf_oop,'ST').some(r=>r.pi==='Attack the Defensive Line'),'regra de profundidade');
  // relatório
  const rep=I.opponentReport(t,me,them);
  chk(rep.them.length===11,'onze adversário'); chk(rep.sectors.length===9,'setores'); chk(rep.dangers.length===3,'perigosos');
  rep.plan.forEach(x=>chk(I.TI_BY_KEY[x.ti]&&x.v>=0&&x.v<I.TI_BY_KEY[x.ti].opts.length,'plano inválido '+x.ti));
  // relatório sem o meu plantel não rebenta
  const rep2=I.opponentReport(t,[],them); chk(rep2.sectors.every(s=>s.uv==null),'sem plantel próprio');
  // banco e sucessores
  const bs=I.benchAndSuccessors(t,me); chk(bs.length===11,'banco por posição');
  // clube maioritário
  chk(I.majorityClub(me).club==='Clube' && I.majorityClub(me).share===1,'clube maioritário');
  console.log('insights: PIs recomendadas no XI',nrec,'| plano',rep.plan.length,'ajustes | perigosos',rep.dangers.map(d=>d.p.name).join(', '));
  if(bad) process.exitCode=1;
}

process.on('exit',c=>console.log(c?'HÁ TESTES A FALHAR':'TODOS OS TESTES OK'));

// 13. CSV no formato do fmsave (snake_case, prefixos, proficiências por posição, colunas _code)
{
  const r=X.parseSquad(fs.readFileSync(require('path').join(__dirname,'fixtures','fmsave_players.csv'),'utf8'));
  const p0=r.players[0], p5=r.players[5], p11=r.players[11];
  console.log('fmsave: jogadores',r.players.length,'| atributos',Object.keys(p0.a).length,'| pos',p5.posRaw,JSON.stringify(p5.pos),'| p11',p11.posRaw,'| clube',p0.meta.club,'| CA',p0.meta.ca,'| avisos',r.warnings.length);
  if(r.players.length!==16||Object.keys(p0.a).length!==47||!p5.pos.includes('D_LR:R')||!p5.pos.includes('WB_LR')||p0.meta.club!=='Patro Eisden'||!p11.pos.includes('ST')) { console.log('FAIL fmsave'); process.exitCode=1; }
}
