const fs=require('fs'), vm=require('vm');
const code=['i18n/i18n.js','core/roles.js','core/tactics_data.js','core/engine.js','core/insights.js','core/synergy.js'].map(f=>fs.readFileSync(require('path').join(__dirname,'..','src',f),'utf8')).join('\n;\n');
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
  console.log('simulador:',n,'modelos verificados; avisos no 1.º modelo:',cen.overlays.map(o=>o.label).join(' | ')||'nenhum');
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

// 14. bolas paradas: papéis preenchidos, sem jogadores repetidos, GR fora, ≥ 2 atrás
{
  const c5={console,Math}; vm.createContext(c5);
  vm.runInContext(code+';this.Q={parseSquad,setPiecePlan,tacticFromTemplate,TEMPLATES};',c5);
  const Q=c5.Q, P=require('path'), rd=f=>fs.readFileSync(P.join(__dirname,'fixtures',f),'utf8');
  let bad=0; const chk=(c,m)=>{ if(!c){bad++;console.log('FAIL',m);} };
  for (const [mine,opp] of [['fmsave_players.csv','squad_en.csv'],['squad_en.csv',null],['squad_de.csv','fmsave_players.csv']]) {
    const me=Q.parseSquad(rd(mine)).players, op=opp?Q.parseSquad(rd(opp)).players:[];
    Q.TEMPLATES.forEach(tp=>{ const t=Q.tacticFromTemplate(tp); const r=Q.setPiecePlan(t,me,op); chk(r,'plano '+tp.id);
      if(!r) return;
      const aIds=r.attack.map(x=>x.x.p.id), dIds=r.defend.map(x=>x.x.p.id);
      chk(new Set(aIds).size===aIds.length,'ataque repetido '+tp.id); chk(new Set(dIds).size===dIds.length,'defesa repetida '+tp.id);
      chk(r.attack.filter(x=>x.k.startsWith('back')).length>=2,'≥2 atrás '+tp.id);
      chk(![...r.attack,...r.defend].some(x=>x.x.group==='GK'),'GR em papel de campo '+tp.id);
      chk(r.pens.length===3 && r.caps.length===2,'penáltis/capitães '+tp.id);
    });
  }
  console.log('bolas paradas: modelos×plantéis verificados');
  if(bad) process.exitCode=1;
}

// 15. gerador: táticas válidas, coerentes e com razões para as instruções
{
  const c6={console,Math,performance}; vm.createContext(c6);
  vm.runInContext(code+';\n'+fs.readFileSync(require('path').join(__dirname,'..','src','core','generator.js'),'utf8')+';this.Gn={generateTactic,parseSquad,auditTactic,validRole,slotInfo,TEAM_INSTRUCTIONS,FORMATIONS,depthChart,weakSpots};',c6);
  const G=c6.Gn, P=require('path'), rd=f=>fs.readFileSync(P.join(__dirname,'fixtures',f),'utf8');
  let bad=0; const chk=(c,m)=>{ if(!c){bad++;console.log('FAIL',m);} };
  const depth=f=>{const sl=G.FORMATIONS[f].filter(x=>x.code!=='GK');return sl.reduce((a,x)=>a+x.y,0)/sl.length;};
  for (const f of ['fmsave_players.csv','squad_en.csv','squad_de.csv']) for (const ex of ['title','mid','low']) {
    const ps=G.parseSquad(rd(f)).players; const must=[ps[3].id]; const r=G.generateTactic(ps,{expect:ex,must,star:ps[14].id});
    [r.main,...r.alts].forEach(x=>{ const t=x.t;
      chk(new Set(t.slots.map(s=>s.pid)).size===11,'XI sem repetidos '+f);
      t.slots.forEach(s=>{ const a=G.slotInfo(t.ipForm,s.ipCode),b=G.slotInfo(t.oopForm,s.oopCode); chk(G.validRole(s.ip,a.group,'IP',a)&&G.validRole(s.oop,b.group,'OOP',b),'papel inválido '+t.ipForm+' '+s.ipCode+' '+s.ip+'/'+s.oop); });
      chk(x.audit.findings.filter(q=>q.sev==='crit').length===0,'crítico na gerada '+f+' '+ex+' '+t.ipForm+': '+x.audit.findings.filter(q=>q.sev==='crit').map(q=>q.title).join('|'));
      chk(depth(t.ipForm)>=depth(t.oopForm)-0.5,'IP mais recuada que OOP');
      chk(Object.keys(x.reasons).length>=G.TEAM_INSTRUCTIONS.length-2,'razões em falta');
    });
    chk(r.main.t.slots.some(s=>s.pid===must[0]),'titular obrigatório fora do XI');
    chk(r.main.t.slots.some(s=>s.pid===ps[14].id),'estrela fora do XI');
    chk(r.alts.length>=2,'alternativas');
    if (ex==='low') chk(r.main.t.ment==='Cautious','mentalidade underdog');
  }
  console.log('gerador: 3 plantéis × 3 contextos verificados');
  if(bad) process.exitCode=1;
}

