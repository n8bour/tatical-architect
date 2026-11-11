// FM Tactical Architect - explainable tactical engine
const ROLE_DB = [
  {name:'Ball-Playing Goalkeeper',phase:'IP',groups:['GK'],traits:['build-up','distribution','sweeper'],attrs:['First Touch','Passing','Composure','Decisions','Kicking'],risk:['poor passing','poor composure']},
  {name:'No-Nonsense Goalkeeper',phase:'IP',groups:['GK'],traits:['security','direct'],attrs:['Handling','Reflexes','One on Ones','Kicking'],risk:['limited build-up']},
  {name:'Centre-Back',phase:'IP',groups:['CB'],traits:['security','progression-neutral'],attrs:['Marking','Tackling','Positioning','Anticipation','Strength'],risk:['slow recovery']},
  {name:'Overlapping Centre-Back',phase:'IP',groups:['CB'],traits:['overlap','progression','aggressive'],attrs:['Pace','Acceleration','Dribbling','Passing','Off the Ball'],risk:['space behind']},
  {name:'Inside Wing-Back',phase:'IP',groups:['FB','WB'],traits:['infield','midfield-overload','cover'],attrs:['Passing','Decisions','Off the Ball','First Touch','Anticipation'],risk:['wide coverage']},
  {name:'Playmaking Wing-Back',phase:'IP',groups:['FB','WB'],traits:['wide-creator','progression','overlap'],attrs:['Crossing','Passing','Vision','Decisions','Technique'],risk:['counter exposure']},
  {name:'Advanced Wing-Back',phase:'IP',groups:['FB','WB'],traits:['width','depth','aggressive'],attrs:['Acceleration','Pace','Crossing','Off the Ball','Dribbling'],risk:['space behind']},
  {name:'Full-Back',phase:'IP',groups:['FB','WB'],traits:['balance','width','security'],attrs:['Positioning','Marking','Crossing','Pace','Decisions'],risk:['lower attacking output']},
  {name:'Defensive Midfielder',phase:'IP',groups:['DM'],traits:['screen','security','cover'],attrs:['Positioning','Anticipation','Tackling','Passing','Decisions'],risk:['limited creativity']},
  {name:'Midfield Playmaker',phase:'IP',groups:['DM','CM'],traits:['creator','progression','link'],attrs:['Passing','Vision','First Touch','Decisions','Technique'],risk:['transition coverage']},
  {name:'Wide Central Midfielder',phase:'IP',groups:['CM'],traits:['half-space','support','mobility'],attrs:['Work Rate','Off the Ball','Passing','Decisions','Stamina'],risk:['central coverage']},
  {name:'Advanced Playmaker',phase:'IP',groups:['CM','AM'],traits:['creator','between-lines'],attrs:['Vision','Passing','First Touch','Technique','Decisions'],risk:['defensive output']},
  {name:'Winger',phase:'IP',groups:['W','AM'],traits:['width','depth','1v1'],attrs:['Pace','Acceleration','Dribbling','Crossing','Off the Ball'],risk:['inside congestion']},
  {name:'Inside Winger',phase:'IP',groups:['W','AM'],traits:['infield','half-space','combination'],attrs:['Dribbling','Passing','Off the Ball','Decisions','Acceleration'],risk:['width']},
  {name:'Inside Forward',phase:'IP',groups:['W','AM'],traits:['infield','box','runner'],attrs:['Off the Ball','Acceleration','Dribbling','Finishing','Composure'],risk:['width']},
  {name:'Wide Forward',phase:'IP',groups:['W','AM'],traits:['depth','attack-space','box'],attrs:['Pace','Acceleration','Off the Ball','Finishing','Dribbling'],risk:['build-up link']},
  {name:'Free Role',phase:'IP',groups:['AM'],traits:['freedom','creator','movement'],attrs:['Decisions','Vision','Off the Ball','Technique','First Touch'],risk:['structural discipline']},
  {name:'Second Striker',phase:'IP',groups:['AM','ST'],traits:['box','link','depth'],attrs:['Finishing','Off the Ball','Composure','Acceleration','First Touch'],risk:['midfield coverage']},
  {name:'Centre Forward',phase:'IP',groups:['ST'],traits:['depth','link','box'],attrs:['Finishing','Off the Ball','Composure','Acceleration','Strength'],risk:['role ambiguity']},
  {name:'Target Forward',phase:'IP',groups:['ST'],traits:['aerial','reference','hold-up'],attrs:['Strength','Jumping Reach','Heading','First Touch','Finishing'],risk:['slow transitions']},
  {name:'Poacher',phase:'IP',groups:['ST'],traits:['box','depth','finisher'],attrs:['Off the Ball','Finishing','Anticipation','Composure','Acceleration'],risk:['limited link play']},
  {name:'Deep-Lying Forward',phase:'IP',groups:['ST'],traits:['drop','link','creator'],attrs:['First Touch','Passing','Vision','Off the Ball','Decisions'],risk:['depth']},
  {name:'False Nine',phase:'IP',groups:['ST'],traits:['drop','creator','drag'],attrs:['First Touch','Passing','Vision','Off the Ball','Decisions'],risk:['no depth']},
  {name:'Channel Forward',phase:'IP',groups:['ST'],traits:['channels','stretch','depth'],attrs:['Acceleration','Pace','Off the Ball','Dribbling','Finishing'],risk:['central presence']},
  {name:'Pressing Forward',phase:'OOP',groups:['ST'],traits:['press','screen','transition'],attrs:['Work Rate','Stamina','Aggression','Acceleration','Anticipation'],risk:['technical dependency']},
  {name:'Tracking Winger',phase:'OOP',groups:['W','AM'],traits:['track','cover','press'],attrs:['Work Rate','Stamina','Pace','Anticipation','Positioning'],risk:['attacking energy']},
  {name:'Pressing Full-Back',phase:'OOP',groups:['FB','WB'],traits:['press','deny-width','recovery'],attrs:['Acceleration','Pace','Work Rate','Positioning','Tackling'],risk:['space inside']},
  {name:'Holding Full-Back',phase:'OOP',groups:['FB','WB'],traits:['hold','rest-defence','security'],attrs:['Positioning','Marking','Anticipation','Strength','Decisions'],risk:['attacking width']},
  {name:'Pressing Central Midfielder',phase:'OOP',groups:['CM','DM'],traits:['press','jump','cover'],attrs:['Work Rate','Stamina','Anticipation','Tackling','Acceleration'],risk:['space behind']},
  {name:'Screening Central Midfielder',phase:'OOP',groups:['CM','DM'],traits:['screen','protect','block'],attrs:['Positioning','Anticipation','Concentration','Tackling','Decisions'],risk:['pressing range']},
  {name:'Wide-Covering Central Midfielder',phase:'OOP',groups:['CM','DM'],traits:['wide-cover','shift','support'],attrs:['Stamina','Positioning','Work Rate','Anticipation','Pace'],risk:['central gap']},
  {name:'Covering Centre-Back',phase:'OOP',groups:['CB'],traits:['cover','depth','sweep'],attrs:['Pace','Acceleration','Anticipation','Positioning','Concentration'],risk:['front-foot duel']},
  {name:'Defending Centre-Back',phase:'OOP',groups:['CB'],traits:['duel','protect','security'],attrs:['Marking','Tackling','Positioning','Strength','Anticipation'],risk:['space behind']},
  {name:'Sweeper Keeper',phase:'OOP',groups:['GK'],traits:['sweep','support','depth'],attrs:['Acceleration','One on Ones','Anticipation','Composure','Kicking'],risk:['risk outside box']},
];

