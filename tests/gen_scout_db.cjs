// Gera uma base de dados sintética no formato exacto do `fmsave export SAVE players --all` (nomes planos).
// Determinística (semente fixa). Uso: node tests/gen_scout_db.cjs N > ficheiro.csv
const ATTR=['crossing','dribbling','finishing','heading','long_shots','marking','off_the_ball','passing','penalty_taking','tackling','vision','handling','aerial_reach','command_of_area','communication','kicking','throwing','anticipation','decisions','one_on_ones','positioning','reflexes','first_touch','technique','flair','corners','teamwork','work_rate','long_throws','eccentricity','rushing_out','punching','acceleration','free_kick_taking','strength','stamina','pace','jumping_reach','leadership','dirtiness','balance','bravery','consistency','aggression','agility','important_matches','injury_proneness','versatility','natural_fitness','determination','composure','concentration'];
const POS=['gk','dl','dc','dr','dm','ml','mc','mr','aml','amc','amr','stc','wbl','wbr'];
const PERS=['ambition','professionalism','pressure','loyalty','temperament','adaptability','sportsmanship','controversy'];
const COLS=['uid','unique_id','name','age','nation_id','height_cm','ability_current','ability_potential','ability_potential_range_code','reputation_current','reputation_world','club_uid','club_name','club_nation_id','club_reputation','on_loan','loan_parent_club_name','transfer_value','contract_wage','contract_end','contract_squad_status','left_foot','right_foot',...POS.map(x=>'positions_'+x),...PERS.map(x=>'personality_'+x),...ATTR.map(x=>'attributes_'+x),'transfer_value_state','transfer_value_state_code'];
function gen(n, seed=42){ let s=seed; const r=()=>{ s=(s*1103515245+12345)%2147483648; return s/2147483648; }; const ri=(a,b)=>a+Math.floor(r()*(b-a+1));
  const clubs=Array.from({length:400},(_,i)=>({uid:1000+i,name:'Club '+i+(i%7===0?' "B"':''),nat:1+(i%20),rep:ri(500,9500)}));
  const lines=[COLS.join(',')];
  for(let i=0;i<n;i++){ const age=ri(15,38), ca=Math.max(30,Math.min(195,Math.round(60+r()*110-(age<19?40:0)))), pa=Math.min(200,ca+(age<21?ri(0,70):ri(0,10)));
    const free=r()<0.04, c=free?null:clubs[ri(0,clubs.length-1)], main=POS[ri(0,POS.length-1)];
    const lvl=Math.max(1,Math.min(20,Math.round(ca/10)));
    const att=ATTR.map(()=>Math.max(1,Math.min(20,lvl+ri(-5,5))));
    const posv=POS.map(p=>p===main?20:(r()<0.15?ri(10,18):ri(1,6)));
    const endY=2027+ri(0,4), lf=ri(3,20), rf=ri(3,20);
    const row={uid:100000+i,unique_id:200000+i,name:'Player '+i,age,nation_id:1+(i%30),height_cm:ri(165,198),ability_current:ca,ability_potential:pa,ability_potential_range_code:'',
      reputation_current:ri(100,9000),reputation_world:ri(100,9000),club_uid:c?c.uid:'',club_name:c?c.name:'',club_nation_id:c?c.nat:'',club_reputation:c?c.rep:'',on_loan:r()<0.05?'True':'False',
      loan_parent_club_name:'',transfer_value:free?'':Math.round(ca*ca*ri(30,120)*(age<22?2:1)),contract_wage:free?'':ri(500,150000),contract_end:free?'':`${endY}-06-30`,contract_squad_status:'Rotation',left_foot:lf,right_foot:rf,
      transfer_value_state:'ok',transfer_value_state_code:0};
    POS.forEach((p,j)=>row['positions_'+p]=posv[j]); PERS.forEach(p=>row['personality_'+p]=ri(1,20)); ATTR.forEach((a,j)=>row['attributes_'+a]=att[j]);
    lines.push(COLS.map(k=>{ const v=row[k]; const t=v==null?'':String(v); return /[",]/.test(t)?'"'+t.replace(/"/g,'""')+'"':t; }).join(','));
  }
  return lines.join('\n');
}
module.exports={gen,COLS};
if (require.main===module) process.stdout.write(gen(+process.argv[2]||2000));