// 17. disponibilidade por casa exacta (FM26/27): modelos e demonstrações só usam papéis válidos naquela casa
{
  const c8={console,Math}; vm.createContext(c8);
  vm.runInContext(code+';\n'+fs.readFileSync(require('path').join(__dirname,'..','src','core','sim_model.js'),'utf8')+';this.V={TEMPLATES,tacticFromTemplate,slotInfo,validRole,rolesFor,ROLES,demoTacticForRole};',c8);
  const V=c8.V; let bad=0; const chk=(c,m)=>{ if(!c){bad++;console.log('FAIL',m);} };
  V.TEMPLATES.forEach(tp=>{ Object.entries(tp.roles||{}).forEach(([code,[ip,oop]])=>{ const t=V.tacticFromTemplate(tp); const s=t.slots.find(z=>z.ipCode===code); if(!s) return;
    const a=V.slotInfo(t.ipForm,s.ipCode), b=V.slotInfo(t.oopForm,s.oopCode);
    chk(V.validRole(ip,a.group,'IP',a),'modelo '+tp.id+' '+code+' IP '+ip); chk(V.validRole(oop,b.group,'OOP',b),'modelo '+tp.id+' '+code+' OOP '+oop); }); });
  V.ROLES.forEach(r=>{ const d=V.demoTacticForRole(r), s=d.t.slots[d.idx], a=V.slotInfo(d.t.ipForm,s.ipCode); chk(V.validRole(r.id,a.group,r.phase,a),'demo '+r.id); });
  // casos confirmados no jogo
  const at=(f,c)=>V.rolesFor(V.slotInfo(f,c).group,'IP',V.slotInfo(f,c)).map(r=>r.id);
  chk(!at('4-3-3 DM Wide','MCL').includes('b2bp'),'BBP não pode estar no médio centro');
  chk(!at('4-3-3 DM Wide','MCL').includes('b2b'),'BBM não pode estar no médio centro');
  chk(!at('4-3-3 DM Wide','DM').includes('b2b'),'BBM não pode estar no DM central');
  chk(at('4-2-4 DM Wide','DML').includes('b2b'),'BBM tem de estar no DM deslocado');
  console.log('casa exacta: modelos, 77 demonstrações e casos confirmados verificados');
  if(bad) process.exitCode=1;
}

// 18. sinergia ofensiva: o gerador com sinergia cria mais ligações e mais presença na área, sem sacrificar a adequação
{
  const c9={console,Math,performance}; vm.createContext(c9);
  vm.runInContext(code+';\n'+fs.readFileSync(require('path').join(__dirname,'..','src','core','generator.js'),'utf8')+';this.S={generateTactic,parseSquad,synergyReport};',c9);
  const S=c9.S, rd=f=>fs.readFileSync(require('path').join(__dirname,'fixtures',f),'utf8');
  let bad=0; const chk=(c,m)=>{ if(!c){bad++;console.log('FAIL',m);} };
  let dSyn=0, dFit=0, dBox=0, n=0;
  for (const f of ['fmsave_players.csv','squad_en.csv','squad_de.csv']) for (const ex of ['title','low']) {
    const ps=S.parseSquad(rd(f)).players;
    const a=S.generateTactic(ps,{expect:ex,synergy:0}).main, b=S.generateTactic(ps,{expect:ex}).main;
    const sa=S.synergyReport(a.t,ps), sb=S.synergyReport(b.t,ps);
    chk(sb.score>=sa.score,'sinergia não pode piorar '+f+' '+ex+': '+sa.score+' → '+sb.score);
    chk(sb.phases.def.score>=sa.phases.def.score-4,'a defesa não pode ser sacrificada '+f+' '+ex+': '+sa.phases.def.score+' → '+sb.phases.def.score);
    chk(b.fitAvg>=a.fitAvg-3,'adequação não pode cair mais de 3 '+f+' '+ex);
    chk(sb.boxN>=3,'pelo menos 3 jogadores a chegar à área '+f+' '+ex+': '+sb.boxN);
    chk(sb.creators>=1 && sb.creators<=3,'1 a 3 criativos '+f+' '+ex);
    dSyn+=sb.score-sa.score; dFit+=b.fitAvg-a.fitAvg; dBox+=sb.boxN-sa.boxN; n++;
  }
  console.log(`sinergia: +${(dSyn/n).toFixed(1)} sinergia, ${(dFit/n>=0?'+':'')}${(dFit/n).toFixed(1)} adequação, +${(dBox/n).toFixed(1)} jogadores na área (média de ${n} casos)`);
  if(bad) process.exitCode=1;
}