const FORMATIONS = {
 '4-3-3 DM Wide': {slots:[['GK','GK'],['RB','FB/WB'],['RCB','CB'],['LCB','CB'],['LB','FB/WB'],['DM','DM'],['RCM','CM'],['LCM','CM'],['RW','W/AM'],['LW','W/AM'],['ST','ST']]},
 '4-2-3-1': {slots:[['GK','GK'],['RB','FB/WB'],['RCB','CB'],['LCB','CB'],['LB','FB/WB'],['RDM','DM'],['LDM','DM'],['RW','W/AM'],['AM','AM'],['LW','W/AM'],['ST','ST']]},
 '4-4-2': {slots:[['GK','GK'],['RB','FB/WB'],['RCB','CB'],['LCB','CB'],['LB','FB/WB'],['RM','W/AM'],['RCM','CM'],['LCM','CM'],['LM','W/AM'],['RST','ST'],['LST','ST']]},
 '3-4-2-1': {slots:[['GK','GK'],['RCB','CB'],['CB','CB'],['LCB','CB'],['RWB','FB/WB'],['RCM','CM'],['LCM','CM'],['LWB','FB/WB'],['RAM','AM'],['LAM','AM'],['ST','ST']]},
 '3-5-2': {slots:[['GK','GK'],['RCB','CB'],['CB','CB'],['LCB','CB'],['RWB','FB/WB'],['RCM','CM'],['DM','DM'],['LCM','CM'],['LWB','FB/WB'],['RST','ST'],['LST','ST']]},
 '5-2-3': {slots:[['GK','GK'],['RWB','FB/WB'],['RCB','CB'],['CB','CB'],['LCB','CB'],['LWB','FB/WB'],['RCM','CM'],['LCM','CM'],['RW','W/AM'],['LW','W/AM'],['ST','ST']]}
};

