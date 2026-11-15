// ============================================================
// Definições + cartões de integrações (dependências externas explícitas)
// ============================================================
const LINKS = {
  fmve: 'https://github.com/ElectroHugin/FM-View-Exporter/releases',
  fmsave: 'https://github.com/rhiever/fmsave',
  fmsaveWeb: 'https://rhiever.github.io/fmsave/app/',
  uv: 'https://docs.astral.sh/uv/getting-started/installation/',
  lab: 'https://github.com/lqf0624/fm-tactics-lab/releases',
};
// compact: versão curta (Plantel); completa nas Definições
function integrationsHTML(compact) {
  const f9 = ST.f9Found, save = ST.auto.save, err = ST.auto.saveErr;
  const pill = (ok, txtOk, txtNo) => `<span class="pill ${ok ? 'ok' : 'no'}">${ok ? txtOk : txtNo}</span>`;
  const card = (title, pills, body, actions) => `<div class="integ"><div class="integ-h"><b>${title}</b><span class="row" style="gap:6px">${pills}</span></div>${body}<div class="row" style="margin-top:8px">${actions}</div></div>`;
  const fmsaveCard = card('Ler o save (.fm) — fmsave',
    DESKTOP ? pill(save, 'ligado', 'desligado') + (err ? pill(false, '', 'falta instalar') : '') : pill(false, '', 'só na app instalada'),
    `<p class="small">Lê o plantel directamente do save, sempre sobre uma cópia, sem alterar nada. <b>Requer</b> o <a href="${LINKS.uv}" target="_blank" rel="noopener">uv</a> (instala-se uma vez com <code>winget install --id=astral-sh.uv -e</code>); o <a href="${LINKS.fmsave}" target="_blank" rel="noopener">fmsave</a> (MIT) é descarregado sozinho na primeira leitura.</p>
     ${!DESKTOP ? `<p class="small">No browser: use o <a href="${LINKS.fmsaveWeb}" target="_blank" rel="noopener">fmsave online</a> e arraste o CSV para aqui.</p>` : ''}
     ${DESKTOP && !compact ? `<label class="f">Pasta dos saves (opcional)<input type="text" value="${esc(ST.auto.saveDir)}" placeholder="Documentos\\Sports Interactive\\Football Manager 27\\games" onchange="A.saveDir(this.value)"></label>
       <div class="small">${ST.lastSave ? `<span>Último save:</span> <b>${esc(ST.lastSave.name)}</b> (${(ST.lastSave.size / 1048576).toFixed(0)} MB)` : 'Nenhum save encontrado ainda.'}${ST.auto.saveLast ? `<br><span>Lido:</span> ${esc(ST.auto.saveLast)}` : ''}</div>` : ''}
     ${err && DESKTOP ? `<div class="notice warn small">${esc(err)}</div>` : ''}`,
    DESKTOP ? `<label class="row small"><input type="checkbox" ${save ? 'checked' : ''} onchange="A.saveToggle()"> Ler automaticamente sempre que o FM grava</label><button class="btn sm" data-busy="save" onclick="A.saveNow()">Ler o save agora</button>` : `<a class="btn sm" href="${LINKS.fmsaveWeb}" target="_blank" rel="noopener">Abrir fmsave online</a>`);
  const f9Card = card('F9 no jogo — FM View Exporter',
    DESKTOP ? pill(f9, 'plugin encontrado', 'plugin não encontrado') + (f9 ? pill(ST.auto.on, 'automático', 'manual') : '') : '',
    `<p class="small"><b>Requer</b> o plugin gratuito <a href="${LINKS.fmve}" target="_blank" rel="noopener">FM View Exporter</a> (Windows + Steam): descarregue o <code>setup.exe</code>, clique Install, e no FM importe a vista <code>FM27-PlayerAnalyzer-Squad-Standard.fmf</code>. Depois, F9 no ecrã do plantel exporta-o${DESKTOP ? ' e a app carrega-o sozinha' : ''}. F9 no plantel de outro clube entra como adversário.</p>
     ${DESKTOP && !f9 ? '<div class="notice small">Ainda não encontrei a pasta do plugin (Documentos › Sports Interactive › FM View Exporter). Instale-o e carregue F9 uma vez no FM.</div>' : ''}
     ${DESKTOP && !compact ? `<label class="f">Pasta personalizada (opcional)<input type="text" value="${esc(ST.auto.dir)}" placeholder="ex.: D:\\Documentos\\Sports Interactive\\FM View Exporter" onchange="A.autoDir(this.value)"></label><div id="dirs" class="grid" style="gap:4px"></div>` : ''}`,
    `<a class="btn sm" href="${LINKS.fmve}" target="_blank" rel="noopener">Descarregar o plugin</a>${DESKTOP ? `<label class="row small"><input type="checkbox" ${ST.auto.on ? 'checked' : ''} onchange="A.autoToggle()"> Carregar cada F9 automaticamente</label><button class="btn sm" data-busy="f9" onclick="A.autoNow()">Carregar o último F9</button>` : ''}`);
  const labCard = compact ? '' : card('Testar no motor do FM27 — FM Tactics Lab',
    DESKTOP ? pill(!!(ST.lab && ST.lab.status), 'ligado', 'não ligado') : pill(false, '', 'só na app instalada'),
    `<p class="small"><b>Requer</b> o <a href="${LINKS.lab}" target="_blank" rel="noopener">FM27 Tactics Lab</a> aberto, com o Steam aberto na conta que tem o FM27. Corre jogos com o motor original e exporta .fmf verificados pelo jogo.</p>`,
    `<a class="btn sm" href="${LINKS.lab}" target="_blank" rel="noopener">Descarregar o Tactics Lab</a><button class="btn sm" onclick="A.go('lab')">Abrir Laboratório</button>`);
  return fmsaveCard + f9Card + labCard;
}
function vSettings() {
  if (DESKTOP) setTimeout(async () => {
    const el = document.getElementById('dirs'); if (!el) return;
    const ds = await desktopDirs(); const uniq = []; ds.forEach(([p, ok]) => { if (!uniq.some(u => u[0] === p)) uniq.push([p, ok]); });
    el.innerHTML = uniq.map(([p, ok]) => `<div class="small"><span class="dot ${ok ? 'ok' : ''}"></span><code>${esc(p)}</code>${ok ? ' — encontrada' : ''}</div>`).join('');
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
