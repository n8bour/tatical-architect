# FM27 Tactical Bible

Analisador tático, campo animado e bíblia de papéis e instruções para o Football Manager 27. Funciona offline e não precisa de conta.

## Como usar (3 passos)

1. **Plantel, lido do save.**
   - **App instalada:** grave o jogo no FM27 e a app lê o save sozinha (verifica a cada 10 segundos). Na primeira vez é preciso instalar o leitor:
     ```
     winget install --id=astral-sh.uv -e
     ```
     Reinicie a app depois de instalar. Na primeira leitura o `uv` descarrega o [fmsave](https://github.com/rhiever/fmsave) e precisa de internet só nessa vez.
   - **Browser:** abra o [fmsave online](https://rhiever.github.io/fmsave/app/), largue lá o seu `.fm` (o save é lido no seu PC e nunca é enviado), descarregue o CSV do plantel e arraste-o para a app.
   - **Alternativa:** carregue em F9 no jogo com o plugin [FM View Exporter](https://github.com/ElectroHugin/FM-View-Exporter). Na app instalada também é automático.
2. **Tática.** Em **A minha equipa › Gerar tática** a app monta uma tática completa para o plantel. A "Cenas v2" (4-2-4 DM Wide) também vem carregada. Em **Tática › Analisar** clique num jogador para mudar papéis e instruções. **Campo animado** mostra a tática em movimento.
3. **Adversário.** Abra o plantel do próximo adversário no FM e carregue **F9**. A app vê que é outro clube e gera o relatório em **A minha equipa › Adversário**. O plano de jogo aplica-se à tática com um clique.

Para levar a tática para o jogo, use **Tática › Ficha para o FM27**. A ficha segue a ordem do ecrã de táticas e tem caixas para ir marcando.

## O que tem

| Secção | O que faz |
|---|---|
| Início | Estado dos 3 passos, o que corrigir primeiro, o sistema que mais serve o plantel e o resumo do adversário |
| A minha equipa › Plantel | Leitura do save (fmsave) ou importação (CSV/HTML/JSON do FM View Exporter, FMPT Chief Analyst ou FMLibreEditor) e os melhores papéis de cada jogador |
| A minha equipa › **Gerar tática** | Tática completa feita à volta do plantel. Testa todos os pares de formações com e sem bola, escolhe o papel IP/OOP de cada posição com regras de equilíbrio e o XI exato (algoritmo húngaro). Gera as 27 instruções de equipa, cada uma com a razão, comparando cada setor com a média do plantel, e as instruções individuais onde os atributos as pedem. Inclui bolas paradas, capitães, profundidade por posição, onde reforçar, 3 alternativas e treino. Tem em conta o objetivo na liga, titulares obrigatórios, um jogador-estrela, até 3 jovens a desenvolver e o pé fraco nas alas. Botões para usar a tática, ver em campo ou copiar como texto |
| A minha equipa › Melhor XI | Adequação por posição, papel alternativo melhor, **instruções individuais recomendadas pelos atributos**, suplente e sucessor (≤ 21 anos), e 9 sistemas ordenados pelo plantel |
| A minha equipa › Adversário | 9 setores comparados com o teu XI, os 3 jogadores mais perigosos com instruções de adversário e um plano de ajustes aplicável |
| **Laboratório** | Ligação ao [FM Tactics Lab](https://github.com/lqf0624/fm-tactics-lab) (MIT), que corre o **motor de jogo original do FM27** no seu PC. Lista as experiências e mostra cada uma com estatística a sério: A e B comparados jogo a jogo com a mesma semente, saldo de xG, golos e pontos com intervalo de confiança de 95%, veredicto e quantos jogos faltam quando ainda não dá para concluir. Botões para **correr mais jogos** (continua as sementes) e **exportar `.fmf`** da tática, gerado e verificado pelo próprio jogo e pronto a importar no FM27. No browser, importa o JSON de uma experiência |
| Tática › Analisar | Formação com e sem bola, papéis, instruções e diagnóstico (rest-defence, largura, área, pressão, transições, complexidade FM27, adequação) |
| Tática › Campo animado | 6 fases do jogo com o movimento de cada papel, mais o efeito das instruções em tempo real |
| Tática › Bolas paradas | Marcadores (cantos com o pé certo, livres diretos e cruzados, penáltis, lançamentos longos), capitão e vice, canto ofensivo e defensivo num diagrama (zonas, marcações aos mais fortes no ar do adversário, quem fica atrás e quem fica na frente) e dicas aplicáveis |
| Tática › Ficha para o FM27 | Passo-a-passo para recriar a tática no jogo, incluindo a secção *Set Pieces & Captains* |
| Treino | Prioridades de Tactical Understanding, semana-tipo, Role Training e foco por jogador |
| Bíblia | 77 papéis (com **▶ Ver em campo**), instruções (incluindo as novas do FM27), conceitos e Tactical Effectiveness |

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

## Limitações

- **Leitura do save.** A leitura é feita pelo [fmsave](https://github.com/rhiever/fmsave) (MIT, de Randal S. Olson), sempre sobre uma **cópia** do save. Nunca altera o save, não lê a memória do jogo e não precisa do FM aberto. O fmsave marca como "não confirmados" alguns campos do FM27. A app usa só nome, idade, clube, posições e atributos.
- **Ler a memória do jogo, como o FMPT Chief Analyst.** Não é necessário: o save tem os mesmos dados e não parte a cada patch.
- **`.fmf`:** a nossa app não decifra `.fmf` sozinha. Com o Tactics Lab aberto, o Laboratório exporta `.fmf` gerados e verificados pelo jogo. Sem ele, a tática volta ao jogo pela Ficha.
- **Pesos dos atributos.** Os papéis marcados EST usam pesos estimados. Ajustam-se em `src/core/roles.js`.
- **Campo animado.** É um modelo da intenção de cada papel e instrução, não o motor de jogo do FM.

## Estrutura

```
src/core/   roles.js · tactics_data.js · engine.js · insights.js (PIs, adversário, bolas paradas) · generator.js (gerador) · lab.js (estatística do Tactics Lab) · sim_model.js
src/ui/     state.js · components.js · sim_view.js · views_*.js · actions.js · main.js
build.mjs   junta tudo em app/index.html (ficheiro único)
tests/      npm test — motor, simulador (473 cenários), insights, .fmf real e CSV do FM View Exporter
src-tauri/  janela nativa: último save (.fm) → fmsave → plantel; última exportação F9; ponte HTTP local para o Tactics Lab
```

Não afiliado à Sports Interactive nem à SEGA.
