# Auditoria de idioma nos DOIS sentidos, com vocabulário tirado do próprio código:
#  • palavras só portuguesas = lado PT de cada L(pt, en) e do dicionário, menos as que também aparecem em inglês
#  • palavras só inglesas   = o inverso (sem termos do FM que ficam em inglês, nem nomes de produtos)
# Mostra todos os ecrãs, em vários estados, em EN e em PT, e falha se encontrar texto da outra língua.
# Uso: python3 tests/ui_lang_audit.py
import os, re, sys, json, glob, subprocess
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ui_layout as U
from playwright.sync_api import sync_playwright
SRC = os.path.join(U.HERE, '..', 'src')

def literal(s, i):
    q = s[i]; j = i + 1; depth = 0
    while j < len(s):
        c = s[j]
        if c == '\\': j += 2; continue
        if q == '`' and c == '$' and s[j + 1:j + 2] == '{': depth += 1; j += 2; continue
        if q == '`' and c == '}' and depth: depth -= 1; j += 1; continue
        if c == q and not depth: return s[i + 1:j], j + 1
        j += 1
    return None, len(s)
def pairs():
    out = []
    for f in glob.glob(os.path.join(SRC, '**', '*.js'), recursive=True):
        s = open(f, encoding='utf8').read()
        for m in re.finditer(r"(?<![\w.])L\(\s*", s):
            i = m.end()
            if i >= len(s) or s[i] not in "'`\"": continue
            a, j = literal(s, i); k = j
            while k < len(s) and s[k] in ' \n\t': k += 1
            if k >= len(s) or s[k] != ',': continue
            k += 1
            while k < len(s) and s[k] in ' \n\t': k += 1
            if k >= len(s) or s[k] not in "'`\"": continue
            b, _ = literal(s, k)
            if a is not None and b is not None: out.append((a, b))
    d = open(os.path.join(SRC, 'i18n', 'en_dict.js'), encoding='utf8').read()
    for m in re.finditer(r'^\s*("(?:[^"\\]|\\.)*"):\s*("(?:[^"\\]|\\.)*"),\s*$', d, re.M): out.append((json.loads(m.group(1)), json.loads(m.group(2))))
    return out
WORD = re.compile(r"[A-Za-zÀ-ÿ’']+")
def words(t): t = re.sub(r"\$\{[^}]*\}|<[^>]+>", " ", t); return {w.lower().strip("’'") for w in WORD.findall(t) if len(w.strip("’'")) >= 3}
P = pairs()
pt_w, en_w = set(), set()
for a, b in P: pt_w |= words(a); en_w |= words(b)
terms = open(os.path.join(SRC, 'i18n', 'pt_terms.js'), encoding='utf8').read()
fm_en = set(); fm_pt = set()
for m in re.finditer(r"'([^']+)':'([^']+)'", terms): fm_en |= words(m.group(1)); fm_pt |= words(m.group(2))
ALLOW = set('fmsave steam tactics lab fm27 fm26 view exporter pypi github windows csv json html xml tsv zstd aes gcm nonce fmf uvx winget astral clean slate box ok pwb iwb'.split()) | \
        set('wide flat diamond narrow dm wb deep'.split()) | set('player club country test tactic db'.split())
PT_ONLY = {w for w in pt_w - en_w - fm_en - ALLOW if not w.isdigit()}
EN_ONLY = {w for w in en_w - pt_w - fm_pt - fm_en - ALLOW if not w.isdigit()}
PT_ACC = re.compile(r"[ãõçâêôàÃÕÇÂÊÔÀ]")

GRAB = r"""(lang) => { const out = []; const skip = (n) => { const e = n.parentElement; return !e || /^(SCRIPT|STYLE|TEXTAREA|CODE)$/.test(e.tagName) || e.closest('[data-raw]') || (lang === 'pt' && e.closest('[data-gameterms="en"]')); };
  const w = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT); while (w.nextNode()) { const n = w.currentNode; if (skip(n)) continue; const v = n.nodeValue.trim(); if (v) out.push(v); }
  document.querySelectorAll('[title],[placeholder],[aria-label]').forEach(e => { if (lang === 'pt' && e.closest('[data-gameterms="en"]')) return; ['title','placeholder','aria-label'].forEach(a => { const v = e.getAttribute(a); if (v && v.trim()) out.push(v.trim()); }); });
  return out; }"""
DB = subprocess.run(['node', os.path.join(U.HERE, 'gen_scout_db.cjs'), '1500'], capture_output=True, text=True).stdout
SQ = subprocess.run(['node', '-e', "const {gen}=require(process.argv[1]); const L=gen(30,7).split('\\n'); const h=L[0].split(','), c=h.indexOf('club_name'); console.log([L[0],...L.slice(1).map(l=>{const x=l.split(',');x[c]='My Club';return x.join(',');})].join('\\n'))", os.path.join(U.HERE, 'gen_scout_db.cjs')], capture_output=True, text=True).stdout
NAMES = re.compile(r"^(Player \d+|Club \d+( \"B\")?|Country \d+|My Club|Tactical Bible)$")