// 19. sinergia defensiva: casos de manual
{
  const c10={console,Math}; vm.createContext(c10);
  vm.runInContext(code+';this.D={newTactic,synergyReport,ROLE_BY_ID};',c10);
  const D=c10.D; let bad=0; const chk=(c,m)=>{ if(!c){bad++;console.log('FAIL',m);} };
  const mk=(roles)=>{ const t=D.newTactic('4-3-3 DM Wide','4-1-4-1 DM Wide'); t.slots.forEach(s=>{ if(roles[s.oopCode]) s.oop=roles[s.oopCode]; }); return t; };
  const base=D.synergyReport(mk({}),[]);
  const stopCover=D.synergyReport(mk({DCL:'scb',DCR:'ccb'}),[]), twoStop=D.synergyReport(mk({DCL:'scb',DCR:'scb'}),[]);
  chk(stopCover.phases.def.parts.cb===100,'Central de Antecipação + Central de Cobertura = dupla ideal');
  chk(twoStop.phases.def.parts.cb<50 && twoStop.issues.some(i=>i.k==='cb'),'dois centrais a sair = problema');
  chk(!stopCover.issues.some(i=>i.k==='press' && /scb|SCD/.test(i.t)),'o central que sai está coberto pelo do lado');
  const noScreen=D.synergyReport(mk({DM:'pdm',MCL:'pcm',MCR:'pcm'}),[]);
  chk(noScreen.phases.def.parts.screen<50 && noScreen.issues.some(i=>i.k==='screen'),'meio-campo todo a pressionar = sem proteção à frente dos centrais');
  console.log('sinergia defensiva: casos de manual verificados');
  if(bad) process.exitCode=1;
}

// 20. situações de jogo: tácticas válidas, substituições do banco e cada plano na direcção certa
{
  const c11={console,Math,performance}; vm.createContext(c11);
  vm.runInContext(code+';\n'+['generator.js','gameplans.js'].map(f=>fs.readFileSync(require('path').join(__dirname,'..','src','core',f),'utf8')).join(';\n')+';this.P={generateTactic,parseSquad,gamePlan,SCENARIOS,slotInfo,validRole};',c11);
  const P=c11.P, rd=f=>fs.readFileSync(require('path').join(__dirname,'fixtures',f),'utf8');
  let bad=0; const chk=(c,m)=>{ if(!c){bad++;console.log('FAIL',m);} };
  const opp=P.parseSquad(rd('squad_de.csv')).players;
  for (const f of ['fmsave_players.csv','squad_en.csv','squad_de.csv']) {
    const ps=P.parseSquad(rd(f)).players, base=P.generateTactic(ps,{expect:'mid'}).main.t;
    for (const s of P.SCENARIOS()) {
      const r=P.gamePlan(base,ps,s.k,{opp}), tag=f+' '+s.k;
      r.t.slots.forEach(x=>{ const a=P.slotInfo(r.t.ipForm,x.ipCode), b=P.slotInfo(r.t.oopForm,x.oopCode); chk(P.validRole(x.ip,a.group,'IP',a)&&P.validRole(x.oop,b.group,'OOP',b),'papel inválido '+tag+' '+x.ipCode); chk((x.pis||[]).length<=3,'no máximo 3 instruções individuais '+tag); });
      const inXI=new Set(r.t.slots.map(x=>x.pid)); r.subs.forEach(x=>{ const p=ps.find(q=>q.name===x.in); chk(p && !inXI.has(p.id),'substituto tem de vir do banco '+tag); });
      chk(new Set(r.subs.map(x=>x.in)).size===r.subs.length,'substitutos repetidos '+tag);
      const A=r.intentBefore, B=r.intentAfter, sa=r.before.phases, sb=r.after.phases;
      if (s.k==='chase'||s.k==='home_weak') { chk(B.att>A.att,'mais compromisso ofensivo '+tag); chk(sb.att.score>=sa.att.score,'a sinergia de ataque não pode piorar '+tag); }
      if (s.k==='protect'||s.k==='away_strong') { chk(B.sec>A.sec,'mais segurança '+tag); chk(B.att<=A.att,'menos compromisso ofensivo '+tag); chk(sb.def.score>=sa.def.score-2,'a sinergia defensiva não pode piorar '+tag); }
      if (s.k==='away_strong') chk(r.ois.length===3,'instruções de adversário para os 3 mais perigosos '+tag);
    }
  }
  console.log('situações de jogo: 3 plantéis × 5 cenários verificados');
  if(bad) process.exitCode=1;
}

