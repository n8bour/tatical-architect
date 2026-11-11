// ============================================================
// Formações, instruções de equipa/jogador/adversário, treino, conceitos
// ============================================================

// Papéis cujos pesos de atributos foram confirmados em fontes públicas FM26
const ATTR_VERIFIED = new Set(['dlf','poa','f9','iw_ip','scm','pfb','lhk']);

// ---- Formações: x 0(esq)→100(dir), y 0(baliza própria)→100(baliza adversária)
const S = (code, x, y) => ({ code, x, y });
const FORMATIONS = {
  '4-3-3 DM Wide': [S('GK',50,4),S('DL',10,24),S('DCL',36,20),S('DCR',64,20),S('DR',90,24),S('DM',50,38),S('MCL',32,52),S('MCR',68,52),S('AML',12,72),S('ST',50,84),S('AMR',88,72)],
  '4-2-3-1 DM Wide': [S('GK',50,4),S('DL',10,24),S('DCL',36,20),S('DCR',64,20),S('DR',90,24),S('DML',38,38),S('DMR',62,38),S('AML',12,68),S('AMC',50,66),S('AMR',88,68),S('ST',50,84)],
  '4-2-3-1 Wide': [S('GK',50,4),S('DL',10,24),S('DCL',36,20),S('DCR',64,20),S('DR',90,24),S('MCL',38,46),S('MCR',62,46),S('AML',12,68),S('AMC',50,66),S('AMR',88,68),S('ST',50,84)],
  '4-1-4-1 DM Wide': [S('GK',50,4),S('DL',10,24),S('DCL',36,20),S('DCR',64,20),S('DR',90,24),S('DM',50,36),S('ML',10,52),S('MCL',36,52),S('MCR',64,52),S('MR',90,52),S('ST',50,80)],
  '4-4-2': [S('GK',50,4),S('DL',10,24),S('DCL',36,20),S('DCR',64,20),S('DR',90,24),S('ML',10,50),S('MCL',38,48),S('MCR',62,48),S('MR',90,50),S('STL',40,80),S('STR',60,80)],
  '4-4-1-1': [S('GK',50,4),S('DL',10,24),S('DCL',36,20),S('DCR',64,20),S('DR',90,24),S('ML',10,50),S('MCL',38,48),S('MCR',62,48),S('MR',90,50),S('AMC',50,68),S('ST',50,84)],
  '4-4-2 Diamond': [S('GK',50,4),S('DL',10,24),S('DCL',36,20),S('DCR',64,20),S('DR',90,24),S('DM',50,36),S('MCL',30,50),S('MCR',70,50),S('AMC',50,64),S('STL',40,82),S('STR',60,82)],
  '4-2-2-2': [S('GK',50,4),S('DL',10,24),S('DCL',36,20),S('DCR',64,20),S('DR',90,24),S('DML',38,38),S('DMR',62,38),S('AMCL',30,66),S('AMCR',70,66),S('STL',40,84),S('STR',60,84)],
  '4-3-3 Flat': [S('GK',50,4),S('DL',10,24),S('DCL',36,20),S('DCR',64,20),S('DR',90,24),S('MCL',28,48),S('MC',50,46),S('MCR',72,48),S('AML',12,72),S('ST',50,84),S('AMR',88,72)],
  '3-4-2-1': [S('GK',50,4),S('DCL',28,20),S('DC',50,18),S('DCR',72,20),S('WBL',8,42),S('MCL',38,46),S('MCR',62,46),S('WBR',92,42),S('AMCL',32,68),S('AMCR',68,68),S('ST',50,84)],
  '3-4-3': [S('GK',50,4),S('DCL',28,20),S('DC',50,18),S('DCR',72,20),S('WBL',8,44),S('MCL',38,46),S('MCR',62,46),S('WBR',92,44),S('AML',14,72),S('ST',50,84),S('AMR',86,72)],
  '3-5-2': [S('GK',50,4),S('DCL',28,20),S('DC',50,18),S('DCR',72,20),S('WBL',8,44),S('DM',50,36),S('MCL',34,52),S('MCR',66,52),S('WBR',92,44),S('STL',40,82),S('STR',60,82)],
  '5-3-2 WB': [S('GK',50,4),S('WBL',8,28),S('DCL',30,18),S('DC',50,16),S('DCR',70,18),S('WBR',92,28),S('MCL',30,46),S('MC',50,44),S('MCR',70,46),S('STL',40,76),S('STR',60,76)],
  '5-4-1': [S('GK',50,4),S('WBL',8,26),S('DCL',30,18),S('DC',50,16),S('DCR',70,18),S('WBR',92,26),S('ML',12,46),S('MCL',38,44),S('MCR',62,44),S('MR',88,46),S('ST',50,74)],
  '4-2-4 DM Wide': [S('GK',50,4),S('DL',10,24),S('DCL',36,20),S('DCR',64,20),S('DR',90,24),S('DML',38,38),S('DMR',62,38),S('AML',10,66),S('AMR',90,66),S('STL',40,84),S('STR',60,84)],
  '4-4-2 DM Wide': [S('GK',50,4),S('DL',10,22),S('DCL',36,18),S('DCR',64,18),S('DR',90,22),S('DML',38,34),S('DMR',62,34),S('ML',10,48),S('MR',90,48),S('STL',42,72),S('STR',58,72)],
  '4-5-1': [S('GK',50,4),S('DL',10,22),S('DCL',36,18),S('DCR',64,18),S('DR',90,22),S('ML',10,44),S('MCL',32,44),S('MC',50,42),S('MCR',68,44),S('MR',90,44),S('ST',50,72)],
};