def collect(lang):
    texts = set(); errs = []
    with sync_playwright() as p:
        b = p.chromium.launch()
        for desktop in (False, True):
            pg = b.new_page(viewport={'width': 1500, 'height': 950}); pg.on('pageerror', lambda e: errs.append(str(e)))
            if desktop:
                pg.add_init_script("""window.__TAURI__={core:{invoke:async(c,a)=>{ if(c==='export_dirs')return [['C:/x',true]]; if(c==='latest_save')return {path:'C:/x/T.fm',name:'T.fm',modified:5,size:1e8};
                  if(c==='export_save') throw 'Não consegui correr o fmsave. uvx: UnsupportedGameError';
                  return null; }}};""")
            pg.goto(U.APP); pg.wait_for_timeout(150)
            g = lambda: [texts.add(x) for x in pg.evaluate(GRAB, lang)]
            pg.evaluate(f"() => {{ ST.lang = LANG = '{lang}'; render(); }}"); g()
            for v, t in U.VIEWS: pg.evaluate(f"A.go('{v}'{', ' + repr(t) if t else ''})"); pg.wait_for_timeout(40); g()
            pg.evaluate("t => { ingestSquadText(t, 'Career.fm (save)'); }", SQ); pg.evaluate("t => ingestSquadText(t, 'rival.csv', { force: 'opp' })", U.RIVAL)
            pg.evaluate("() => { ST.myClub = 'My Club'; autoPickXI(ST.tactic, players()); genOpts(); A.genRun(); }"); pg.wait_for_timeout(900)
            pg.evaluate("t => A.scoutIngest(t, { save: 'db.csv' })", DB); pg.wait_for_function("SCOUT.db", timeout=20000)
            if desktop: pg.evaluate("() => { ST.auto.save = true; ST.auto.saveErr = 'Não consegui correr o fmsave. uvx: UnsupportedGameError'; ST.lastSave = { name: 'T.fm', size: 1e8 }; ST.f9Found = true; render(); }")
            for v, t in U.VIEWS:
                pg.evaluate(f"A.go('{v}'{', ' + repr(t) if t else ''})"); pg.wait_for_timeout(50); g()
                if (v, t) == ('team', 'squad'):
                    for m in ('roles', 'ability', 'pers'): pg.evaluate(f"A.sqMode('{m}')"); g()
                    pg.evaluate("A.squadOpen(players()[0].id)"); g(); pg.evaluate("A.squadClose()")
                if (v, t) == ('tactic', 'analyze'): pg.evaluate("A.sel(3)"); g(); pg.evaluate("A.sel(3)")
                if (v, t) == ('tactic', 'plans'):
                    for k in ('chase', 'protect', 'away_strong', 'away_equal', 'home_weak'): pg.evaluate(f"A.planK('{k}')"); g()
                if (v, t) == ('tactic', 'sim'):
                    for i in range(6): pg.evaluate(f"A.simPhase({i})"); g()
                if v == 'scout':
                    for k in ('interested', 'cheap_wonder', 'reach', 'expiring', 'free', 'bargain', 'upgrade', 'veteran', 'hidden', 'elite_pers', 'weak_spot'):
                        pg.evaluate(f"() => {{ A.scoutPreset('{k}'); const r = scoutResults(); if (r[0]) A.scoutOpen(r[0].id); }}"); g(); pg.evaluate("A.scoutClose()")
                    pg.evaluate("A.scoutTab('list')"); g(); pg.evaluate("A.scoutTab('results')")
            pg.evaluate("() => { ST.openClips = {}; ROLES.slice(0, 20).forEach(r => ST.openClips['role:' + r.id] = 1); SIM_TI.forEach(k => ST.openClips['ti:' + k] = 1); }")
            for t in ('roles', 'ti', 'concepts', 'btraining'): pg.evaluate(f"A.go('bible','{t}')"); pg.wait_for_timeout(120); g()
            pg.evaluate("() => { busyStart('x', L('A ler o save ', 'Reading save ') + 'T.fm', { p: .4, detail: L('lê uma cópia, o save não é alterado', 'reads a copy, the save is not changed') }); }"); g(); pg.evaluate("busyEnd('x')")
            pg.close()
        b.close()
    return texts, errs

def bad_en(t):
    if NAMES.match(t): return None
    t = re.sub(r'Jogador \d+', '', t)   # nomes dos jogadores do adversário de teste
    if PT_ACC.search(t) and not re.search(r"Çifçi|Marković|Ferrán", t): return 'acento'
    w = words(t) & PT_ONLY
    return ', '.join(sorted(w)) if w else None
def bad_pt(t):
    if NAMES.match(t): return None
    w = words(t) & EN_ONLY
    return ', '.join(sorted(w)) if w else None

# auto-teste: os detectores têm de apanhar exemplos conhecidos (senão um “0” não quer dizer nada)
probe_en = ['Ler o save agora', 'Contrato a terminar', 'Plano aplicado à tática', 'Jogadores livres', '— alvo de recrutamento/treino.']
probe_pt = ['Read the save now', 'Contract expiring', 'Plan applied to the tactic', 'Free agents', 'Squad read just now']
miss = [x for x in probe_en if not bad_en(x)] + [x for x in probe_pt if not bad_pt(x)]
print('auto-teste dos detectores:', 'OK' if not miss else 'FALHOU em ' + repr(miss))
fails = len(miss)
for lang, test in (('en', bad_en), ('pt', bad_pt)):
    texts, errs = collect(lang)
    hits = sorted({(t, r) for t in texts for r in [test(t)] if r})
    print(f'[{lang}] {len(texts)} textos · {len(hits)} com palavras da outra língua' + (f' · {len(errs)} erros JS' for _ in [0]).__next__() if errs else f'[{lang}] {len(texts)} textos · {len(hits)} com palavras da outra língua')
    for t, r in hits[:400]: print(f'   «{t[:120]}»  ← {r}')
    for e in errs[:3]: print('   JS:', e)
    fails += len(hits) + len(errs)
print('auditoria de idioma nos dois sentidos:', 'OK' if not fails else 'COM PROBLEMAS')
sys.exit(1 if fails else 0)