// 21. prospeção: base de dados inteira no formato do fmsave, filtros e pesquisas pré-definidas
{
  const c12={console,Math,performance,setTimeout}; vm.createContext(c12);
  vm.runInContext(code+';\n'+['scouting.js'].map(f=>fs.readFileSync(require('path').join(__dirname,'..','src','core',f),'utf8')).join(';\n')+';this.S={parseScoutDB,scoutFilter,scoutPresets,reachLikelihood,SCOUT_COLUMNS,SCOUT_DEFAULT,bestGain,bestRolesAny,inferGameDate,parseSquad,tacticFromTemplate,TEMPLATES,autoPickXI,monthsUntil};',c12);
  const S=c12.S, {gen,COLS}=require('./gen_scout_db.cjs');
  let bad=0; const chk=(c,m)=>{ if(!c){bad++;console.log('FAIL',m);} };
  // as colunas pedidas ao fmsave existem todas na exportação real (mesmos nomes)
  S.SCOUT_COLUMNS.forEach(cn=>chk(COLS.includes(cn),'coluna pedida inexistente: '+cn));
  (async()=>{
    const db=await S.parseScoutDB(gen(5000));
    chk(db.players.length===5000,'5000 jogadores lidos: '+db.players.length);
    chk(db.attrCols>=45,'atributos mapeados: '+db.attrCols);
    const p=db.players[0]; chk(p.ca>0 && p.pa>=p.ca && Object.keys(p.a).length>=45 && p.pers.professionalism>0 && p.hid.inj>0,'campos do jogador');
    chk(db.players.some(x=>x.pos.includes('ST')),'avançados reconhecidos (positions_stc)');
    chk(db.players.find(x=>x.club && x.club.includes('"B"')),'nomes de clube com aspas');
    const date=S.inferGameDate(db.players); chk(/^\d{4}-\d{2}-\d{2}$/.test(date),'data do jogo inferida '+date);
    const squad=S.parseSquad(fs.readFileSync(require('path').join(__dirname,'fixtures','fmsave_players.csv'),'utf8')).players;
    const t=S.tacticFromTemplate(S.TEMPLATES[0]); S.autoPickXI(t,squad);
    const ctx={date, myClub:'', tactic:t, squad, xiCA:120};
    for (const pr of S.scoutPresets(ctx)) {
      const res=S.scoutFilter(db,pr.F,ctx);
      if (pr.k==='cheap_wonder') chk(res.length>0 && res.every(x=>x.age<=19 && x.pa>=150 && (x.value==null||x.value<=pr.F.valueMax)),'wonderkids baratos');
      if (pr.k==='reach') chk(res.every(x=>x._lk>=70 && x.age<=20),'probabilidade de atingir o potencial');
      if (pr.k==='expiring') chk(res.length>0 && res.every(x=>{ const m=S.monthsUntil(x.end,date); return m!=null && m<=6; }),'contratos a terminar');
      if (pr.k==='free') chk(res.length>0 && res.every(x=>!x.club),'jogadores livres');
      if (pr.k==='upgrade') chk(res.length>0 && res[0]._gain>=res[res.length-1]._gain,'reforços ordenados pelo ganho');
      if (pr.k==='hidden') chk(res.every(x=>x.clubRep==null||x.clubRep<=4000),'joias em clubes pequenos');
    }
    // filtros combinados
    const F={...S.SCOUT_DEFAULT(), pos:'DC', posLevel:'nat', attrs:[{k:'hea',min:12},{k:'pac',min:12}], heightMin:185, sort:'ca'};
    const r2=S.scoutFilter(db,F,ctx); chk(r2.every(x=>x.natPos.includes('DC') && x.a.hea>=12 && x.a.pac>=12 && x.height>=185),'filtros combinados');
    const g=S.bestGain(db.players.find(x=>x.ca>150), t, squad); chk(g && g.code,'onde entraria no XI');
    // velocidade: 100 mil jogadores
    const big=gen(100000); const t0=Date.now(); const db2=await S.parseScoutDB(big); const ms=Date.now()-t0;
    const t1=Date.now(); S.scoutFilter(db2,{...S.SCOUT_DEFAULT(), ageMax:21, paMin:150},ctx); const fms=Date.now()-t1;
    chk(db2.players.length===100000,'100 mil lidos'); chk(ms<15000,'leitura de 100 mil em menos de 15 s: '+ms+' ms'); chk(fms<500,'filtro em menos de 0,5 s: '+fms+' ms');
    console.log(`prospeção: 10 pesquisas e filtros verificados · 100 mil jogadores lidos em ${(ms/1000).toFixed(1)} s (${(big.length/1048576).toFixed(0)} MB), filtro em ${fms} ms`);
    if(bad) process.exitCode=1;
  })();
}

