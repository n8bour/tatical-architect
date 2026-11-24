// ============================================================
// Definições + cartões de integrações (dependências externas explícitas)
// ============================================================
// erros do fmsave traduzidos para o que o utilizador precisa de saber
function saveErrText(err) {
  const e = String(err || '');
  if (/UnsupportedGameError|unsupported|unverified/i.test(e)) return L('O fmsave ainda não lê o plantel nesta versão do FM27 (o jogo foi actualizado). A app usa sempre o fmsave mais recente: assim que sair uma versão que a suporte, volta a funcionar sozinha. Entretanto use o fmsave online.', 'fmsave cannot read the squad in this FM27 build yet (the game was updated). The app always uses the latest fmsave: as soon as a version supporting it is released, it works again on its own. Meanwhile, use fmsave online.') + ' (' + T(e).slice(0, 160) + ')';
  return e;
}
const LINKS = {
  fmsave: 'https://github.com/rhiever/fmsave',
  fmsaveWeb: 'https://rhiever.github.io/fmsave/app/',
  uv: 'https://docs.astral.sh/uv/getting-started/installation/',
  bridge: 'https://github.com/n8bour/tatical-architect/releases',
};
// compact: versão curta (Plantel); completa nas Definições
function integrationsHTML(compact) {
  const save = ST.auto.save, err = ST.auto.saveErr;
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
  const fb = (typeof FMB !== 'undefined' && FMB.state) || {};
  const plug = DESKTOP && !compact ? pluginHTML() : '';
  const bridgeCard = compact ? '' : card('Aplicar no FM — Tactical Bible Bridge',
    DESKTOP ? pill(fb.ok, fb.ok ? (fb.ready ? 'ligado' : 'a aguardar o jogo') : '', 'não encontrado') : pill(false, '', 'só na app instalada'),
    `${plug}<p class="small">${L(`<b>Requer</b> o plugin <a href="${LINKS.bridge}" target="_blank" rel="noopener">Tactical Bible Bridge</a> (BepInEx 6) dentro do FM27, só Windows. Opcional: sem ele a app funciona igual e a Ficha para o FM27 continua a ser o caminho. Com ele, os botões <b>Colocar XI no FM</b> (Tática › Analisar) e <b>Aplicar no FM</b> (Tática › Bolas paradas) mudam o jogo directamente.`, `<b>Requires</b> the <a href="${LINKS.bridge}" target="_blank" rel="noopener">Tactical Bible Bridge</a> plugin (BepInEx 6) inside FM27, Windows only. Optional: without it the app works the same and the FM27 sheet stays the way in. With it, the <b>Place XI in FM</b> (Tactic › Analyse) and <b>Apply in FM</b> (Tactic › Set pieces) buttons change the game directly.`)}</p>
     ${fb.ok ? `<div class="row"><button class="btn sm" onclick="A.bridgeRec()">${FMB.rec && FMB.rec.on ? '⏹ ' + L('Parar gravação', 'Stop recording') : '⏺ ' + L('Gravar uma acção no FM', 'Record an action in FM')}</button>${FMB.rec && FMB.rec.out ? `<button class="btn sm" onclick="A.bridgeRecCopy()">${L('Copiar', 'Copy')}</button>` : ''}</div>
       <p class="small muted">${L('Para acrescentar acções ou rever depois de um patch: carregue em Gravar, mude essa opção à mão no FM, e carregue em Parar.', 'To add actions or re-check after a patch: press Record, change that option by hand in FM, then press Stop.')}</p>
       ${FMB.rec && FMB.rec.out ? `<textarea readonly rows="6" style="width:100%;font-family:monospace;font-size:11px">${esc(FMB.rec.out)}</textarea>` : ''}` : ''}
     ${fb.ok ? `<p class="small muted">${esc('Tactical Bible Bridge' + (fb.version ? ' · v' + fb.version + ' · FM ' + (fb.fm || '?') : ''))} · ${esc(FMB_IDS.builtFor)}</p>` : (DESKTOP && fb.err ? `<p class="small muted">${esc(fb.err)}</p>` : '')}`,
    DESKTOP ? `<button class="btn sm" onclick="A.bridgeCheck()">${L('Verificar ligação', 'Check connection')}</button>` : '');
  return fmsaveCard + bridgeCard;
}
function vSettings() {
  if (DESKTOP) fmbRefresh('settings');
  if (DESKTOP && !FMB.pluginChecked) { FMB.pluginChecked = true; A.pluginCheck(); }
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
      <p class="small">${L('<b>De onde vêm os dados:</b> do save, através do <b>fmsave</b> (Randal S. Olson, MIT — lê uma cópia do save, nunca o altera nem lê a memória do jogo), e do FM aberto através do plugin opcional (próximo adversário, aplicar no jogo). A app não gera .fmf (o FM27 cifra-os).', '<b>Where the data comes from:</b> the save, through <b>fmsave</b> (Randal S. Olson, MIT — reads a copy of the save, never changes it or reads the game’s memory), and the running FM through the optional plugin (next opponent, applying in the game). The app doesn’t create .fmf files (FM27 encrypts them).')}</p>
      <p class="small muted">v${VERSION} · não afiliado à Sports Interactive/SEGA.</p>
    </div>
  </div>`;
}

// Plugin inside FM: the app installs it (the DLL ships inside the app). FMB.plugin = result of the native plugin_status.
function pluginHTML() {
  const p = FMB.plugin;
  if (!p) return '';
  const msg = !p.fm_dir ? L('Não encontrei o FM27 nas pastas do Steam.', 'FM27 not found in the Steam folders.')
    : !p.bepinex ? L('Falta o BepInEx 6 (IL2CPP) no FM27: instale-o a partir de builds.bepinex.dev.', 'BepInEx 6 (IL2CPP) is missing in FM27: install it from builds.bepinex.dev.')
    : p.up_to_date ? L(`Plugin v${p.version} instalado no FM27.`, `Plugin v${p.version} installed in FM27.`)
    : p.installed ? L(`Há uma versão mais recente do plugin (v${p.version}).`, `A newer plugin is available (v${p.version}).`)
    : L('O plugin ainda não está instalado no FM27.', 'The plugin is not installed in FM27 yet.');
  const can = p.fm_dir && p.bepinex && !p.up_to_date;
  return `<div class="notice small${p.up_to_date ? '' : ' warn'}"><b>${msg}</b>${p.fm_dir ? ` <span class="muted">${esc(p.fm_dir)}</span>` : ''}
    ${can ? `<div class="row" style="margin-top:6px"><button class="btn primary sm" data-busy="plugin" onclick="A.pluginInstall()">${p.installed ? L('Actualizar o plugin', 'Update the plugin') : L('Instalar o plugin no FM27', 'Install the plugin in FM27')}</button><span class="muted">${L('Com o FM fechado; depois abra o FM.', 'With FM closed; then start FM.')}</span></div>` : ''}
    ${!p.fm_dir ? `<label class="f" style="margin-top:6px">${L('Pasta do FM27', 'FM27 folder')}<input type="text" value="${esc(ST.fmDir || '')}" placeholder="D:\\SteamLibrary\\steamapps\\common\\Football Manager 27" onchange="A.pluginDir(this.value)"></label>` : ''}</div>`;
}