function groupOf(code, formation) {
  if (code === 'GK') return 'GK';
  if (code === 'DL' || code === 'DR') return 'D_LR';
  if (code.startsWith('DC')) return 'DC';
  if (code.startsWith('WB')) return 'WB_LR';
  if (code.startsWith('DM')) return 'DM';
  if (code === 'ML' || code === 'MR') return 'M_LR';
  if (code.startsWith('MC')) return 'MC';
  if (code === 'AML' || code === 'AMR') return 'AM_LR';
  if (code.startsWith('AMC')) return 'AMC';
  if (code.startsWith('ST')) return 'ST';
  return 'MC';
}
function isWideCB(code, formation) {
  const slots = FORMATIONS[formation] || [];
  const nCB = slots.filter(s => s.code.startsWith('DC')).length;
  return nCB >= 3 && (code === 'DCL' || code === 'DCR');
}
function sideOf(x) { return x < 33 ? 'L' : x > 67 ? 'R' : 'C'; }

const MENTALITIES = ['Very Defensive','Defensive','Cautious','Balanced','Positive','Attacking','Very Attacking'];
const MENTALITY_INFO = {
  'Very Defensive':'Bloco muito baixo, risco mínimo. Para proteger vantagens curtas ou jogos de enorme inferioridade.',
  'Defensive':'Prioriza a solidez, transições cautelosas. Underdog fora de casa.',
  'Cautious':'Equilibrado mas com prudência; posse mais lenta e paciente.',
  'Balanced':'Ponto de partida neutro; as instruções e os papéis definem o carácter.',
  'Positive':'Proactivo sem extremos: avança linhas, mais risco controlado. Óptimo default para equipas favoritas.',
  'Attacking':'Mais largura, ritmo e risco. Atenção à rest-defence.',
  'Very Attacking':'Tudo ao ataque. Use em fases finais quando precisa de golos; expõe muito as transições.',
};