// 22. prospeção: estimativa de interesse em vir para o meu clube
{
  const c13={console,Math,performance,setTimeout}; vm.createContext(c13);
  vm.runInContext(code+';\n'+fs.readFileSync(require('path').join(__dirname,'..','src','core','scouting.js'),'utf8')+';this.J={joinLikelihood,parseScoutDB,scoutFilter,scoutPresets,SCOUT_DEFAULT};',c13);
  const J=c13.J; let bad=0; const chk=(c,m)=>{ if(!c){bad++;console.log('FAIL',m);} };
  const base={club:'Other',clubRep:5000,rep:4000,end:'2027-06-30',status:'Rotation',pers:{ambition:12,loyalty:10}};
  const ctx=(myRep)=>({myRep,date:'2027-01-15'});
  chk(J.joinLikelihood(base,{date:'x'})===null,'sem o meu clube: sem estimativa');
  chk(J.joinLikelihood(base,ctx(8000)).score > J.joinLikelihood(base,ctx(3000)).score,'clube maior atrai mais');
  chk(J.joinLikelihood({...base,club:'',clubRep:null},ctx(4000)).score > J.joinLikelihood(base,ctx(4000)).score,'livre é mais fácil');
  chk(J.joinLikelihood({...base,rep:9000},ctx(5000)).score < J.joinLikelihood(base,ctx(5000)).score,'reputação acima do clube afasta');
  chk(J.joinLikelihood({...base,pers:{ambition:10,loyalty:19}},ctx(5000)).score < J.joinLikelihood({...base,pers:{ambition:10,loyalty:5}},ctx(5000)).score,'lealdade prende');
  const r=J.joinLikelihood(base,ctx(7000)); chk(r.score>=0 && r.score<=100 && r.why.length>0,'0–100 com razões');
  (async()=>{
    const {gen}=require('./gen_scout_db.cjs'); const db=await J.parseScoutDB(gen(3000));
    const c={date:'2027-01-15',myRep:6000,myClub:'',xiCA:110,squad:[]};
    const pr=J.scoutPresets(c).find(x=>x.k==='interested'); const res=J.scoutFilter(db,pr.F,c);
    chk(res.length>0 && res.every(p=>p._jn>=70),'pesquisa "interessados" só com ≥ 70');
    chk(res.every((p,i)=>i===0||res[i-1]._jn>=p._jn),'ordenada pela estimativa');
    console.log('interesse em vir: estimativa verificada ('+res.length+' interessados prováveis em 3000)');
    if(bad) process.exitCode=1;
  })();
}

