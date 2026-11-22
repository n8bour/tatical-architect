// ============================================================
// Definições + cartões de integrações (dependências externas explícitas)
// ============================================================
// erros do fmsave traduzidos para o que o utilizador precisa de saber
function saveErrText(err) {
  const e = String(err || '');
  if (/UnsupportedGameError|unsupported|unverified/i.test(e)) return L('O fmsave ainda não lê o plantel nesta versão do FM27 (o jogo foi actualizado). A app usa sempre o fmsave mais recente: assim que sair uma versão que a suporte, volta a funcionar sozinha. Entretanto use o F9 ou o fmsave online.', 'fmsave cannot read the squad in this FM27 build yet (the game was updated). The app always uses the latest fmsave: as soon as a version supporting it is released, it works again on its own. Meanwhile, use F9 or fmsave online.') + ' (' + T(e).slice(0, 160) + ')';
  return e;
}
const LINKS = {
  fmve: 'https://github.com/ElectroHugin/FM-View-Exporter/releases',
  fmsave: 'https://github.com/rhiever/fmsave',
  fmsaveWeb: 'https://rhiever.github.io/fmsave/app/',
  uv: 'https://docs.astral.sh/uv/getting-started/installation/',
  bridge: 'https://github.com/n8bour/tatical-architect/releases',
};
// compact: versão curta (Plantel); completa nas Definições
function integrationsHTML(compact) {
  const f9 = ST.f9Found, save = ST.auto.save, err = ST.auto.saveErr;
  const pill = (ok, txtOk, txtNo) => `<span class="pill ${ok ? 'ok' : 'no'}">${ok ? txtOk : txtNo}</span>`;
  const card = (title, pills, body, actions) => `<div class="integ"><div class="integ-h"><b>${title}</b><span class="row" style="gap:6px">${pills}</span></div>${body}<div class="row" style="margin-top:8px">${actions}</div></div>`;
  const fmsaveCard = card('Ler o save (.fm) — fmsave',
    DESKTOP ? pill(save, 'ligado', 'desligado') + (err ? pill(false, '', 'falta instalar') : '') : pill(false, '', 'só na app instalada'),
    `<p class="small">Lê o plantel directamente do save, sempre sobre uma cópia, sem alterar nada. ${L(`<b>Requer</b> o <a href="${LINKS.uv}" target="_blank" rel="noopener">uv</a> (instala-se uma vez com <code>winget install --id=astral-sh.uv -e</code>); o <a href="${LINKS.fmsave}" target="_blank" rel="noopener">fmsave</a> (MIT) é descarregado sozinho na primeira leitura.`, `<b>Requires</b> <a href="${LINKS.uv}" target="_blank" rel="noopener">uv</a> (install it once with <code>winget install --id=astral-sh.uv -e</code>); <a href="${LINKS.fmsave}" target="_blank" rel="noopener">fmsave</a> (MIT) downloads itself on the first read.`)}</p>
     ${!DESKTOP ? `<p class="small">No browser: use o <a href="${LINKS.fmsaveWeb}" target="_blank" rel="noopener">fmsave online</a> e arraste o CSV para aqui.</p>` : ''}
     ${DESKTOP && !compact ? `<label class="f">Pasta dos saves (opcional)<input type="text" value="${esc(ST.auto.saveDir)}" placeholder="Documentos\\Sports Interactive\\Football Manager 27\\games" onchange="A.saveDir(this.value)"></label>
       <div class="small">${ST.lastSave ? `<span>Último save:</span> <b>${esc(ST.lastSave.name)}</b> (${(ST.lastSave.size / 1048576).toFixed(0)} MB)` : 'Nenhum save encontrado ainda.'}${ST.auto.saveLast ? `<br><span>Lido:</span> ${esc(ST.auto.saveLast)}` : ''}</div>` : ''}
     ${err && DESKTOP ? `<div class="notice warn small">${esc(saveErrText(err))}</div>` : ''}
     <p class="small muted">${L('A app usa sempre a versão mais recente do fmsave (verifica o PyPI a cada leitura; sem internet, usa a que já tem). Se instalou o fmsave com pip, actualize com', 'The app always uses the latest fmsave (it checks PyPI on every read; offline, it uses the one you already have). If you installed fmsave with pip, update it with')} <code>python -m pip install --upgrade fmsave</code>.</p>`,
    DESKTOP ? `<label class="row small"><input type="checkbox" ${save ? 'checked' : ''} onchange="A.saveToggle()"> Ler automaticamente sempre que o FM grava</label><button class="btn primary sm" data-busy="save" onclick="A.saveNow()">📥 ${L('Ler o save agora', 'Read the save now')}</button>` : `<a class="btn sm" href="${LINKS.fmsaveWeb}" target="_blank" rel="noopener">Abrir fmsave online</a>`);
  const f9Card = card('F9 no jogo — FM View Exporter',
    DESKTOP ? pill(f9, 'plugin encontrado', 'plugin não encontrado') + (f9 ? pill(ST.auto.on, 'automático', 'manual') : '') : '',
    `<p class="small"><b>Requer</b> o plugin gratuito <a href="${LINKS.fmve}" target="_blank" rel="noopener">FM View Exporter</a> (Windows + Steam): descarregue o <code>setup.exe</code>, clique Install, e no FM importe a vista <code>FM27-PlayerAnalyzer-Squad-Standard.fmf</code>. Depois, F9 no ecrã do plantel exporta-o${DESKTOP ? ' e a app carrega-o sozinha' : ''}. F9 no plantel de outro clube entra como adversário.</p>
     ${DESKTOP && !f9 ? '<div class="notice small">Ainda não encontrei a pasta do plugin (Documentos › Sports Interactive › FM View Exporter). Instale-o e carregue F9 uma vez no FM.</div>' : ''}
     ${DESKTOP && !compact ? `<label class="f">Pasta personalizada (opcional)<input type="text" value="${esc(ST.auto.dir)}" placeholder="ex.: D:\\Documentos\\Sports Interactive\\FM View Exporter" onchange="A.autoDir(this.value)"></label><div id="dirs" class="grid" style="gap:4px"></div>` : ''}`,
    `<a class="btn sm" href="${LINKS.fmve}" target="_blank" rel="noopener">Descarregar o plugin</a>${DESKTOP ? `<label class="row small"><input type="checkbox" ${ST.auto.on ? 'checked' : ''} onchange="A.autoToggle()"> Carregar cada F9 automaticamente</label><button class="btn sm" data-busy="f9" onclick="A.autoNow()">Carregar o último F9</button>` : ''}`);
  const fb = (typeof FMB !== 'undefined' && FMB.state) || {};
  const bridgeCard = compact ? '' : card('Aplicar no FM — Tactical Bible Bridge',
    DESKTOP ? pill(fb.ok, fb.ok ? (fb.ready ? 'ligado' : 'a aguardar o jogo') : '', 'não encontrado') : pill(false, '', 'só na app instalada'),
    `<p class="small">${L(`<b>Requer</b> o plugin <a href="${LINKS.bridge}" target="_blank" rel="noopener">Tactical Bible Bridge</a> (BepInEx 6) dentro do FM27, só Windows. Se já tem o <b>FM27 Assistant</b> instalado, o plugin dele (FM27Bridge) também serve. Opcional: sem ele a app funciona igual e a Ficha para o FM27 continua a ser o caminho. Com ele, <b>Colocar o meu XI no FM</b> põe os jogadores na tática activa do jogo.`, `<b>Requires</b> the <a href="${LINKS.bridge}" target="_blank" rel="noopener">Tactical Bible Bridge</a> plugin (BepInEx 6) inside FM27, Windows only. If you already have <b>FM27 Assistant</b> installed, its plugin (FM27Bridge) works too. Optional: without it the app works the same and the FM27 sheet stays the way in. With it, <b>Place my XI in FM</b> puts the players into the game's active tactic.`)}</p>
     ${fb.ok ? `<div class="row"><button class="btn sm" onclick="A.bridgeRec()">${FMB.rec && FMB.rec.on ? '⏹ ' + L('Parar gravação', 'Stop recording') : '⏺ ' + L('Gravar uma acção no FM', 'Record an action in FM')}</button>${FMB.rec && FMB.rec.out ? `<button class="btn sm" onclick="A.bridgeRecCopy()">${L('Copiar', 'Copy')}</button>` : ''}<button class="btn sm" data-busy="bridge-dump" onclick="A.bridgeDumpSP()">📋 ${L('Copiar estado das bolas paradas', 'Copy set-piece state')}</button>${fb.port === 27027 ? `<button class="btn sm" onclick="A.bridgeSample()">${FMB.sampling ? '⏹ ' + L('Parar recolha', 'Stop sampling') : '🔎 ' + L('Recolher dados do ecrã', 'Sample screen data')}</button>` : ''}</div>
       <p class="small muted">${L('Para acrescentar acções ou rever depois de um patch: carregue em Gravar, mude essa opção à mão no FM, e carregue em Parar.', 'To add actions or re-check after a patch: press Record, change that option by hand in FM, then press Stop.')}</p>
       ${FMB.rec && FMB.rec.out ? `<textarea readonly rows="6" style="width:100%;font-family:monospace;font-size:11px">${esc(FMB.rec.out)}</textarea>` : ''}` : ''}
     ${fb.ok ? `<p class="small muted">${esc((fb.plugin || '') + ' · ' + L('porta', 'port') + ' ' + (fb.port || '') + (fb.version ? ' · v' + fb.version + ' · FM ' + (fb.fm || '?') : ''))} · ${esc(FMB_IDS.builtFor)}</p>` : (DESKTOP && fb.err ? `<p class="small muted">${esc(fb.err)}</p>` : '')}`,
    DESKTOP ? `<button class="btn sm" onclick="A.bridgeCheck()">${L('Verificar ligação', 'Check connection')}</button>${fb.ok && fb.ready ? `<button class="btn primary sm" data-busy="bridge" onclick="A.bridgeXI()">${L('Colocar o meu XI no FM', 'Place my XI in FM')}</button>` : ''}` : '');
  return fmsaveCard + f9Card + bridgeCard;
}
function vSettings() {
  if (DESKTOP) fmbRefresh('settings');
  if (DESKTOP) setTimeout(async () => {
    const el = document.getElementById('dirs'); if (!el) return;
    const ds = await desktopDirs(); const uniq = []; ds.forEach(([p, ok]) => { if (!uniq.some(u => u[0] === p)) uniq.push([p, ok]); });
    el.innerHTML = uniq.map(([p, ok]) => `<div class="small"><span class="dot ${ok ? 'ok' : ''}"></span><code>${esc(p)}</code>${ok ? L(' — encontrada', ' — found') : ''}</div>`).join('');
  }, 0);
  return `<div class="top"><div><h2>Definições</h2></div></div>
  <div class="grid g2">
    <div class="panel grid">
      <h3>Integrações</h3>
      <p class="small muted" style="margin:0">Ferramentas externas opcionais. Cada uma diz o que precisa e onde se descarrega.</p>
      ${integrationsHTML(false)}
      <h3>O meu clube</h3>
      <p class="small muted" style="margin:0">Exportações de outro clube entram como adversário.</p>
      <label class="f">Clube<input type="text" value="${esc(ST.myClub || '')}" placeholder="detectado ao carregar o plantel" onchange="A.setMyClub(this.value)"></label>
      <h3>Tema</h3><div class="seg">${[['auto','Sistema'],['light','Claro'],['dark','Escuro']].map(([v, l]) => `<button class="${ST.theme === v ? 'on' : ''}" onclick="A.theme('${v}')">${l}</button>`).join('')}</div>
    </div>
    <div class="panel grid">
      <h3>Cópia de segurança</h3><p class="small muted" style="margin:0">Tudo fica guardado neste computador.</p>
      <div class="row"><button class="btn" onclick="A.exportAll()">Exportar tudo</button><label class="btn">Restaurar<input type="file" accept=".json" hidden onchange="A.importJSON(this)"></label><button class="btn" onclick="A.wipe()">Apagar tudo</button></div>
      <h3>Sobre</h3>
      <p class="small">Papéis e instruções seguem o sistema IP/OOP do FM26 com as novidades confirmadas do FM27. As pontuações (0–100) são médias ponderadas dos atributos-chave (peso 2) e preferidos (peso 1), ajustadas pela posição — uma segunda opinião transparente, não as estrelas do jogo.</p>
      <p class="small"><b>De onde vêm os dados:</b> do save, através do <b>fmsave</b> (Randal S. Olson, MIT — lê uma cópia do save, nunca o altera nem lê a memória do jogo), ou do F9 do FM View Exporter. A app não gera .fmf (o FM27 cifra-os): as táticas voltam ao jogo pela Ficha para o FM27.</p>
      <p class="small muted">v${VERSION} · não afiliado à Sports Interactive/SEGA.</p>
    </div>
  </div>`;
}