// ---- Instruções de Equipa (nomes FM26; FM27 pode reagrupar/renomear — confirmar)
// opts: valores ordenados. def: índice default. cx: peso de complexidade por passo afastado do default.
const TEAM_INSTRUCTIONS = [
  // IP — gerais
  { k:'passing', n:'Passing Directness', ph:'IP', area:'Geral', opts:['Much Shorter','Shorter','Balanced','More Direct','Much More Direct'], def:2, cx:1,
    d:'Comprimento médio dos passes.', pro:'Curto: retém a bola, cansa o adversário. Directo: chega rápido à frente, ignora a pressão.',
    con:'Curto exige Passe/Técnica/Compostura em todos. Directo perde mais bolas se não houver referência/velocidade.',
    need:{ low:['pas','tec','cmp','fir'], high:['hea','str','otb','pac'] } },
  { k:'tempo', n:'Tempo', ph:'IP', area:'Geral', opts:['Much Lower','Lower','Balanced','Higher','Much Higher'], def:2, cx:1,
    d:'Velocidade da tomada de decisão com bola.', pro:'Alto: apanha o adversário desorganizado. Baixo: controlo e paciência.',
    con:'Alto + passe curto exige Decisões/Técnica altas; cansa mais.', need:{ high:['dec','tec','fir','sta'] } },
  { k:'width', n:'Attacking Width', ph:'IP', area:'Geral', opts:['Much Narrower','Narrower','Balanced','Wider','Much Wider'], def:2, cx:1,
    d:'Quão aberta fica a equipa com bola.', pro:'Largo: estica o bloco adversário, abre corredores interiores. Estreito: combinações centrais.',
    con:'Largo dificulta a reacção à perda (counter-press). Estreito facilita o fecho do adversário ao meio.' },
  { k:'creative', n:'Creative Freedom', ph:'IP', area:'Geral', opts:['More Disciplined','Balanced','More Expressive'], def:1, cx:1,
    d:'Liberdade para sair do plano.', pro:'Expressivo: jogadores de topo encontram soluções. Disciplinado: execução previsível e segura.',
    con:'Expressivo com jogadores fracos de Decisões/Talento = caos.', need:{ high:['dec','fla','vis'] } },
  { k:'att_trans', n:'Attacking Transition', ph:'IP', area:'Transição', opts:['Hold Shape','Balanced','Counter-Attack'], def:1, cx:1,
    d:'O que fazer ao recuperar a bola.', pro:'Counter: explora defesa desorganizada. Hold Shape: reorganiza e assume posse.',
    con:'Counter sem velocidade/outlets resulta em perdas rápidas.', need:{ high:['pac','acc','otb'] } },
  { k:'set_pieces', n:'Play for Set Pieces', ph:'IP', area:'Geral', opts:['Keep Ball in Play','Balanced','Play for Set Pieces'], def:1, cx:0.5, fm27:true, unconf:true,
    d:'Procurar ou evitar ganhar cantos e livres (nova no FM27).', pro:'Play for Set Pieces rende com bons cabeceadores e marcadores de bolas paradas.',
    con:'Keep Ball in Play mantém o ritmo mas desperdiça uma arma se fores forte no ar.' },
  { k:'timewaste', n:'Time Wasting', ph:'IP', area:'Geral', opts:['Less','Balanced','More'], def:1, cx:0.5,
    d:'Gestão do relógio.', pro:'Útil a ganhar no fim.', con:'Pode irritar adversários/árbitro; quebra o ritmo da própria equipa.' },
  { k:'reception', n:'Pass Reception', ph:'IP', area:'Progressão', opts:['Pass into Space','Balanced','Pass to Feet'], def:1, cx:1,
    d:'Passar para o espaço ou para o pé (novo no FM26).', pro:'Ao pé: controlo. Espaço: profundidade e velocidade.',
    con:'Ao pé contra pressão alta exige Primeiro Toque/Compostura. Espaço exige Sem Bola/Aceleração.', need:{ low:['otb','acc'], high:['fir','cmp','tec'] } },
  // IP — construção
  { k:'buildup', n:'Build-up Strategy', ph:'IP', area:'Construção', opts:['Play Through Press','Balanced','Bypass Press'], def:1, cx:1,
    d:'Sair a jogar curto contra a pressão ou ultrapassá-la (novo no FM26).', pro:'Through: atrai e quebra a pressão. Bypass: risco mínimo atrás.',
    con:'Through com GR/centrais fracos tecnicamente = golos sofridos. Bypass perde posse.', need:{ low:['pas','cmp','fir','dec'] } },
  { k:'goal_kicks', n:'Goal Kicks', ph:'IP', area:'Construção', opts:['Short','Mixed','Long'], def:1, cx:0.5,
    d:'Tipo de pontapé de baliza.', pro:'Curto: inicia a construção. Longo: salta a pressão para a referência.',
    con:'Curto contra pressão alta e defesas fracos com bola é arriscado.' },
  { k:'gk_speed', n:'GK Distribution Speed', ph:'IP', area:'Construção', opts:['Slow Pace Down','Balanced','Distribute Quickly'], def:1, cx:0.5,
    d:'Ritmo de reposição do guarda-redes.', pro:'Rápido alimenta contra-ataques.', con:'Rápido com a equipa desposicionada perde bolas.' },
  { k:'gk_target', n:'GK Distribution', ph:'IP', area:'Construção', opts:['Centre-Backs','Full-Backs','Balanced','Flanks','Target Forward'], def:2, cx:0.5,
    d:'Para onde o GR distribui.', pro:'Dirigir a bola ao seu ponto forte (central técnico, ala rápido, ponta-de-lança alto).',
    con:'Previsível: o adversário pode montar armadilha.' },
  // IP — progressão
  { k:'support_runs', n:'Supporting Runs', ph:'IP', area:'Progressão', opts:['Underlap','Balanced','Overlap'], def:1, cx:1,
    d:'Laterais/alas apoiam por dentro (underlap) ou por fora (overlap).', pro:'Overlap com extremos interiores; underlap com extremos abertos.',
    con:'Contraria-se com o papel: um W com overlap gera aglomeração no corredor.' },
  { k:'dribbling', n:'Dribbling', ph:'IP', area:'Progressão', opts:['Discourage','Balanced','Encourage'], def:1, cx:1,
    d:'Frequência de condução/drible.', pro:'Encoraje com driblers de qualidade.', con:'Encoraje com Drible fraco = perdas em zonas perigosas.', need:{ high:['dri','agi','fla'] } },
  { k:'progress', n:'Progress Through', ph:'IP', area:'Progressão', opts:['Left','Balanced','Centre','Right'], def:1, cx:0.5,
    d:'Zona preferida de progressão.', pro:'Concentre no lado do seu melhor jogador ou do ponto fraco adversário.',
    con:'Previsível; adversário pode sobrecarregar esse lado.' },
  // IP — último terço
  { k:'patience', n:'Final Third Patience', ph:'IP', area:'Último terço', opts:['Low','Balanced','High'], def:1, cx:1,
    d:'Trabalhar a bola até à área ou finalizar cedo.', pro:'Alta: melhores oportunidades. Baixa: mais remates, apanha bloco desorganizado.',
    con:'Alta contra bloco baixo pode gerar posse estéril.' },
  { k:'long_shots', n:'Shots from Distance', ph:'IP', area:'Último terço', opts:['Discourage','Balanced','Encourage'], def:1, cx:0.5,
    d:'Frequência de remates de fora da área.', pro:'Encoraje com bons rematadores contra blocos baixos.', con:'Remates fracos desperdiçam posses.', need:{ high:['lon','tec'] } },
  { k:'crossing', n:'Crossing Style', ph:'IP', area:'Último terço', opts:['Low Crosses','Balanced','Whipped Crosses','Floated Crosses'], def:1, cx:0.5,
    d:'Tipo de cruzamento.', pro:'Baixos para avançados rápidos; Floated para referências aéreas.',
    con:'Floated sem jogadores fortes no ar é desperdício.' },
  // OOP
  { k:'loe', n:'Line of Engagement', ph:'OOP', area:'Pressão', opts:['Low Block','Mid Block','High Press'], def:1, cx:1.5, fm27:true,
    d:'Onde a equipa começa a pressionar.', pro:'Alta: recupera bolas perto da baliza adversária. Baixa: bloco compacto, convida a sair.',
    con:'Alta exige Resistência/Índice de Trabalho; desligada da linha defensiva abre espaço entre linhas.', need:{ high:['wor','sta','acc','agg'] } },
  { k:'dline', n:'Defensive Line', ph:'OOP', area:'Bloco', opts:['Much Lower','Lower','Standard','Higher','Much Higher'], def:2, cx:1,
    d:'Altura da linha defensiva.', pro:'Alta: equipa compacta, adversário longe da baliza. Baixa: protege a profundidade.',
    con:'Alta exige centrais rápidos e Sweeper Keeper. Baixa convida pressão e remates.', need:{ high:['pac','acc','ant','pos'] } },
  { k:'trigger', n:'Trigger Press', ph:'OOP', area:'Pressão', opts:['Much Less','Less','Balanced','More','Much More'], def:2, cx:1,
    d:'Frequência com que os jogadores disparam a pressão.', pro:'Mais: recuperações altas, caos para o adversário.',
    con:'Esgota fisicamente; pressão descoordenada é facilmente batida.', need:{ high:['wor','sta','agg','ant','tea'] } },
  { k:'def_trans', n:'Defensive Transition', ph:'OOP', area:'Transição', opts:['Regroup','Balanced','Counter-Press'], def:1, cx:1,
    d:'Reacção imediata à perda da bola.', pro:'Counter-Press: recupera bolas antes do adversário se organizar. Regroup: protege a estrutura.',
    con:'Counter-Press com equipa muito aberta/cansada é atravessado em transição.', need:{ high:['wor','sta','acc','tea'] } },
  { k:'tackling', n:'Tackling', ph:'OOP', area:'Duelos', opts:['Stay On Feet','Balanced','Get Stuck In'], def:1, cx:0.5,
    d:'Agressividade nos desarmes.', pro:'Get Stuck In ganha mais duelos.', con:'Mais faltas e cartões; perigoso com defesas de baixo Desarme.', need:{ high:['tck','agg','bra'] } },
  { k:'press_trap', n:'Pressing Trap', ph:'OOP', area:'Pressão', opts:['Inside','Balanced','Outside'], def:1, cx:1,
    d:'Para onde se força o portador da bola (bloco médio/pressão alta).', pro:'Outside: encurrala no corredor. Inside: força para o meio congestionado.',
    con:'Outside sem laterais/extremos agressivos não fecha a armadilha; Inside exige meio-campo forte.' },
  { k:'cross_eng', n:'Cross Engagement', ph:'OOP', area:'Bloco baixo', opts:['Invite Crosses','Balanced','Stop Crosses'], def:1, cx:0.5,
    d:'Permitir ou impedir cruzamentos (bloco baixo).', pro:'Invite: com centrais dominantes no ar. Stop: com defesas baixos.',
    con:'Invite sem centrais fortes no ar = golos de cabeça.' },
  { k:'short_gk', n:'Short Goalkeeper Distribution', ph:'OOP', area:'Pressão', opts:['No','Yes'], def:0, cx:0.5, fm27:true,
    d:'Pressionar a reposição curta do GR adversário (secção High Press).', pro:'Yes força o adversário a bater longo.', con:'Exige subir os avançados e médios — mais espaço atrás.' },
  { k:'dl_behaviour', n:'Defensive Line Behaviour', ph:'OOP', area:'Bloco', opts:['Drop Off','Balanced','Step Up'], def:1, cx:0.5, fm27:true, unconf:true,
    d:'Se a linha recua para proteger a profundidade ou sobe para apertar o espaço (nova no FM27).', pro:'Step Up: equipa compacta, apanha foras-de-jogo. Drop Off: protege as costas.',
    con:'Step Up com centrais lentos é perigoso; Drop Off afasta a defesa do meio-campo.' },
];
const TI_BY_KEY = Object.fromEntries(TEAM_INSTRUCTIONS.map(t => [t.k, t]));