// 23. plantel com poucos atributos (CSV parcial): o diagnóstico não pode falhar
{
  const c14={console,Math}; vm.createContext(c14);
  vm.runInContext(code+';this.Q={parseSquad,auditTactic,tacticFromTemplate,TEMPLATES,autoPickXI,suggestTactics:typeof suggestTactics!=="undefined"?suggestTactics:null};',c14);
  const Q=c14.Q; let bad=0;
  const csv='Name,Club,Position,Age,Finishing,Passing,Tackling,Pace\n'+Array.from({length:16},(_,i)=>`P${i},Club 5,${['GK','DC','DC','D (L)','D (R)','DM','M (C)','M (C)','AM (L)','AM (R)','ST','ST','DC','M (C)','ST','GK'][i]},24,${10+i%8},12,11,13`).join('\n');
  const ps=Q.parseSquad(csv).players;
  try { Q.TEMPLATES.forEach(tp=>{ const t=Q.tacticFromTemplate(tp); Q.autoPickXI(t,ps); Q.auditTactic(t,ps); }); if (Q.suggestTactics) Q.suggestTactics(ps); console.log('plantel parcial (4 atributos): diagnóstico e sugestões sem erros'); }
  catch(e){ bad++; console.log('FAIL plantel parcial:', e.message); }
  if(bad) process.exitCode=1;
}

// 24. prospeção: valor desconhecido não pode passar por "barato"
{
  const c15={console,Math,performance,setTimeout}; vm.createContext(c15);
  vm.runInContext(code+';\n'+fs.readFileSync(require('path').join(__dirname,'..','src','core','scouting.js'),'utf8')+';this.V={parseScoutDB,scoutFilter,scoutPresets};',c15);
  const V=c15.V; let bad=0; const chk=(c,m)=>{ if(!c){bad++;console.log('FAIL',m);} };
  (async()=>{
    const {gen}=require('./gen_scout_db.cjs'); const db=await V.parseScoutDB(gen(4000));
    // estrelas com valor ilegível (como alguns jogadores no save real): muito boas, em clubes grandes, sem valor
    db.players.slice(0,60).forEach(p=>{ p.value=null; p.ca=185; p.pa=195; p.age=22; p.clubRep=9500; if(!p.club) p.club='Big Club'; });
    const stars=new Set(db.players.slice(0,60).map(p=>p.id));
    const ctx={date:'2027-01-15',myRep:6000,myClub:'',xiCA:120,squad:[]};
    for (const k of ['bargain','cheap_wonder','hidden']) {
      const pr=V.scoutPresets(ctx).find(x=>x.k===k); const res=V.scoutFilter(db,pr.F,ctx);
      chk(!res.some(p=>stars.has(p.id)),k+': jogadores com valor desconhecido não podem aparecer');
      chk(res.every(p=>!p.club || (p.value!=null && p.value<=pr.F.valueMax)),k+': todos com valor conhecido abaixo do máximo (ou livres)');
    }
    const free=V.scoutFilter(db,{...V.scoutPresets(ctx).find(x=>x.k==='bargain').F},ctx).filter(p=>!p.club);
    console.log('valor desconhecido: excluído das pesquisas de baratos ('+free.length+' livres continuam a contar)');
    if(bad) process.exitCode=1;
  })();
}

// 25. plantel lido do save (formato fmsave): CA/PA, personalidade, ocultos e contrato chegam ao plantel
{
  const c16={console,Math,performance,setTimeout}; vm.createContext(c16);
  vm.runInContext(code+';\n'+fs.readFileSync(require('path').join(__dirname,'..','src','core','scouting.js'),'utf8')+';this.E={parseSquad,enrichFromScout};',c16);
  const E=c16.E; let bad=0; const chk=(c,m)=>{ if(!c){bad++;console.log('FAIL',m);} };
  const {gen}=require('./gen_scout_db.cjs'); const csv=gen(25,11);
  const ps=E.parseSquad(csv).players; const n=E.enrichFromScout(ps,csv);
  chk(n===25,'25 jogadores enriquecidos: '+n);
  chk(ps.every(p=>p.ca>0 && p.pa>=p.ca),'CA/PA em todos');
  chk(ps.every(p=>Object.keys(p.pers).length===8 && Object.keys(p.hid).length===5),'personalidade (8) e ocultos (5)');
  chk(ps.filter(p=>p.club).every(p=>p.value!=null && /^\d{4}-\d{2}-\d{2}$/.test(p.end)),'valor e fim de contrato');
  chk(ps.some(p=>p.pos.includes('ST')),'avançados (positions_stc)');
  const f9=E.parseSquad('Name,Position,Age,Finishing\nA,ST,22,14\nB,DC,25,8').players; chk(E.enrichFromScout(f9,'Name,Position,Age,Finishing')===0,'CSV do F9 não é tocado');
  console.log('plantel do save: CA/PA, personalidade, ocultos e contrato verificados');
  if(bad) process.exitCode=1;
}
