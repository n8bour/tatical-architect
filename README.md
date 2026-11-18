# FM27 Tactical Bible

Analisador tático, campo animado e bíblia de papéis e instruções para o Football Manager 27. Funciona offline e não precisa de conta.

## Como usar (3 passos)

1. **Plantel.** Há duas vias, ambas opcionais e ambas dependentes de uma ferramenta externa gratuita. Os cartões em **Plantel** e **Definições › Integrações** dizem o que cada uma **requer**, têm o link de download e mostram o estado no seu PC.
   A leitura automática do save e o carregamento automático do F9 vêm **desligados** por defeito. Ligue-os nos cartões, ou use "Ler o save agora" e "Carregar o último F9".

   **Plantel, lido do save:**
   A app corre sempre o **fmsave mais recente** (`uvx fmsave@latest`). Quando o FM27 recebe um patch, basta sair a versão do fmsave que o suporte e a app passa a usá-la sozinha. Sem internet, usa a versão que já está em cache. Quem instalou o fmsave com pip atualiza com `python -m pip install --upgrade fmsave`.
   - **App instalada:** grave o jogo no FM27 e a app lê o save sozinha (verifica a cada 10 segundos). Na primeira vez é preciso instalar o leitor:
     ```
     winget install --id=astral-sh.uv -e
     ```
     Reinicie a app depois de instalar. Na primeira leitura o `uv` descarrega o [fmsave](https://github.com/rhiever/fmsave) e precisa de internet só nessa vez.
   - **Browser:** abra o [fmsave online](https://rhiever.github.io/fmsave/app/), largue lá o seu `.fm` (o save é lido no seu PC e nunca é enviado), descarregue o CSV do plantel e arraste-o para a app.
   - **Alternativa:** carregue em F9 no jogo com o plugin [FM View Exporter](https://github.com/ElectroHugin/FM-View-Exporter). Na app instalada também é automático.
2. **Tática.** Em **A minha equipa › Gerar tática** a app monta uma tática completa para o plantel. Também pode criar a sua em **Tática › Analisar**. Em **Tática › Analisar** clique num jogador para mudar papéis e instruções. **Campo animado** mostra a tática em movimento.
3. **Adversário.** Abra o plantel do próximo adversário no FM e carregue **F9**. A app vê que é outro clube e gera o relatório em **A minha equipa › Adversário**. O plano de jogo aplica-se à tática com um clique.

Para levar a tática para o jogo, use **Tática › Ficha para o FM27**. A ficha segue a ordem do ecrã de táticas e tem caixas para ir marcando.

## O que tem

| Secção | O que faz |
|---|---|
| Início | Estado dos 3 passos, o que corrigir primeiro, o sistema que mais serve o plantel e o resumo do adversário |
| A minha equipa › Plantel | Leitura do save (fmsave) ou importação (CSV/HTML/JSON do FM View Exporter, FMPT Chief Analyst ou FMLibreEditor) e os melhores papéis de cada jogador |
| A minha equipa › **Gerar tática** | Tática completa feita à volta do plantel. Testa todos os pares de formações com e sem bola, escolhe o papel IP/OOP de cada posição com regras de equilíbrio e o XI exato (algoritmo húngaro). Gera as 27 instruções de equipa, cada uma com a razão, comparando cada setor com a média do plantel, e as instruções individuais onde os atributos as pedem. Inclui bolas paradas, capitães, profundidade por posição, onde reforçar, 3 alternativas e treino. Tem em conta o objetivo na liga, titulares obrigatórios, um jogador-estrela, até 3 jovens a desenvolver e o pé fraco nas alas. Botões para usar a tática, ver em campo ou copiar como texto |
| **Sinergia entre papéis — todas as fases** (Gerador e Analisador) | Mede se os papéis se servem e se cobrem uns aos outros, e não só se cada jogador está no seu melhor papel, com os atributos reais. **Construção:** quem sai a jogar → quem recebe, apoios entre linhas, resistência à pressão, bola longa → referência. **Ataque:** criador → finalizador, cruzamentos → alvos, dupla da frente, presença na área, flancos, carga criativa. **Defesa:** quem pressiona tem cobertura, dupla de centrais (um sai, outro cobre), proteção à frente dos centrais, defesa dos flancos (extremo que fica na frente obriga o lateral a segurar), linha vs. perfis, jogo aéreo. **Transições:** rest-defence à perda, quem lança → quem arranca no contra-ataque, número de saídas. Tem dois mapas (com bola e sem bola) e os problemas por fase, cada um com a correção. O Gerador pesa 55% adequação, 30% sinergia e 15% equilíbrio, e nos testes melhora todas as fases sem sacrificar a defesa |
| **Prospeção** (Scouting) | Todos os jogadores do save, lidos pelo fmsave (`export players --all`) sobre uma cópia, com cache até o save mudar. Inclui atributos, CA/PA, personalidade, atributos ocultos, contrato, salário e valor. **11 pesquisas pré-definidas:** interessados em vir (estimativa), wonderkids baratos, wonderkids que vão atingir o potencial, contrato a terminar, jogadores livres, pechinchas, reforços para a minha tática, veteranos a custo zero, joias escondidas, personalidade de elite e a minha posição mais fraca. **Filtros:** nome, idade, CA/PA, posição (competente ou natural), pé, altura, clube, país da liga, nacionalidade, valor, salário, fim de contrato, reputação do clube, livres, empréstimos, personalidade, ocultos, probabilidade de atingir o potencial, probabilidade de querer vir para o teu clube (estimativa), atributos mínimos (vários) e papel com adequação mínima. Tabela ordenável e paginada, ficha do jogador (atributos, melhores papéis e onde entraria no teu XI), lista de observação e exportação CSV. No browser, aceita o CSV de todos os jogadores do fmsave online. A **probabilidade de querer vir** é uma estimativa (o fmsave não lê o “interessado” do jogo). Usa a reputação do teu clube contra a do clube dele, a reputação dele, o contrato (livre, a terminar, estatuto no plantel), a ambição e a lealdade, e mostra as razões na ficha |
| A minha equipa › **Plantel** | A tabela dos jogadores tem três vistas: **Papéis** (melhores papéis com e sem bola), **Habilidade e contrato** (CA, PA, probabilidade de atingir o PA, valor, salário, fim de contrato, estatuto) e **Personalidade e ocultos** (profissionalismo, ambição, pressão, lealdade, temperamento, adaptabilidade, consistência, jogos importantes, propensão a lesões, versatilidade). Todas as colunas se ordenam e cada jogador abre a ficha completa. Estes dados vêm do save lido pelo fmsave; o F9 e outros CSV trazem só o que a vista exportar |
| A minha equipa › Melhor XI | Adequação por posição, papel alternativo melhor, **instruções individuais recomendadas pelos atributos**, suplente e sucessor (≤ 21 anos), e 9 sistemas ordenados pelo plantel |
| A minha equipa › Adversário | 9 setores comparados com o teu XI, os 3 jogadores mais perigosos com instruções de adversário e um plano de ajustes aplicável |
| **Laboratório** | Ligação ao [FM Tactics Lab](https://github.com/lqf0624/fm-tactics-lab) (MIT), que corre o **motor de jogo original do FM27** no seu PC. Lista as experiências e mostra cada uma com estatística a sério: A e B comparados jogo a jogo com a mesma semente, saldo de xG, golos e pontos com intervalo de confiança de 95%, veredicto e quantos jogos faltam quando ainda não dá para concluir. Botões para **correr mais jogos** (continua as sementes) e **exportar `.fmf`** da tática, gerado e verificado pelo próprio jogo e pronto a importar no FM27. No browser, importa o JSON de uma experiência |
| Tática › Analisar | Formação com e sem bola, papéis, instruções e diagnóstico (rest-defence, largura, área, pressão, transições, complexidade FM27, adequação) |
| Tática › Campo animado | 6 fases do jogo com o movimento de cada papel, mais o efeito das instruções em tempo real |
| Tática › **Situações de jogo** | Planos B/C a partir da tática atual e do plantel, em cinco cenários: **Precisamos de golos**, **Segurar o resultado**, **Fora contra mais fortes**, **Fora contra equipa do mesmo nível** e **Em casa contra mais fracos** (bloco baixo). Cada plano diz o que muda e porquê, com os atributos dos jogadores: mentalidade, instruções de equipa, papéis, instruções individuais (as que se acrescentam e as que se retiram), Tactical Tweaks, substituições vindas do banco e instruções de adversário para os três mais perigosos dele. As trocas de papel são testadas contra a sinergia: num plano para atacar só entram se não piorarem o ataque, e num plano para defender só entram se não piorarem a defesa. O impacto mostra o compromisso ofensivo e a segurança defensiva (a intensidade) e a sinergia por fase (a coerência). Pode guardar o plano como tática, usá-lo já ou copiá-lo como texto |
| Tática › Bolas paradas | Marcadores (cantos com o pé certo, livres diretos e cruzados, penáltis, lançamentos longos), capitão e vice, canto ofensivo e defensivo num diagrama (zonas, marcações aos mais fortes no ar do adversário, quem fica atrás e quem fica na frente) e dicas aplicáveis |
| Tática › Ficha para o FM27 | Passo-a-passo para recriar a tática no jogo, incluindo a secção *Set Pieces & Captains* |
| Treino | Prioridades de Tactical Understanding, semana-tipo, Role Training e foco por jogador |
| Bíblia | 77 papéis e instruções (incluindo as novas do FM27), conceitos e Tactical Effectiveness. Cada papel tem um **clip** com o movimento individual desse jogador. Cada instrução animada tem um **clip duplo** que mostra, lado a lado, o efeito da opção mínima e da máxima |

Pode arrastar qualquer ficheiro para qualquer sítio da janela. Plantéis do teu clube vão para o Plantel; plantéis de outro clube vão para o Adversário; ficheiros `.fmf` vão para o inspector; cópias de segurança `.json` são restauradas.

## Instalar

**Sem instalar nada.** Abra `app/index.html` com duplo clique. Tem tudo, excepto o carregamento automático do F9.

**App para Windows (GitHub Actions, sem instalar nada no PC):**
1. Crie um repositório e envie estes ficheiros **para a raiz**. Confirme que vê `.github/`, `app/`, `src/`, `src-tauri/` e `build.mjs`.
2. O build corre sozinho a cada push para o `main`. Também pode lançá-lo em **Actions › Build desktop app › Run workflow**.
3. No fim, abra **Releases › "FM27 Tactical Bible — última build"** (coluna da direita da página principal do repositório) e descarregue de *Assets* o `…setup.exe` (instalador) ou o `FM27-Tactical-Bible-portatil.exe` (corre sem instalar).

Duração: cerca de 6 a 8 minutos na primeira vez e 2 a 3 minutos nas seguintes, graças à cache. Por defeito só compila para Windows. Para macOS e Linux, marque a opção no *Run workflow* ou crie uma tag `v*`.

**Local** (precisa de Node, Rust e das Build Tools C++ do Visual Studio): corra `npm run app:build`.

## Laboratório (motor do FM27)
1. Instale e abra o **FM27 Tactics Lab**. O Steam tem de estar aberto com a conta que tem o FM27.
2. Em **Laboratório**, carregue em **Ligar**. A app procura o servidor local sozinha (por defeito na porta 8765).
3. No Tactics Lab, monte A (referência) e B (alteração) ou carregue o save, e corra a comparação. Na nossa app vê o veredicto, pode **correr mais jogos** e **exportar o `.fmf`** da vencedora.

A ligação passa pelo lado nativo da app, porque o servidor do Tactics Lab recusa pedidos vindos de outras páginas. Só fala com `127.0.0.1` e só usa caminhos `/api/`.

## Idioma
A app abre em **inglês** por defeito. O português escolhe-se no seletor **EN/PT** no fundo do menu, e a escolha fica guardada.
- **Tradução completa:** cobre a interface, a Bíblia (descrições e dicas dos 77 papéis, instruções, conceitos, treino) e as mensagens do diagnóstico, do gerador, do relatório do adversário, das bolas paradas e do Laboratório.
- **Gerador:** ao trocar de idioma, as razões do relatório são recalculadas na nova língua.
- **Nomes do FM em português:** com a app em português, os papéis, as instruções e as respetivas opções, as mentalidades, as instruções individuais e de adversário, os Tactical Tweaks e as áreas de treino aparecem com nomes portugueses, também dentro das frases. Nos papéis, 34 nomes estão **confirmados no FM26 em português**, a partir de táticas publicadas pelo Salão de Jogos, e estão marcados com ✓ em `src/i18n/pt_terms.js`. Os restantes seguem o mesmo padrão de nomes do jogo. O FM27 pode ter acertos pontuais.
- **Ficha para o FM27 em português:** por defeito mantém os nomes como aparecem no jogo em inglês, para recriar a tática sem enganos. O botão **Português** mostra-a com os nomes traduzidos.
- **Pesquisa da Bíblia:** encontra os papéis pelo nome nas duas línguas.
- **Testes automáticos:** `tests/ui_i18n.py` falha o build se algum texto ficar em português com a app em inglês, incluindo erros e mensagens temporárias. `tests/ui_pt_terms.py` falha se algum termo do FM ficar em inglês com a app em português. `tests/i18n_audit.py` é uma auditoria estática de apoio. `tests/ui_lang_audit.py` é a auditoria nos dois sentidos: tira o vocabulário de cada língua do próprio código e procura português com a app em inglês e inglês com a app em português, em todos os ecrãs e vários estados. Antes de correr, verifica que os detetores apanham exemplos conhecidos.

## Interface
- **Painéis recolhíveis:** clique no ▾ (ou no título) de qualquer painel para o fechar ou abrir. A app lembra-se do estado em cada ecrã.
- **Menu só com ícones:** botão « no topo do menu ou **Ctrl+B**. Em janelas estreitas (por exemplo, 1366 com escala de 125%) o menu passa a ícones sozinho.
- **Dados do jogo sempre à mão:** o painel "Dados do jogo" na barra lateral tem os botões **📥 Ler o save agora** e **Último F9**, e diz há quanto tempo o plantel foi lido. Os mesmos botões aparecem no topo do Plantel, no passo 1 do Início e nos ecrãs que pedem o plantel. Atalho: **Ctrl+Shift+L**. No browser, o botão abre o ficheiro do plantel.
- **Arrastar e largar** qualquer ficheiro para a janela.
- **Atividade sempre visível:** tudo o que demora mostra que está a trabalhar. Aparece uma barra fina no topo da janela e um cartão na barra lateral com o que está a acontecer, o tempo decorrido e, quando se sabe, a percentagem. O cartão nunca tapa conteúdo. Os botões dessa tarefa mostram um spinner e ficam bloqueados até acabar. Está ligado à leitura do save (com aviso quando a primeira leitura demora), ao F9, a ficheiros grandes, ao Gerador (progresso par a par), à análise de `.fmf` e ao Laboratório (procurar o servidor, abrir experiências, exportar `.fmf` e jogos a correr no motor).
- **Testes de interface no GitHub:** `ui_layout.py` corre em inglês completo e numa passagem rápida em português. `ui_busy.py` verifica os indicadores de atividade e `ui_i18n.py` a cobertura de idioma.
- **Portão de layout:** `tests/ui_layout.py` percorre os 17 ecrãs em 6 larguras e 3 modos (normal, menu recolhido, painéis fechados) com um plantel de 62 jogadores de nomes longos. Corre no GitHub em cada build, e o Release não é publicado se encontrar sobreposições ou erros.

## Papéis disponíveis por posição
- **Casa exata:** no FM26/27 alguns papéis só existem na casa central de uma linha (DM, MC, AMC, ST) ou só nas casas deslocadas (DML/DMR, MCL/MCR…). A app respeita isto no analisador, no campo animado, no Gerador e nos modelos. Um papel que o jogo não permite naquela casa não aparece na lista e é assinalado como inválido.
- **Confirmado:**
  - O Box-To-Box Midfielder e o Box-To-Box Playmaker não existem no médio centro, só na linha de DM.
  - O Box-To-Box Midfielder só existe no DM deslocado (DML/DMR).
- **Validação completa:** o catálogo do motor do FM27 (Tactics Lab › Laboratório › "Exportar catálogo do motor") traz a lista exata de papéis por posição. A app tem o ponto de ligação preparado (`ROLE_AVAIL_ENGINE`) para aplicar essa lista a todos os 77 papéis.

## Limitações

- **Leitura do save.** A leitura é feita pelo [fmsave](https://github.com/rhiever/fmsave) (MIT, de Randal S. Olson), sempre sobre uma **cópia** do save. Nunca altera o save, não lê a memória do jogo e não precisa do FM aberto. O fmsave marca como "não confirmados" alguns campos do FM27. A app usa só nome, idade, clube, posições e atributos.
- **Ler a memória do jogo, como o FMPT Chief Analyst.** Não é necessário: o save tem os mesmos dados e não parte a cada patch.
- **`.fmf`:** a nossa app não decifra `.fmf` sozinha. Com o Tactics Lab aberto, o Laboratório exporta `.fmf` gerados e verificados pelo jogo. Sem ele, a tática volta ao jogo pela Ficha.
- **Pesos dos atributos.** Os papéis marcados EST usam pesos estimados. Ajustam-se em `src/core/roles.js`.
- **Campo animado.** É um modelo da intenção de cada papel e instrução, não o motor de jogo do FM.

## Estrutura

```
src/i18n/   i18n.js (L/T/translateDOM) · en_dict.js (dicionário PT→EN)
src/core/   roles.js · tactics_data.js · engine.js · insights.js (PIs, adversário, bolas paradas) · generator.js (gerador) · lab.js (estatística do Tactics Lab) · sim_model.js
src/ui/     state.js · components.js · sim_view.js · views_*.js · actions.js · main.js
build.mjs   junta tudo em app/index.html (ficheiro único)
tests/      npm test — motor, simulador (473 cenários), insights, .fmf real e CSV do FM View Exporter
src-tauri/  janela nativa: último save (.fm) → fmsave → plantel; última exportação F9; ponte HTTP local para o Tactics Lab
```

Não afiliado à Sports Interactive nem à SEGA.
