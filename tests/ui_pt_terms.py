# Em português: nenhum termo do FM (papel, instrução, opção…) pode ficar em inglês fora da Ficha para o FM27;
# a Ficha mostra os nomes do jogo (inglês) por defeito e em português quando se escolhe.
import os, sys, re, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ui_layout as U
from playwright.sync_api import sync_playwright
fails, errors = [], []
with sync_playwright() as p:
    b = p.chromium.launch(); pg = b.new_page(viewport={'width': 1500, 'height': 950}); pg.on('pageerror', lambda e: errors.append(str(e)))
    pg.goto(U.APP); pg.wait_for_timeout(150)
    pg.evaluate("() => { ST.lang = LANG = 'pt'; }"); pg.evaluate(U.SETUP)
    pg.evaluate("t => ingestSquadText(t, 'rival.csv', { force: 'opp' })", U.RIVAL)
    pg.evaluate("() => { genOpts(); A.genRun(); }"); pg.wait_for_timeout(1000)
    terms = pg.evaluate("Object.keys(PT_TERMS).filter(k => k.length >= 8 && /\\s|-/.test(k))")   # termos compostos e inequívocos
    rx = re.compile(r"(?<![A-Za-z\-])(" + "|".join(re.escape(t) for t in sorted(terms, key=len, reverse=True)) + r")(?![A-Za-z])")
    GRAB = """() => { const out = []; const w = document.createTreeWalker(document.querySelector('main'), NodeFilter.SHOW_TEXT);
      while (w.nextNode()) { const n = w.currentNode; if (n.parentElement.closest('[data-gameterms="en"]')) continue; const v = n.nodeValue.trim(); if (v) out.push(v); }
      document.querySelectorAll('main option').forEach(o => out.push(o.textContent)); return out; }"""
    found = {}
    for v, t in U.VIEWS:
        pg.evaluate(f"A.go('{v}'{', ' + repr(t) if t else ''})"); pg.wait_for_timeout(100)
        if (v, t) == ('tactic', 'analyze'): pg.evaluate("A.sel(3)")
        for x in pg.evaluate(GRAB):
            m = rx.search(x)
            if m: found.setdefault(m.group(1), (v, t, x[:90]))
    for k, (v, t, x) in sorted(found.items()): fails.append(f'termo em inglês em {v}/{t}: «{k}» em «{x}»')
    # Ficha: nomes do jogo por defeito, português a pedido
    pg.evaluate("A.tab('tactic','sheet')"); pg.wait_for_timeout(100)
    sheet_en = pg.locator('.sheet').inner_text()
    pg.evaluate("A.sheetNames('pt')"); pg.wait_for_timeout(100); sheet_pt = pg.locator('.sheet').inner_text()
    if 'Passing Directness' not in sheet_en and 'Goalkeeper' not in sheet_en: fails.append('Ficha (nomes do jogo) sem termos em inglês')
    if 'Guarda-Redes' not in sheet_pt: fails.append('Ficha em português sem nomes portugueses')
    pg.evaluate("A.sheetNames('en')")
    b.close()
for f in fails[:40]: print('  ' + f)
print('termos do FM em português:', 'OK' if not fails and not errors else f'{len(fails)} problemas, {len(errors)} erros JS')
for e in errors[:3]: print('  JS:', e)
sys.exit(1 if fails or errors else 0)