const ATTR={
  speed:['Pace','Acceleration'], finishing:['Finishing','Composure','Anticipation'], creation:['Passing','Vision','Technique','Decisions','First Touch'], pressing:['Work Rate','Stamina','Aggression','Acceleration','Anticipation'], defending:['Marking','Tackling','Positioning','Anticipation','Concentration'], aerial:['Jumping Reach','Heading','Strength'], movement:['Off the Ball','Acceleration','Pace'], security:['Decisions','Composure','Positioning','First Touch']
};

function clamp(n,a=0,b=100){return Math.max(a,Math.min(b,n));}
function mean(a){const v=a.filter(x=>Number.isFinite(x));return v.length?v.reduce((x,y)=>x+y,0)/v.length:0;}
function attr(player,name){return Number(player.attributes?.[name]??0);}
function avgAttr(player,names){return mean(names.map(n=>attr(player,n)));}
function fitRole(player,role){
  const vals=role.attrs.map(a=>attr(player,a));
  const base=mean(vals)*5;
  const missing=role.attrs.filter(a=>attr(player,a)===0).length;
  const penalty=missing*4;
  return clamp(Math.round(base-penalty));
}
function roleMap(){return Object.fromEntries(ROLE_DB.map(r=>[r.name,r]));}
function groupOk(role,group){return role.groups.some(g=>group.split('/').includes(g));}

function squadDNA(players){
  const scores={
    pace: mean(players.map(p=>avgAttr(p,ATTR.speed))),
    finishing: mean(players.map(p=>avgAttr(p,ATTR.finishing))),
    creation: mean(players.map(p=>avgAttr(p,ATTR.creation))),
    pressing: mean(players.map(p=>avgAttr(p,ATTR.pressing))),
    defending: mean(players.map(p=>avgAttr(p,ATTR.defending))),
    aerial: mean(players.map(p=>avgAttr(p,ATTR.aerial))),
    movement: mean(players.map(p=>avgAttr(p,ATTR.movement))),
    security: mean(players.map(p=>avgAttr(p,ATTR.security)))
  };
  const labels={pace:'Transition speed',finishing:'Box finishing',creation:'Build-up & creation',pressing:'Pressing capacity',defending:'Defensive stability',aerial:'Aerial threat',movement:'Off-ball movement',security:'Ball security'};
  const strengths=Object.entries(scores).sort((a,b)=>b[1]-a[1]).slice(0,3).map(([k,v])=>({key:k,label:labels[k],score:Math.round(v)}));
  const weaknesses=Object.entries(scores).sort((a,b)=>a[1]-b[1]).slice(0,3).map(([k,v])=>({key:k,label:labels[k],score:Math.round(v)}));
  return {scores:Object.fromEntries(Object.entries(scores).map(([k,v])=>[k,Math.round(v)])),strengths,weaknesses};
}

function candidateForFormation(formation,players){
  const cfg=FORMATIONS[formation];
  const used=new Set(); const picks=[];
  for(const [slot,group] of cfg.slots){
    const roleCandidates=ROLE_DB.filter(r=>groupOk(r,group));
    let best=null;
    for(const player of players){
      if(used.has(player.id)) continue;
      for(const role of roleCandidates){
        const fit=fitRole(player,role);
        let pBonus=0;
        if(slot==='ST' && ['Centre Forward','Poacher','Channel Forward'].includes(role.name)) pBonus=avgAttr(player,ATTR.finishing)*0.4;
        if(['RW','LW','RM','LM','RWB','LWB'].includes(slot) && role.traits.includes('width')) pBonus+=avgAttr(player,['Pace','Acceleration','Dribbling'])*0.15;
        const score=fit+pBonus;
        if(!best||score>best.score) best={player,role,score};
      }
    }
    if(best){used.add(best.player.id); picks.push({...best,slot});}
  }
  return picks;
}