// ---- Instruções individuais (FM26 base + novidades confirmadas FM27)
const PLAYER_INSTRUCTIONS = [
  { n:'Shoot more', g:'Remate', d:'Remata mais vezes, incluindo de fora da área.', tip:'Remates de Longe ≥ 15 e Técnica alta.' },
  { n:'Shoot less', g:'Remate', d:'Procura o passe em vez do remate.', tip:'Quem desperdiça posses a rematar.' },
  { n:'Dribble more', g:'Drible', d:'Conduz e procura o um-para-um.', tip:'Drible + Agilidade altos.' },
  { n:'Dribble less', g:'Drible', d:'Solta a bola mais cedo.', tip:'Drible fraco em zonas perigosas.' },
  { n:'Take more risks', g:'Passe', d:'Tenta passes de ruptura.', tip:'Passe + Visão ≥ 15.' },
  { n:'Take fewer risks', g:'Passe', d:'Passes seguros.', tip:'Centrais/médios com Passe fraco.' },
  { n:'Cross from byline', g:'Cruzamento', d:'Vai à linha antes de cruzar.', tip:'Wingers rápidos.' },
  { n:'Cross from deep', g:'Cruzamento', d:'Cruza mais cedo, de trás.', tip:'Alas e extremos interiores com bom Cruzamento.' },
  { n:'Roam from position', g:'Movimento', d:'Liberdade para sair da posição.', tip:'Só para jogadores com Decisões/Sem Bola altos; aumenta a complexidade.' },
  { n:'Stay wider', g:'Movimento', d:'Fica mais aberto.', tip:'Equilibra flancos assimétricos.' },
  { n:'Sit narrower', g:'Movimento', d:'Fecha para dentro.', tip:'Ajuda a superioridade central.' },
  { n:'Hold position', g:'Movimento', d:'Fica na sua zona em vez de acompanhar o ataque.', tip:'Conta para a rest-defence.' },
  { n:'Get further forward', g:'Movimento', d:'Chega mais vezes ao último terço.', tip:'Médios/laterais com Resistência e Sem Bola altos.' },
  { n:'Move into channels', g:'Movimento', d:'Procura o espaço entre central e lateral.', tip:'Avançados rápidos contra linhas altas.' },
  { n:'Press more', g:'Pressão', d:'Pressiona mais do que o resto da equipa.', tip:'Índice de Trabalho e Resistência altos.' },
  { n:'Press less', g:'Pressão', d:'Poupa energia na pressão.', tip:'Resistência baixa.' },
  { n:'Mark tighter', g:'Defesa', d:'Marcação mais apertada.', tip:'Marcação ≥ 15.' },
  { n:'Tackle harder', g:'Defesa', d:'Desarmes mais agressivos.', tip:'Desarme e Agressividade altos.' },
  { n:'Ease off tackles', g:'Defesa', d:'Desarmes mais cautelosos.', tip:'Desarme fraco ou jogador amarelado.' },
  { n:'Arrive Late', g:'Área (FM27)', d:'Chega à área em segunda vaga, atacando o espaço que se abre.', tip:'Box-to-Box, Attacking Midfielder, Channel Midfielder.', fm27:true },
  { n:'Attack the Defensive Line', g:'Área (FM27)', d:'Ataca a profundidade na última linha.', tip:'Poacher, Channel Forward, Inside Forward rápidos.', fm27:true },
  { n:'Target Near Post', g:'Área (FM27)', d:'Ataca o primeiro poste nos cruzamentos.', tip:'Avançado rápido, Poacher.', fm27:true },
  { n:'Target Centre', g:'Área (FM27)', d:'Ataca o centro da área nos cruzamentos.', tip:'Referência aérea.', fm27:true },
  { n:'Target Far Post', g:'Área (FM27)', d:'Ataca o segundo poste nos cruzamentos.', tip:'Extremo do lado oposto, Wide Forward.', fm27:true },
];

