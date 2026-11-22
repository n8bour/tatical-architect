# Cobertura de idioma: com a app em inglês, nenhum texto visível (nem mensagens temporárias) pode ficar em português.
# Uso: python3 tests/ui_i18n.py
import os, sys, re, json, glob
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ui_layout as U
from playwright.sync_api import sync_playwright
HERE = U.HERE
PTW = set("""de da do das dos para com sem os ao aos um uma que por mais menos em na nos nas se ou já não ver ler abrir fechar guardar nova novo limpar usar gerar plantel jogador jogadores equipa tática táticas treino bíblia papel papéis
instrução instruções posição posições ligar porta jogos golos ficheiro carregar escolher importar exportar apagar tudo todos cada quando onde como isso isto este esta aqui sobre entre até após antes depois ainda também só muito
pouco bem melhor pior alto baixo médio média atrás frente meio lado flanco corredor área bola linha linhas pressão bloco defesa ataque avançado avançados centrais lateral laterais extremo extremos médios fica ficam faz pode
deve vai sobe entra sai joga início definições dados tema sistema claro escuro idioma cópia segurança restaurar versão ligado desligado automático encontrado olá vamos próximo três passos computador corrigir primeiro
completo analisador relatório adversário mentalidade formação estilo campo animado bolas paradas ficha construção progressão último terço perda recuperação nome objectivo estrela titulares jovens opções recomendada
alternativa adequação equilíbrio diagnóstico marcadores cantos livres penáltis capitão ideias ofensivo defensivo misto experiências motor ligação procurar veredicto sementes referência vitórias pontos remates posse
semana dia prioridades mentoria foco chave preferidos comportamento""".split())
NAMES = "Adam Clark|Ben Carter|Chris Doyle|Dan Evans|Ed Fox|Frank Hall|George King|Harry Lane|Ian Moss|Jack Nash"
def is_pt(s):
    if s.strip() in {'o', 'os', 'as', 'do', 'da', 'dos', 'das', 'e', 'ou', 'no', 'na'} and s.strip() != 'no': return True   # artigos soltos entre links
    if re.search(r"[ãõçâêôàÃÕÇÂÊÔÀ]|ção|ões", s): return True
    w = re.findall(r"[a-zà-ú]+", s.lower())
    return sum(x in PTW for x in w) >= (1 if len(w) <= 3 else 2)
SETUP = U.SETUP.replace("const names = [", "const names = ['Adam Clark','Ben Carter','Chris Doyle','Dan Evans','Ed Fox','Frank Hall','George King','Harry Lane','Ian Moss','Jack Nash'];\n  const _x = [")
SETUP = SETUP.replace("ST.myClub = 'K Beerschot Voetbalclub Antwerpen'", "ST.myClub = 'Test FC'").replace("ST.squad = { ...EMPTY_SQUAD(), players: ps, source: 'Beerschot - Época 2027-28 (save muito comprido).fm' };", "ST.squad = { ...EMPTY_SQUAD(), players: ps, source: 'squad.csv' };")
SETUP = SETUP.replace("ST.tactic.name = 'Uma tática com um nome bastante comprido para testar os cabeçalhos da aplicação';", "ST.tactic.name = 'Test tactic';")
GRAB = r"""() => { const out = []; const w = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT, { acceptNode: n => (n.parentNode && /^(SCRIPT|STYLE|TEXTAREA)$/.test(n.parentNode.nodeName)) ? 2 : 1 });
  while (w.nextNode()) { const v = w.currentNode.nodeValue.trim(); if (v) out.push(v); }
  document.querySelectorAll('[title],[placeholder],[aria-label]').forEach(e => ['title','placeholder','aria-label'].forEach(a => { const v = e.getAttribute(a); if (v && v.trim()) out.push(v.trim()); }));
  return out; }"""
