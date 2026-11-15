# Auditoria estática de idioma: TODAS as frases em português do código (interface e motor) têm de estar
# dentro de L(pt, en) ou ser traduzidas pelo dicionário. Apanha mensagens que só aparecem em erros.
# Uso: python3 tests/i18n_audit.py
import os, re, sys, glob, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ui_layout as U
from playwright.sync_api import sync_playwright
ROOT = os.path.join(U.HERE, '..', 'src')
PT = re.compile(r"[ãõçáéíóúâêôàÃÕÇÁÉÍÓÚÂÊÔÀ]|\b(de|da|dos|das|para|sem|não|uma|que|mais|menos|jogador|jogadores|equipa|tática|plantel|ficheiro|abra|ligue|carregue|escolha|confirme)\b", re.I)
SKIP_FILES = ('en_dict.js', 'pt_terms.js', 'roles.js')   # roles.js: nomes de atributos (traduzidos por attrName)
LIT = re.compile(r"`(?:[^`\\]|\\.)*`|'(?:[^'\\\n]|\\.)*'")
static, templates = [], []
for f in sorted(glob.glob(os.path.join(ROOT, '**', '*.js'), recursive=True)):
    if f.endswith(SKIP_FILES): continue
    s = open(f, encoding='utf8').read()
    for m in LIT.finditer(s):
        lit = m.group(0); body = lit[1:-1]
        if not PT.search(body) or len(body) < 3: continue
        before = s[max(0, m.start() - 3):m.start()]
        # dentro de uma chamada L(...): tem tradução ao lado (procura um 'L(' aberto e não fechado antes deste literal, na mesma instrução)
        ctx = s[max(0, m.start() - 600):m.start()]
        k = ctx.rfind('L('); depth = 0
        if k >= 0:
            for ch in ctx[k + 2:]: depth += (ch == '(') - (ch == ')')
            if depth >= 0 and ';' not in ctx[k:]: continue
        # 2.º argumento de L() (o inglês) não interessa; nem comentários
        line_start = s.rfind('\n', 0, m.start()) + 1
        if s[line_start:m.start()].lstrip().startswith('//'): continue
        if re.search(r"\*\s", s[line_start:m.start()]) and s[line_start:m.start()].lstrip().startswith('*'): continue
        line = s.count('\n', 0, m.start()) + 1
        rel = os.path.relpath(f, ROOT)
        if rel == 'ui/state.js' and 'first' in s[max(0, m.start() - 200):m.start()]: continue   # nomes do plantel de exemplo
        if body.startswith('https://'): continue
        if '${' in body: templates.append((rel, line, body))
        else: static.append((rel, line, body))
# Cada frase é partida nos pedaços de texto que de facto aparecem no ecrã (entre etiquetas HTML e
# expressões ${...}); cada pedaço tem de ser traduzido pelo T() real da app em inglês.
def segments(body):
    b = re.sub(r"\$\{(?:[^{}]|\{[^{}]*\})*\}", "\u0000#\u0000", body)      # expressões → marcador numérico/neutro
    out = []
    for part in re.split(r"<[^>]*>", b):
        for seg in re.split(r"\u0000#\u0000", part):
            seg = re.sub(r"\s+", " ", seg).strip()
            if seg and PT.search(seg) and not re.fullmatch(r"[\W\d#]*", seg): out.append(seg)
    return out
items = [(f, l, seg) for f, l, x in static + templates for seg in segments(x)]
uniq = sorted({x for _, _, x in items})
with sync_playwright() as p:
    b = p.chromium.launch(); pg = b.new_page(); pg.goto(U.APP); pg.wait_for_timeout(150)
    pg.evaluate("() => { ST.lang = LANG = 'en'; }")
    tr = dict(zip(uniq, pg.evaluate("ls => ls.map(x => T(x))", uniq)))
    b.close()
bad = sorted({(f, x) for f, l, x in items if tr.get(x) == x})
for f, x in bad: print(f'  SEM TRADUÇÃO  {f}  «{x[:110]}»')
print(f'auditoria de idioma: {len(bad)} pedaços de texto em português sem tradução')
sys.exit(1 if bad else 0)
