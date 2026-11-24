// ============================================================
// Termos do FM em português (usados quando a app está em PT).
// Papéis: 34 nomes confirmados no FM26 em português (marcados ✓); o FM27 pode ter acertos pontuais;
// por isso a Ficha para o FM27 mostra, por defeito, os nomes como no jogo em inglês.
// ============================================================
const PT_TERMS = {
  // ---- papéis — ✓ = nome confirmado no FM26 em português (táticas publicadas pelo Salão de Jogos, 2025–26);
  //      os restantes seguem o mesmo padrão do jogo (Trabalhador = Pressing, Esforçado = Tracking, de Transição = Wide Outlet,
  //      de Ligação / de Desmarcação = Central / Splitting Outlet, Invertido = Inside de lado)
  'Goalkeeper':'Guarda-Redes', /* ✓ */ 'Ball-Playing Goalkeeper':'Guarda-Redes De Construção', /* ✓ */ 'Sweeper Keeper':'Guarda-Redes Líbero', /* ✓ */
  'No-Nonsense Goalkeeper':'Guarda-Redes Limitado', 'Line-Holding Keeper':'Guarda-Redes de Linha',
  'Centre-Back':'Defesa Central', /* ✓ */ 'Ball-Playing Centre-Back':'Central De Construção', /* ✓ */ 'Covering Centre-Back':'Central De Cobertura', /* ✓ */
  'Stopping Centre-Back':'Central de Antecipação', /* ✓ */ 'No-Nonsense Centre-Back':'Central Limitado', 'Advanced Centre-Back':'Central Avançado',
  'Overlapping Centre-Back':'Central de Sobreposição', 'Covering Wide Centre-Back':'Central Aberto de Cobertura', 'Stopping Wide Centre-Back':'Central Aberto de Antecipação',
  'Full-Back':'Defesa Lateral', /* ✓ */ 'Pressing Full-Back':'Lateral Trabalhador', /* ✓ */ 'Inside Full-Back':'Lateral Invertido', 'Holding Full-Back':'Defesa Lateral Fixo',
  'Wing-Back':'Ala', /* ✓ */ 'Inside Wing-Back':'Ala Invertido', /* ✓ */ 'Advanced Wing-Back':'Ala Projetado', /* ✓ */
  'Playmaking Wing-Back':'Ala Construtor', 'Holding Wing-Back':'Ala Fixo', 'Pressing Wing-Back':'Ala Trabalhador',
  'Defensive Midfielder':'Médio Defensivo', /* ✓ */ 'Dropping Defensive Midfielder':'Médio Defensivo Recuado', /* ✓ */ 'Deep-Lying Playmaker':'Construtor De Jogo Recuado', /* ✓ */
  'Box-To-Box Midfielder':'Médio Área-A-Área', /* ✓ */ 'Box-To-Box Playmaker':'Construtor De Jogo Área-A-Área', /* ✓ */
  'Half Back':'Médio Recuado', 'Screening Defensive Midfielder':'Médio Defensivo de Proteção', 'Wide Covering Defensive Midfielder':'Médio Defensivo de Cobertura',
  'Pressing Defensive Midfielder':'Médio Defensivo Trabalhador',
  'Central Midfielder':'Médio Centro', /* ✓ */ 'Pressing Central Midfielder':'Médio Trabalhador', /* ✓ */ 'Attacking Midfielder':'Médio Entrelinhas', /* ✓ */
  'Advanced Playmaker':'Construtor De Jogo Avançado', /* ✓ */ 'Midfield Playmaker':'Construtor De Jogo Médio', 'Wide Central Midfielder':'Médio Centro Aberto',
  'Channel Midfielder':'Médio de Corredor', 'Screening Central Midfielder':'Médio Centro de Proteção', 'Wide Covering Central Midfielder':'Médio Centro de Cobertura',
  'Wide Midfielder':'Médio Ala', /* ✓ */ 'Winger':'Extremo', /* ✓ */ 'Inside Winger':'Extremo de Apoio Interior', /* ✓ */ 'Playmaking Winger':'Construtor De Jogo Aberto', /* ✓ */
  'Tracking Wide Midfielder':'Médio Ala Esforçado', /* ✓ */ 'Wide Outlet Wide Midfielder':'Médio Ala de Transição', /* ✓ */
  'Free Role':'Criador Livre', /* ✓ */ 'Second Striker':'Segundo Avançado', /* ✓ */ 'Inside Forward':'Avançado Interior', /* ✓ */ 'Wide Forward':'Ponta-de-Lança Aberto', /* ✓ */
  'Central Outlet Attacking Midfielder':'Médio Ofensivo de Ligação', 'Splitting Outlet Attacking Midfielder':'Médio Ofensivo de Desmarcação', 'Tracking Attacking Midfielder':'Médio Ofensivo Esforçado',
  'Tracking Winger':'Extremo Esforçado', 'Wide Outlet Winger':'Extremo de Transição', 'Inside Outlet Winger':'Extremo de Transição Interior',
  'Centre Forward':'Ponta-de-Lança', /* ✓ */ 'Target Forward':'Avançado De Referência', /* ✓ */ 'Deep-Lying Forward':'Avançado Recuado', /* ✓ */ 'Tracking Centre Forward':'Avançado Esforçado', /* ✓ */
  'Central Outlet Centre Forward':'Avançado De Ligação', /* ✓ */ 'Splitting Outlet Centre Forward':'Avançado de Desmarcação', /* ✓ */
  'Channel Forward':'Avançado de Corredor', 'False Nine':'Falso Nove', 'Poacher':'Ponta-de-Lança Fixo',
  // ---- mentalidades
  'Very Defensive':'Muito Defensiva', 'Defensive':'Defensiva', 'Cautious':'Cautelosa', 'Positive':'Positiva', /* ✓ */ 'Attacking':'Ofensiva', 'Very Attacking':'Muito Ofensiva',
  // ---- instruções de equipa (nomes)
  'Passing Directness':'Objetividade do Passe', 'Tempo':'Ritmo', 'Attacking Width':'Largura Ofensiva', 'Creative Freedom':'Liberdade Criativa',
  'Attacking Transition':'Transição Ofensiva', 'Time Wasting':'Queimar Tempo', 'Play for Set Pieces':'Procurar Bolas Paradas', 'Pass Reception':'Receção do Passe',
  'Build-up Strategy':'Estratégia de Construção', 'Goal Kicks':'Pontapés de Baliza', 'GK Distribution Speed':'Velocidade de Reposição do GR', 'GK Distribution':'Reposição do GR',
  'Supporting Runs':'Movimentos de Apoio', 'Dribbling':'Drible', 'Progress Through':'Progredir Por', 'Final Third Patience':'Paciência no Último Terço',
  'Shots from Distance':'Remates de Longe', 'Crossing Style':'Estilo de Cruzamento', 'Line of Engagement':'Linha de Pressão', 'Defensive Line Behaviour':'Comportamento da Linha Defensiva',
  'Defensive Line':'Linha Defensiva', 'Trigger Press':'Gatilho de Pressão', 'Defensive Transition':'Transição Defensiva', 'Tackling':'Desarme', 'Pressing Trap':'Armadilha de Pressão',
  'Short Goalkeeper Distribution':'Reposição Curta do GR', 'Cross Engagement':'Abordagem a Cruzamentos',
  // ---- opções
  'Much Shorter':'Muito Mais Curto', 'Shorter':'Mais Curto', 'More Direct':'Mais Direto', 'Much More Direct':'Muito Mais Direto', 'Much Lower':'Muito Mais Baixo', 'Lower':'Mais Baixo',
  'Higher':'Mais Alto', 'Much Higher':'Muito Mais Alto', 'Much Narrower':'Muito Mais Estreito', 'Narrower':'Mais Estreito', 'Wider':'Mais Largo', 'Much Wider':'Muito Mais Largo',
  'More Disciplined':'Mais Disciplinado', 'More Expressive':'Mais Expressivo', 'Hold Shape':'Manter a Forma', 'Counter-Attack':'Contra-Ataque', 'Keep Ball in Play':'Manter a Bola em Jogo',
  'Pass into Space':'Passe para o Espaço', 'Pass to Feet':'Passe ao Pé', 'Play Through Press':'Sair a Jogar contra a Pressão', 'Bypass Press':'Contornar a Pressão',
  'Slow Pace Down':'Abrandar', 'Distribute Quickly':'Repor Rápido', 'Centre-Backs':'Centrais', 'Full-Backs':'Laterais', 'Flanks':'Flancos',
  'Underlap':'Por Dentro', 'Overlap':'Por Fora', 'Discourage':'Desencorajar', 'Encourage':'Encorajar', 'Hit Early Crosses':'Cruzar Cedo', 'Work Ball Into Box':'Trabalhar a Bola até à Área', 'Low Crosses':'Cruzamentos Rasteiros', 'Whipped Crosses':'Cruzamentos Tensos',
  'Floated Crosses':'Cruzamentos Altos', 'Low Block':'Bloco Baixo', 'Mid Block':'Bloco Médio', 'High Press':'Pressão Alta', 'Drop Off':'Recuar', 'Step Up':'Subir',
  'Much Less':'Muito Menos', 'Much More':'Muito Mais', 'Regroup':'Reagrupar', 'Counter-Press':'Contra-Pressão', 'Stay On Feet':'Ficar de Pé', 'Get Stuck In':'Entrar Forte',
  'Invite Crosses':'Convidar Cruzamentos', 'Stop Crosses':'Impedir Cruzamentos', 'Balanced':'Equilibrado', 'Standard':'Normal', 'Short':'Curtos', 'Mixed':'Mistos', 'Long':'Longos',
  'Inside':'Por Dentro', 'Outside':'Por Fora', 'Centre':'Centro', 'Left':'Esquerda', 'Right':'Direita', 'More':'Mais', 'Less':'Menos', 'High':'Alta', 'Low':'Baixa', 'Allow':'Permitir',
  // ---- instruções individuais e de adversário
  'Shoot more':'Rematar mais', 'Shoot less':'Rematar menos', 'Dribble more':'Driblar mais', 'Dribble less':'Driblar menos', 'Take more risks':'Arriscar mais', 'Take fewer risks':'Arriscar menos',
  'Cross from byline':'Cruzar da linha de fundo', 'Cross from deep':'Cruzar de trás', 'Roam from position':'Deambular', 'Stay wider':'Ficar mais aberto', 'Sit narrower':'Fechar por dentro',
  'Hold position':'Manter posição', 'Get further forward':'Subir mais', 'Move into channels':'Procurar os corredores', 'Press more':'Pressionar mais', 'Press less':'Pressionar menos',
  'Mark tighter':'Marcar mais apertado', 'Tackle harder':'Desarmar com mais força', 'Ease off tackles':'Desarmar com cautela', 'Arrive Late':'Chegar Atrasado',
  'Attack the Defensive Line':'Atacar a Linha Defensiva', 'Target Near Post':'Alvo 1.º Poste', 'Target Centre':'Alvo Centro', 'Target Far Post':'Alvo 2.º Poste',
  'Show onto (weaker) foot':'Forçar para o pé (mais fraco)', 'Show onto weaker foot':'Forçar para o pé mais fraco', 'Close down (more/less)':'Pressionar (mais/menos)',
  'Close down more':'Pressionar mais', 'Tight marking':'Marcação apertada',
  // ---- Tactical Tweaks, treino e Effectiveness
  'More Urgency':'Mais Urgência', 'Play Longer':'Jogar Mais Longo', 'Exploit Width':'Explorar a Largura', 'Press Less':'Pressionar Menos', 'Wingers Swap Sides':'Trocar Extremos de Flanco',
  'Tactical Understanding':'Compreensão Tática', 'Tactical Effectiveness':'Eficácia Tática', 'Tactical Tweaks':'Ajustes Táticos', 'Tactical Complexity':'Complexidade Tática',
  'Player Ability':'Capacidade dos Jogadores', 'Player Buy-In':'Adesão dos Jogadores', 'Next Match Information':'Informação do Próximo Jogo', 'Projected Effectiveness':'Eficácia Projetada',
  'In-Possession Shape':'Forma com Bola', 'Out-of-Possession Shape':'Forma sem Bola', 'IP Shape':'Forma com Bola', 'OOP Shape':'Forma sem Bola', 'Passing Style':'Estilo de Passe',
  'Build Up':'Construção', 'Chance Creation':'Criação de Oportunidades', 'Match Preparation':'Preparação do Jogo', 'Match Prep':'Prep. do Jogo', 'Team Cohesion':'Coesão da Equipa',
  'Role Training':'Treino de Papel', 'Individual Development':'Desenvolvimento Individual', 'Physical':'Físico', 'Clean Slate':'Estilo Livre',
  'Team Instructions':'Instruções à Equipa' /* ✓ */, 'In Possession':'Com Bola', 'Out of Possession':'Sem Bola', 'Final Third':'Último Terço', 'Progression':'Progressão', 'Buildup':'Construção',
  'Overview':'Visão Geral', 'Team Shape':'Forma da Equipa', 'Set Pieces & Captains':'Bolas Paradas e Capitães', 'Captain':'Capitão', 'Vice Captain':'Vice-Capitão',
  'Corners (right)':'Cantos (direita)', 'Corners (left)':'Cantos (esquerda)', 'Free Kicks (direct)':'Livres (diretos)', 'Free Kicks (crossed)':'Livres (cruzados)', 'Long Throws':'Lançamentos Longos',
  'Mentality':'Mentalidade', 'Style':'Estilo',
};
// só por correspondência exacta (são palavras portuguesas comuns e não podem ser trocadas no meio de frases)
const PT_EXACT_ONLY = { 'No':'Não', 'Yes':'Sim' };

// Papéis com o mesmo nome em inglês mas nomes diferentes no FM em português
const PT_ROLE_ID = { am_oop: 'Médio Ofensivo' /* ✓ ex.: Paz (Médio Entrelinhas/Médio Ofensivo) */ };
function roleName(r) { return r ? (LANG === 'pt' && PT_ROLE_ID[r.id] ? PT_ROLE_ID[r.id] : r.name) : ''; }