function structuralScore(picks,formation,mentality,ti){
  const roles=picks.map(p=>p.role);
  const width=roles.filter(r=>r.traits.includes('width')).length;
  const depth=roles.filter(r=>r.traits.includes('depth')||r.traits.includes('runner')||r.traits.includes('attack-space')).length;
  const creator=roles.filter(r=>r.traits.includes('creator')).length;
  const screen=roles.filter(r=>r.traits.includes('screen')||r.traits.includes('cover')||r.traits.includes('security')).length;
  const press=roles.filter(r=>r.traits.includes('press')||r.traits.includes('track')).length;
  const risk={width:width>=2?0:10,depth:depth>=2?0:10,creator:creator>=1?0:8,screen:screen>=1?0:12,press:ti.press==='High'?(press>=3?0:12):0};
  let base=85-Object.values(risk).reduce((a,b)=>a+b,0);
  if(formation==='4-3-3 DM Wide' && width>=2) base+=5;
  if(mentality==='Attacking' && screen<1) base-=8;
  if(ti.defLine==='High' && picks.filter(p=>p.role.name.includes('Covering')).length===0) base-=6;
  return clamp(Math.round(base));
}

function scoreCandidate(formation,players,mentality,ti){
  const picks=candidateForFormation(formation,players);
  const avgRole=mean(picks.map(x=>x.score));
  const structural=structuralScore(picks,formation,mentality,ti);
  const dna=squadDNA(players);
  let matchup=80;
  if(mentality==='Positive') matchup += dna.strengths.some(x=>x.key==='creation'||x.key==='finishing')?5:0;
  if(mentality==='Balanced') matchup += dna.strengths.some(x=>x.key==='defending')?4:0;
  if(ti.press==='High') matchup += dna.scores.pressing>=70?7:-7;
  if(ti.defLine==='High') matchup += dna.scores.pace>=70?6:-8;
  if(ti.width==='Wide') matchup += dna.scores.pace>=70?4:0;
  if(ti.direct==='Direct') matchup += dna.scores.pace>=70?5:0;
  const complexity=clamp(100-(Object.values(ti).filter(v=>v&&v!=='Standard').length*5));
  const overall=Math.round(avgRole*0.38+structural*0.32+clamp(matchup)*0.18+complexity*0.12);
  const reasons=[]; const risks=[];
  if(dna.scores.pressing>=70 && ti.press==='High') reasons.push('The squad has the work rate, stamina and anticipation profile needed for an aggressive press.');
  if(dna.scores.finishing>=75) reasons.push('Strong finishing gives the structure a high-value final-third outlet.');
  if(dna.scores.creation>=75) reasons.push('Technical and creative quality supports controlled progression and combination play.');
  if(dna.scores.pace<60 && ti.defLine==='High') risks.push('The squad lacks enough recovery speed for a consistently high defensive line.');
  if(dna.scores.pressing<60 && ti.press==='High') risks.push('High pressing is likely to degrade when the first press is broken or fatigue appears.');
  if(structural<75) risks.push('The role structure has an occupation/coverage weakness that should be corrected before adding more instructions.');
  return {formation,mentality,ti,picks,components:{roleFit:Math.round(avgRole),structural,matchup:Math.round(matchup),complexity},overall,confidence:clamp(Math.round(65+players.length*1.2-(players.length<8?18:0))),reasons,risks};
}

function analyse(players){
  const dna=squadDNA(players); const m=['Balanced','Positive','Attacking']; const ti={press:'High',defLine:'High',width:'Wide',direct:'Mixed'};
  const forms=Object.keys(FORMATIONS); const candidates=[];
  for(const f of forms){for(const mentality of m){candidates.push(scoreCandidate(f,players,mentality,ti));}}
  candidates.sort((a,b)=>b.overall-a.overall); return {dna,candidates:candidates.slice(0,6)};
}

window.TA={ROLE_DB,FORMATIONS,analyse,fitRole,candidateForFormation,squadDNA,scoreCandidate};