const OPPOSITION_INSTRUCTIONS = [
  { n:'Show onto (weaker) foot', d:'Força o adversário para o pé mais fraco.', tip:'Default útil contra extremos que cortam para dentro (mostre-os para fora).' },
  { n:'Close down (more/less)', d:'Pressiona mais ou menos um jogador específico.', tip:'Mais no criativo/construtor; menos no central fraco com bola (deixe-o conduzir).' },
  { n:'Tight marking', d:'Marcação em cima.', tip:'No médio ofensivo/falso 9; cuidado com arrastar centrais para fora.' },
  { n:'Tackle harder', d:'Desarmes mais agressivos sobre esse jogador.', tip:'Contra driblers com pouca Bravura/Equilíbrio; evite com o seu defesa amarelado.' },
];

// ---- Tactical Tweaks (FM27, confirmados em antevisões; o jogo mostra as TIs exactas ao passar o rato)
const TACTICAL_TWEAKS = [
  { n:'More Urgency', d:'Aumenta a urgência: tipicamente sobe ritmo e verticalidade.', when:'A perder ou empatado contra bloco baixo nos últimos 20–30 min.' },
  { n:'Play Longer', d:'Joga mais directo.', when:'Quando a construção curta está a ser pressionada e perde bolas atrás, ou para explorar uma referência.' },
  { n:'Exploit Width', d:'Estica o jogo pelos corredores.', when:'Adversário estreito (ex. 4-4-2 losango, 3-5-2 sem alas que recuem).' },
  { n:'Press Less', d:'Baixa a intensidade da pressão.', when:'Jogadores cansados, a ganhar, ou pressão a ser batida sistematicamente.' },
  { n:'Wingers Swap Sides', d:'Troca os extremos de flanco.', when:'Para atacar o lateral mais fraco, ou para meter o extremo no pé forte/fraco conforme o objectivo.' },
];

