# Verifica que as operações longas mostram actividade e a escondem no fim (corre no GitHub com o portão de layout).
# Uso: python3 tests/ui_busy.py
import sys, json, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ui_layout as U
from playwright.sync_api import sync_playwright
CSV = open(os.path.join(U.HERE, 'fixtures', 'fmsave_players.csv'), encoding='utf8').read()
fails, errs = [], []
def ok(c, m):
    if not c: fails.append(m)
with sync_playwright() as p:
    b = p.chromium.launch()
    # gerador: indicador durante, botão bloqueado, tudo limpo no fim
    pg = b.new_page(viewport={'width': 1366, 'height': 860}); pg.on('pageerror', lambda e: errs.append(str(e)))
    pg.goto(U.APP); pg.evaluate(U.SETUP); pg.evaluate("A.tab('team','gen')")
    pg.evaluate("window.__seen=0; new MutationObserver(()=>{ if(document.querySelector('#busy').classList.contains('on')) window.__seen++; }).observe(document.querySelector('aside'),{childList:true,subtree:true,attributes:true})")
    pg.click('button[data-busy="gen"]'); pg.wait_for_timeout(40)
    ok(pg.evaluate("document.querySelector('#topbar').classList.contains('on')"), 'gerador: barra do topo não apareceu')
    ok(pg.evaluate("document.querySelector('button[data-busy=gen]').disabled"), 'gerador: botão não ficou bloqueado')
    pg.wait_for_function("!document.querySelector('#busy').classList.contains('on')", timeout=20000)
    ok(pg.evaluate("window.__seen") > 3, 'gerador: o progresso não foi actualizado')
    ok(not pg.evaluate("document.querySelector('button[data-busy=gen]').disabled"), 'gerador: botão continuou bloqueado')
    ok(pg.evaluate("!!(ST.gen && ST.gen.res)"), 'gerador: sem resultado')
    pg.close()
    # leitura do save (app instalada simulada, fmsave com 2,5 s)
    pg = b.new_page(viewport={'width': 1366, 'height': 860}); pg.on('pageerror', lambda e: errs.append(str(e)))
    pg.add_init_script(f"""window.__TAURI__={{core:{{invoke:async(c,a)=>{{ if(c==='latest_save')return {{path:'C:/x/T.fm',name:'T.fm',modified:1763200000,size:104857600}};
      if(c==='export_save'){{await new Promise(r=>setTimeout(r,2500));return {{save:'T.fm',modified:1763200000,content:{json.dumps(CSV)},runner:'uvx'}};}} return null; }}}}}};""")
    pg.goto(U.APP); pg.wait_for_timeout(1200)
    ok('A ler o save' in pg.locator('#busy').inner_text(), 'save: cartão de actividade não apareceu')
    ok(all(pg.evaluate("[...document.querySelectorAll('button[data-busy=save]')].map(b=>b.disabled)")), 'save: botões não bloqueados')
    pg.wait_for_function("!document.querySelector('#busy').classList.contains('on')", timeout=10000)
    ok(pg.evaluate("players().length") == 16, 'save: plantel não carregou')
    pg.close(); b.close()
for f in fails: print('FALHA:', f)
for e in errs[:3]: print('JS:', e)
print('indicadores de actividade:', 'OK' if not fails and not errs else 'COM FALHAS')
sys.exit(1 if fails or errs else 0)