texts = set(); errors = []
with sync_playwright() as p:
    b = p.chromium.launch()
    for desktop in (False, True):
        pg = b.new_page(viewport={'width': 1600, 'height': 1000}); pg.on('pageerror', lambda e: errors.append(str(e)))
        if desktop:
            pg.add_init_script("""window.__TAURI__={core:{invoke:async(c,a)=>{ if(c==='export_dirs')return [['C:/x/FM View Exporter',false]]; if(c==='latest_save')return {path:'C:/x/T.fm',name:'T.fm',modified:5,size:104857600};
              return null; }}};""")
        pg.goto(U.APP); pg.wait_for_timeout(200)
        pg.evaluate("() => { ST.lang = LANG = 'en'; render(); }"); [texts.add(x) for x in pg.evaluate(GRAB)]
        pg.evaluate(SETUP); pg.evaluate("t => ingestSquadText(t, 'rival.csv', { force: 'opp' })", U.RIVAL)
        pg.evaluate("() => { genOpts(); A.genRun(); }"); pg.wait_for_timeout(1000)
        pg.evaluate("t => A.scoutIngest(t, { save: 'db.csv' })", U.SCOUT_DB); pg.wait_for_function("SCOUT.db", timeout=20000)
        for k in ['cheap_wonder', 'reach', 'upgrade', 'weak_spot']:
            pg.evaluate(f"() => {{ A.scoutPreset('{k}'); A.scoutOpen(scoutResults()[0].id); }}"); [texts.add(x) for x in pg.evaluate(GRAB)]; pg.evaluate("A.scoutClose()")
        pg.evaluate("() => { A.scoutTab('list'); }"); [texts.add(x) for x in pg.evaluate(GRAB)]; pg.evaluate("() => { A.scoutTab('results'); }")
        if desktop: pg.evaluate("() => { ST.auto.save = true; ST.auto.saveErr = 'x'; ST.lastSave = { name: 'T.fm', size: 1e8 }; ST.f9Found = true; render(); }")
        for v, t in U.VIEWS:
            pg.evaluate(f"A.go('{v}'{', ' + repr(t) if t else ''})"); pg.wait_for_timeout(80); [texts.add(x) for x in pg.evaluate(GRAB)]
            if (v, t) == ('tactic', 'analyze'): pg.evaluate("A.sel(3)"); [texts.add(x) for x in pg.evaluate(GRAB)]; pg.evaluate("A.sel(3)")
            if (v, t) == ('tactic', 'sim'):
                for i in range(6): pg.evaluate(f"A.simPhase({i})"); [texts.add(x) for x in pg.evaluate(GRAB)]
        pg.evaluate("() => { ST.openClips = {}; ROLES.forEach(r => ST.openClips['role:' + r.id] = 1); SIM_TI.forEach(k => ST.openClips['ti:' + k] = 1); }")
        for t in ('roles', 'ti'): pg.evaluate(f"A.go('bible','{t}')"); pg.wait_for_timeout(200); [texts.add(x) for x in pg.evaluate(GRAB)]
        if not desktop:
            src = ''.join(open(f, encoding='utf8').read() for f in glob.glob(os.path.join(HERE, '..', 'src', 'ui', '*.js')))
            lits = sorted({m.group(1)[1:-1] for m in re.finditer(r"(?:toast|busyStart|confirm|withBusy|T)\(\s*(?:'[^']*',\s*)?('(?:[^'\\]|\\.)*')", src)})
            trans = pg.evaluate("ls => ls.map(x => T(x))", lits)
            texts.update(l for l, t in zip(lits, trans) if t == l)
        pg.close()
    b.close()
# caminhos raros (erros e acções): forçados com uma app instalada simulada que falha de propósito
with sync_playwright() as p:
    b = p.chromium.launch(); pg = b.new_page(viewport={'width': 1400, 'height': 900}); pg.on('pageerror', lambda e: errors.append(str(e)))
    pg.add_init_script("""window.__TOASTS = []; window.__TAURI__ = { core: { invoke: async (c, a) => {
        if (c === 'export_dirs') return []; if (c === 'latest_save') return { path: 'C:/x/T.fm', name: 'T.fm', modified: 9, size: 1e8 };
        if (c === 'export_save') throw 'Não consegui correr o fmsave. uvx: program not found | fmsave: program not found';
        if (c === 'latest_export') return null;
        return null; } } };""")
    pg.goto(U.APP); pg.wait_for_timeout(200)
    pg.evaluate("() => { ST.lang = LANG = 'en'; const o = toast; toast = (m) => { o(m); window.__TOASTS.push(document.getElementById('toast').textContent); }; render(); }")
    grab = lambda: [texts.add(x) for x in pg.evaluate(GRAB) + pg.evaluate("window.__TOASTS")]
    pg.evaluate("() => { ST.auto.save = true; A.saveNow(); }"); pg.wait_for_timeout(600); pg.evaluate("A.tab('team','squad')"); grab(); pg.evaluate("A.go('settings')"); grab()
    pg.evaluate("() => { ingestJSON({ foo: 1 }, 'x.json'); ingestSquadText('nada aqui', 'vazio.csv'); A.autoXI(); A.resetTI(); A.saveTactic(); A.sampleSquad(); A.applyAllPIs(); }"); pg.wait_for_timeout(200); grab()
    pg.evaluate("() => { A.tab('team','opp'); }"); pg.evaluate("t => ingestSquadText(t, 'rival.csv', { force: 'opp' })", U.RIVAL); pg.evaluate("() => { render(); A.applyPlan('all'); }"); grab()
    pg.evaluate("() => { genOpts(); return A.genRun(); }"); pg.wait_for_timeout(1500); pg.evaluate("() => { A.genUse(); }"); pg.wait_for_timeout(200); grab()
    b.close()
left = sorted(s for s in texts if is_pt(s) and not re.search(NAMES, s))
for s in left: print('  EM PORTUGUÊS:', s[:150])
print('cobertura de idioma (inglês):', 'OK' if not left and not errors else f'{len(left)} textos em português, {len(errors)} erros JS')
for e in errors[:3]: print('  JS:', e)
sys.exit(1 if left or errors else 0)