// ---- Treino FM27 (Tactical Effectiveness)
const TRAINING = {
  effectiveness: [
    { n:'Tactical Understanding', d:'Quanto a equipa compreende cada área da tática: IP Shape, OOP Shape, Passing Style, Build Up, Chance Creation e Pressing (Trigger Press, Pressing Trap, Line of Engagement). Influenciado por contributo do treino, exposição a jogos, capacidade tática dos jogadores e complexidade da tática.',
      fix:'Sessões de Tactical Understanding focadas na área mais fraca; simplificar a tática (menos instruções não-default).' },
    { n:'Player Ability', d:'Adequação ao papel (Player Role Suitability) e capacidade pura (Current Ability/atributos).',
      fix:'Individual Development com Role Training no papel exacto da tática; recrutar para o papel.' },
    { n:'Player Buy-In', d:'Managerial Support (personalidade, desempenho e respeito pelo treinador), relações com colegas/staff e tendências do treinador.',
      fix:'Team Cohesion; conversas individuais; cumprir promessas; evitar muitas mudanças de plantel de uma vez.' },
    { n:'Next Match Information', d:'Qualidade do relatório do próximo adversário (analistas) e conhecimento dos jogadores adversários.',
      fix:'Match Preparation; atribuir o relatório ao melhor analista; observar a divisão e o adversário.' },
  ],
  areas: {
    ip_shape:{ n:'In-Possession Shape', s:'Tactical Understanding: posse/estrutura ofensiva' },
    oop_shape:{ n:'Out-of-Possession Shape', s:'Tactical Understanding: estrutura defensiva' },
    passing:{ n:'Passing Style', s:'Tactical Understanding: estilo de passe' },
    buildup:{ n:'Build Up', s:'Tactical Understanding: construção' },
    chance:{ n:'Chance Creation', s:'Tactical Understanding: criação no último terço' },
    trigger:{ n:'Trigger Press', s:'Tactical Understanding: pressing' },
    trap:{ n:'Pressing Trap', s:'Tactical Understanding: pressing' },
    loe:{ n:'Line of Engagement', s:'Tactical Understanding: pressing' },
  },
  principles: [
    'Treine a ideia, não só os atributos: no FM27 o treino está ligado à compreensão da tática. Uma área com compreensão baixa = sessões dessa área na semana.',
    'Equilibre: ao focar uma área, outras descem ligeiramente. O objectivo é ter todas em níveis semelhantes, não uma perfeita.',
    'Pré-época: carga física + Tactical Understanding de IP/OOP Shape primeiro; pressing e chance creation depois.',
    'Semana com 2 jogos: Recuperação + Match Preparation; sem sessões pesadas.',
    'Individual Development: Role Training no papel EXACTO da tática (IP ou OOP mais exigente para esse jogador).',
    'Jovens: minutos > tudo. Se não joga, empréstimo a um clube que use papel parecido.',
    'Mentoria: grupos pequenos com 1–2 líderes de alta Determinação/profissionalismo e jovens com personalidade a moldar.',
    'Team Cohesion após muitas contratações ou mudança de treinador (Player Buy-In baixo).',
    'Complexidade: cada instrução fora do default aumenta o tempo de adaptação. Comece simples, adicione uma instrução quando resolver um problema identificado.',
  ],
  periodization: [
    { p:'Pré-época (sem 1–2)', f:'Físico + IP/OOP Shape', n:'Base aeróbica; estrutura de posse e bloco.' },
    { p:'Pré-época (sem 3–4)', f:'Pressing (Trigger/Trap/LoE) + Build Up', n:'Ligar a pressão à estrutura; construção contra pressão.' },
    { p:'Pré-época (sem 5–6)', f:'Chance Creation + Match Prep', n:'Último terço; amigáveis com o 11 provável.' },
    { p:'Época — semana normal', f:'Área mais fraca + Match Prep + Recuperação', n:'1 sessão para a área com menor compreensão.' },
    { p:'Época — 2 jogos/semana', f:'Recuperação + Match Prep', n:'Evitar lesões; rotação.' },
    { p:'Pausa internacional', f:'Tactical Understanding + Cohesion', n:'Bom momento para introduzir mudanças táticas.' },
  ],
};

// ---- Conceitos / princípios (bíblia)
const CONCEPTS = [
  { n:'In / Out of Possession', t:'Desde o FM26, cada jogador tem um papel com bola (IP) e outro sem bola (OOP), e a equipa tem uma formação para cada fase. Não existem duties nem fluidez: o comportamento vem do papel + instruções. O jogo avisa se a distância entre a posição IP e OOP de um jogador é demasiado grande — isso cria buracos nas transições.' },
  { n:'Rest-defence', t:'Jogadores que ficam atrás da linha da bola durante o ataque para travar o contra-ataque. Regra prática: 3 a 5 (sem contar o GR) com mentalidade Positive/Attacking — tipicamente 2 centrais + 1 médio defensivo, ou um lateral que fecha por dentro (IFB/IWB).' },
  { n:'Largura', t:'Cada flanco precisa de UM jogador a dar largura no último terço (W, WF, WB, AWB, WCM, OCB). Se o extremo entra por dentro, a largura tem de vir de trás. Dois jogadores abertos no mesmo corredor atropelam-se.' },
  { n:'Ocupação da área', t:'Pelo menos 2–3 jogadores a atacar a área (avançado + extremo interior/AM/BBM). Com um falso 9 ou DLF, alguém tem de atacar a profundidade (IF, CHM, SS). No FM27 as novas instruções de ataque à área (near/centre/far post, arrive late) permitem coordenar quem vai a cada zona.' },
  { n:'Coerência da pressão', t:'Linha de pressão (LoE) e linha defensiva devem andar juntas. LoE alta + linha baixa = buraco entre linhas. LoE baixa + linha alta = espaço nas costas sem pressão sobre o passe. Pressing Trap Outside precisa de quem feche o corredor (Pressing FB/WB, Tracking Winger).' },
  { n:'Outlets', t:'Papéis OOP "Outlet" ficam na frente para o contra-ataque e não defendem. 1–2 dão uma saída; 3+ deixam o bloco em inferioridade.' },
  { n:'Criatividade', t:'1–2 criadores principais (DLP, MP, AP, FR, PW, PWB). Demasiados criadores = todos pedem a bola, ninguém ataca o espaço.' },
  { n:'Assimetria', t:'Lados diferentes não são um defeito: lateral seguro (IFB) num lado e ala ofensivo (WB/AWB) no outro é das estruturas mais estáveis. Faça corresponder o lado ofensivo ao seu melhor jogador.' },
  { n:'Complexidade (FM27)', t:'O FM27 mede a complexidade (instruções de equipa, individuais e de adversário). Quanto mais complexa, mais capazes os jogadores e mais tempo de treino precisam. Em divisões baixas, simplicidade ganha jogos.' },
  { n:'Tactical Effectiveness (FM27)', t:'Substitui a Familiaridade Tática. Quatro pilares: Tactical Understanding, Player Ability, Player Buy-In e Next Match Information. Visível em Treino, no ecrã de tácticas, plantel, base de dados e relatórios de observação ("Projected Effectiveness").' },
];

