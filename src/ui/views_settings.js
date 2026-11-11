// ============================================================
// Definições
// ============================================================
function vSettings() {
  if (DESKTOP) setTimeout(async () => {
    const el = document.getElementById('dirs'); if (!el) return;
    const ds = await desktopDirs(); const uniq = []; ds.forEach(([p, ok]) => { if (!uniq.some(u => u[0] === p)) uniq.push([p, ok]); });
    el.innerHTML = uniq.map(([p, ok]) => `<div class="small"><span class="dot ${ok ? 'ok' : ''}"></span><code>${esc(p)}</code>${ok ? ' — encontrada' : ''}</div>`).join('');
  }, 0);
  return `<div class="top"><div><h2>Definições</h2></div></div>
  <div class="grid g2">
    <div class="panel grid">
      <h3>Ler o save (.fm)</h3>
      ${DESKTOP ? `<label class="row"><input type="checkbox" ${ST.auto.save ? 'checked' : ''} onchange="A.saveToggle()"> Ler automaticamente sempre que o FM grava</label>
        <label class="f">Pasta dos saves (opcional)<input type="text" value="${esc(ST.auto.saveDir)}" placeholder="Documentos\\Sports Interactive\\Football Manager 27\\games" onchange="A.saveDir(this.value)"></label>
        <div class="small">${ST.lastSave ? `Último save: <b>${esc(ST.lastSave.name)}</b> (${(ST.lastSave.size / 1048576).toFixed(0)} MB)` : 'Nenhum save encontrado ainda.'}${ST.auto.saveLast ? `<br>Lido: ${esc(ST.auto.saveLast)}` : ''}</div>
        ${ST.auto.saveErr ? `<div class="notice warn small">${esc(ST.auto.saveErr)}<br>A leitura usa o <b>fmsave</b> (MIT). Instale o <b>uv</b> uma vez — <code>winget install --id=astral-sh.uv -e</code> — e reinicie a app; na primeira leitura o uv descarrega o fmsave (precisa de internet só nessa vez).</div>` : ''}
        <div class="row"><button class="btn" onclick="A.saveNow()">Ler o save agora</button></div>`
      : `<p class="small">No browser: use o <a href="https://rhiever.github.io/fmsave/app/" target="_blank" rel="noopener">fmsave online</a> (lê o save no seu PC, sem enviar nada), descarregue o CSV e arraste-o para aqui. Na app instalada isto é automático.</p>`}
      <h3>Carregamento automático (F9)</h3>
      ${DESKTOP ? `<label class="row"><input type="checkbox" ${ST.auto.on ? 'checked' : ''} onchange="A.autoToggle()"> Carregar sozinho cada exportação nova do FM View Exporter</label>
        <label class="f">Pasta personalizada (opcional)<input type="text" value="${esc(ST.auto.dir)}" placeholder="ex.: D:\\Documentos\\Sports Interactive\\FM View Exporter" onchange="A.autoDir(this.value)"></label>
        <div id="dirs" class="grid" style="gap:4px"><span class="small muted">A procurar pastas…</span></div>
        <div class="row"><button class="btn" onclick="A.autoNow()">Carregar a última exportação agora</button>${ST.auto.last ? `<span class="small muted">Último: ${esc(ST.auto.last)}</span>` : ''}</div>`
      : `<p class="small">Disponível na <b>app instalada</b> (Windows). No browser, arraste o ficheiro para a janela.</p>`}
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