// ---- Contra formações adversárias
const OPPONENTS = [
  { f:'4-4-2', t:'Superioridade central com 3 médios (4-3-3/4-2-3-1). Ataque os meios-espaços nas costas dos médios ala. OOP: 4-4-2 compacto ou 4-1-4-1 cobrindo os dois avançados com 2 centrais + DM.' },
  { f:'4-2-3-1', t:'O AMC é a ameaça: Stopping CB, Screening DM, ou OI de marcação apertada. Os dois pivots podem ser pressionados com dois homens (4-4-2 OOP com AM a pressionar).' },
  { f:'4-3-3', t:'Decida onde quer superioridade: meio (losango/4-2-2-2) ou corredores (alas). Os laterais adversários sobem — explore com Wide Outlet ou Splitting Outlet.' },
  { f:'3-5-2 / 3-4-3', t:'Proteja os meios-espaços (central do lado + ala). Alas adversários dão largura: Pressing Trap Outside com Pressing FB. Use extremos para prender os alas atrás.' },
  { f:'5-3-2 / 5-4-1 (bloco baixo)', t:'Paciência alta, largura, remates de longe se tiver qualidade, sobrecarga num flanco. Counter-press para não ser apanhado em transição. Cruzamentos baixos e ataques ao segundo poste.' },
  { f:'4-4-2 losango', t:'É estreito: Exploit Width, alas/laterais ofensivos. OOP: feche o meio, deixe-os ir para fora e defenda o cruzamento.' },
];

// ---- Modelos táticos (usados pelas sugestões)
// slots: [IPcode, IProle, OOPcode, OOProle] — IP/OOP formações indicadas
const TEMPLATES = [
  { id:'cenas', n:'Cenas v2 — 4-2-4 DM Wide (a tua)', ip:'4-2-4 DM Wide', oop:'4-4-2 DM Wide', ment:'Positive',
    style:'counter', desc:'A tua tática tal como está no jogo (screenshots de 10/11), já com Winger/Wing-Back nos lados certos.',
    roles:{ GK:['gk_ip','gk_oop'], DL:['wb_ip','fb_oop'], DCL:['cb_ip','cb_oop'], DCR:['cb_ip','cb_oop'], DR:['fb_ip','fb_oop'], DML:['dlp','dm_oop'], DMR:['b2b','dm_oop'], AML:['iw_ip','wm_oop'], AMR:['wf_ip','wm_oop'], STL:['poa','cf_oop'], STR:['cf_ip','tcf'] },
    ti:{ passing:1, tempo:3, att_trans:2, timewaste:0, set_pieces:0, dribbling:2, reception:0, loe:1, dline:4, trigger:1, def_trans:2, short_gk:0 } },
  { id:'possession433', n:'4-3-3 Posse & Contra-pressão', ip:'4-3-3 DM Wide', oop:'4-1-4-1 DM Wide', ment:'Positive',
    style:'possession_press', desc:'Domínio com bola, saída curta, recuperação imediata.',
    roles:{ GK:['bpgk','swk'], DL:['wb_ip','fb_oop'], DCL:['bpcb','ccb'], DCR:['cb_ip','scb'], DR:['iwb','pfb'], DM:['dlp','sdm'], MCL:['chm','pcm'], MCR:['mp','cm_oop'], AML:['if_ip','wm_oop'], AMR:['w_ip','wm_oop'], ST:['f9','tcf'] },
    ti:{ passing:1, tempo:3, reception:2, buildup:0, goal_kicks:0, loe:2, dline:3, trigger:3, def_trans:2, press_trap:2 } },
  { id:'balanced4231', n:'4-2-3-1 Equilibrado', ip:'4-2-3-1 DM Wide', oop:'4-4-1-1', ment:'Balanced',
    style:'balanced', desc:'Base sólida e versátil. Fácil de aprender (baixa complexidade).',
    roles:{ GK:['gk_ip','gk_oop'], DL:['fb_ip','fb_oop'], DCL:['bpcb','cb_oop'], DCR:['cb_ip','cb_oop'], DR:['wb_ip','fb_oop'], DML:['dm_ip','scm'], DMR:['b2b','cm_oop'], AML:['iw_ip','wm_oop'], AMC:['am_ip','am_oop'], AMR:['wf_ip','wm_oop'], ST:['cf_ip','cf_oop'] },
    ti:{} },
  { id:'gegen4222', n:'4-2-2-2 Vertical Gegenpress', ip:'4-2-2-2', oop:'4-4-2', ment:'Attacking',
    style:'press_vertical', desc:'Pressão alta, ataques rápidos pelo meio, muitos homens perto da área.',
    roles:{ GK:['gk_ip','swk'], DL:['wb_ip','pfb'], DCL:['cb_ip','ccb'], DCR:['bpcb','scb'], DR:['wb_ip','pfb'], DML:['dm_ip','pcm'], DMR:['b2b','pcm'], AMCL:['ap','twm'], AMCR:['chm','twm'], STL:['dlf','tcf'], STR:['chf','tcf'] },
    ti:{ passing:3, tempo:4, reception:0, loe:2, dline:3, trigger:4, def_trans:2, width:1, short_gk:1 } },
  { id:'counter442', n:'4-4-2 Bloco Médio & Contra-ataque', ip:'4-4-2', oop:'4-4-2', ment:'Cautious',
    style:'counter', desc:'Compacto sem bola, transições rápidas para dois avançados.',
    roles:{ GK:['gk_ip','gk_oop'], DL:['fb_ip','fb_oop'], DCL:['cb_ip','cb_oop'], DCR:['cb_ip','ccb'], DR:['fb_ip','fb_oop'], ML:['w_ip','twm'], MCL:['cm_ip','scm'], MCR:['b2b','cm_oop'], MR:['iw_ip','twm'], STL:['tf','cf_oop'], STR:['chf','socf'] },
    ti:{ passing:3, att_trans:2, loe:1, dline:1, reception:0 } },
  { id:'wb3421', n:'3-4-2-1 Alas & Meios-espaços', ip:'3-4-2-1', oop:'5-4-1', ment:'Positive',
    style:'possession', desc:'Linha de 3 com alas a dar largura e dois 10 nos meios-espaços.',
    roles:{ GK:['bpgk','swk'], DCL:['cb_ip','swcb'], DC:['bpcb','ccb'], DCR:['cb_ip','cwcb'], WBL:['awb','wb_oop'], MCL:['mp','scm'], MCR:['b2b','pcm'], WBR:['wb_ip','wb_oop'], AMCL:['ap','twm'], AMCR:['am_ip','wm_oop'], ST:['cf_ip','cf_oop'] },
    ti:{ passing:1, buildup:0, reception:2, dline:3, loe:1 } },
  { id:'lowblock532', n:'5-3-2 Bloco Baixo', ip:'3-5-2', oop:'5-3-2 WB', ment:'Defensive',
    style:'low_block', desc:'Defende a área, sai rápido com dois avançados complementares.',
    roles:{ GK:['nngk','lhk'], DCL:['cb_ip','cwcb'], DC:['nncb','cb_oop'], DCR:['cb_ip','cwcb'], WBL:['wb_ip','hwb'], DM:['dlp','scm'], MCL:['cm_ip','wccm'], MCR:['b2b','wccm'], WBR:['wb_ip','hwb'], STL:['tf','cocf'], STR:['poa','socf'] },
    ti:{ passing:3, buildup:2, goal_kicks:2, att_trans:2, loe:0, dline:1, cross_eng:0, tackling:2 } },
  { id:'diamond', n:'4-4-2 Losango de Controlo', ip:'4-4-2 Diamond', oop:'4-4-2 Diamond', ment:'Positive',
    style:'possession', desc:'Superioridade central, laterais dão toda a largura.',
    roles:{ GK:['gk_ip','gk_oop'], DL:['wb_ip','fb_oop'], DCL:['bpcb','cb_oop'], DCR:['cb_ip','ccb'], DR:['wb_ip','fb_oop'], DM:['dlp','sdm'], MCL:['wcm','wccm'], MCR:['b2b','pcm'], AMC:['ap','am_oop'], STL:['dlf','cf_oop'], STR:['chf','cf_oop'] },
    ti:{ passing:1, width:3, support_runs:2, reception:2 } },
  { id:'wide4141', n:'4-1-4-1 Pressão Equilibrada', ip:'4-1-4-1 DM Wide', oop:'4-1-4-1 DM Wide', ment:'Balanced',
    style:'press_balanced', desc:'Bloco médio-alto, extremos abertos, um avançado de referência.',
    roles:{ GK:['gk_ip','swk'], DL:['ifb','hfb'], DCL:['bpcb','cb_oop'], DCR:['cb_ip','cb_oop'], DR:['wb_ip','pfb'], DM:['hb','sdm'], ML:['w_ip','wm_oop'], MCL:['am_ip','pcm'], MCR:['mp','cm_oop'], MR:['iw_ip','wm_oop'], ST:['cf_ip','tcf'] },
    ti:{ loe:1, trigger:3, press_trap:2 } },
];

// Perfis de estilo: atributos de equipa necessários (para sugestões)
const STYLE_PROFILES = {
  possession_press:{ n:'Posse + Contra-pressão', a:['pas','tec','fir','cmp','dec','wor','sta','tea','ant'] },
  possession:{ n:'Posse', a:['pas','tec','fir','cmp','dec','vis','tea'] },
  balanced:{ n:'Equilibrado', a:['dec','tea','pos','pas','wor'] },
  press_vertical:{ n:'Pressão + verticalidade', a:['wor','sta','acc','agg','otb','fin','dec'] },
  press_balanced:{ n:'Pressão moderada', a:['wor','sta','tea','ant','pos','pas'] },
  counter:{ n:'Contra-ataque', a:['pac','acc','otb','fin','pos','tea','str'] },
  low_block:{ n:'Bloco baixo', a:['pos','cnt','mar','hea','str','tea','cmp'] },
};

const GLOSSARY = [
  ['IP / OOP','In Possession / Out of Possession — com e sem bola.'],
  ['LoE','Line of Engagement: onde começa a pressão.'],
  ['Meio-espaço','Corredor entre o centro e a ala (half-space).'],
  ['Zona 14','Zona central imediatamente à frente da área adversária.'],
  ['Rest-defence','Jogadores que ficam atrás para travar o contra-ataque.'],
  ['Outlet','Jogador que fica à frente sem bola para receber em transição.'],
  ['Overlap / Underlap','Apoio por fora / por dentro do colega da ala.'],
  ['Counter-press','Pressão imediata após a perda da bola.'],
  ['Bloco baixo/médio/alto','Altura onde a equipa defende organizada.'],
  ['Tactical Tweaks','Atalhos do FM27 que mudam várias instruções de uma vez durante o jogo.'],
  ['Tactical Complexity','Medida do FM27 de quão difícil a tática é de aprender.'],
];
