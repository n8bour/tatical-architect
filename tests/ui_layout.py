# Portão de layout da FM27 Tactical Bible.
# Uso: python3 tests/ui_layout.py   (precisa de: pip install playwright && python -m playwright install chromium)
# Percorre todos os ecrãs em 6 larguras e 3 modos (normal, menu recolhido, painéis fechados)
# com um plantel realista de 62 jogadores e falha (código 1) se encontrar:
#   • elementos irmãos (grelha ou linha flex) a sobreporem-se;
#   • um elemento a sair do seu contentor (que não faça scroll);
#   • scroll horizontal na área principal;
#   • erros de JavaScript.
import os, sys, pathlib
from playwright.sync_api import sync_playwright

HERE = os.path.dirname(os.path.abspath(__file__))
APP = pathlib.Path(HERE, '..', 'app', 'index.html').resolve().as_uri()
import subprocess
SCOUT_DB = subprocess.run(['node', os.path.join(HERE, 'gen_scout_db.cjs'), '1500'], capture_output=True, text=True).stdout
RIVAL = open(os.path.join(HERE, 'fixtures', 'squad_de.csv'), encoding='utf8').read().replace(';Clube;', ';Rival FC;')
LANG = os.environ.get('LAYOUT_LANG', 'en')               # LAYOUT_LANG=pt para verificar em português
WIDTHS = [1024, 1100, 1280, 1366, 1920, 2560]          # 1024/1100 ≈ 1366 ou 1920 com escala 125–150% no Windows
MODES = ['normal', 'menu recolhido', 'painéis fechados']
if os.environ.get('LAYOUT_QUICK'): WIDTHS, MODES = [1024, 1366, 1920], ['normal']   # passagem rápida (ex.: segunda língua no CI)
VIEWS = [('home', None), ('team', 'squad'), ('team', 'gen'), ('team', 'xi'), ('team', 'opp'),
         ('scout', None), ('tactic', 'analyze'), ('tactic', 'sim'), ('tactic', 'plans'), ('tactic', 'setpieces'), ('tactic', 'sheet'), ('tactic', 'import'),
         ('training', None), ('bible', 'roles'), ('bible', 'ti'), ('bible', 'concepts'), ('bible', 'btraining'), ('settings', None)]

CHECK = r"""() => {
  const out = [], R = e => e.getBoundingClientRect(), main = document.querySelector('main');
  const visible = e => { const s = getComputedStyle(e); return s.display !== 'none' && s.visibility !== 'hidden' && R(e).width > 0 && R(e).height > 0; };
  const label = e => e.tagName + (typeof e.className === 'string' && e.className ? '.' + e.className.split(' ')[0] : '');
  // 1. irmãos de grelha / linha flex sobrepostos
  main.querySelectorAll('*').forEach(g => {
    if (g.closest('svg')) return;
    const cs = getComputedStyle(g); if (!(cs.display === 'grid' || cs.display === 'flex' || cs.display === 'inline-flex')) return;
    const k = [...g.children].filter(c => visible(c) && getComputedStyle(c).position !== 'absolute' && getComputedStyle(c).position !== 'fixed');
    for (let i = 0; i < k.length; i++) for (let j = i + 1; j < k.length; j++) {
      const a = R(k[i]), b = R(k[j]);
      const ox = Math.min(a.right, b.right) - Math.max(a.left, b.left), oy = Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top);
      if (ox > 2 && oy > 2) out.push(`sobreposição em ${label(g)}: ${label(k[i])} × ${label(k[j])} (${Math.round(ox)}×${Math.round(oy)}px)`);
    }
  });
  // 2. filho que sai do pai (pai sem scroll)
  main.querySelectorAll('*').forEach(e => {
    const p = e.parentElement; if (!p || p === main || !visible(e)) return;
    if (e.closest('svg') && e.tagName.toLowerCase() !== 'svg') return;
    if (/(auto|scroll|hidden|clip)/.test(getComputedStyle(p).overflowX)) return;
    const sc = e.closest('.table-wrap,.chips,table'); if (sc && sc !== e) return;
    const r = R(e), q = R(p);
    if (r.right > q.right + 2 || r.left < q.left - 2) out.push(`sai do contentor: ${label(e)} em ${label(p)} (+${Math.round(Math.max(r.right - q.right, q.left - r.left))}px)`);
  });
  // 3. botões espremidos (texto partido em várias linhas dentro do botão)
  main.querySelectorAll('.btn').forEach(b => { if (visible(b) && R(b).height > 64) out.push('botão espremido: ' + (b.textContent || '').trim().slice(0, 30) + ' (' + Math.round(R(b).height) + 'px de altura)'); });
  // 4. scroll horizontal da área principal
  if (main.scrollWidth - main.clientWidth > 1) out.push('scroll horizontal na área principal: ' + (main.scrollWidth - main.clientWidth) + 'px');
  return [...new Set(out)].slice(0, 5);
}"""

SETUP = """() => {
  const base = makeSample();
  const names = ['Abubacarr Sedi Kinteh','Alessio Hoste','Altay Çifçi','Angel Massart','Anthony Munganga','Antonio David','Axel Goemaere','Boris Marković','Bram De Bruyn','Bryan Baeten','Carlos Forbs','Cyrillus Winters','Danny Devos','Edwin Ochieng','Ferrán Quintana Rodríguez','Gertjan Gijbels','Gomolemo Kekana','Hasan Stiers','Hugo Mertens','Ives Vermeulen','Joël Kabangu','Jules Mavinga','Lilian Oueslati','Lorenzo Fettal','Cavan Sullivan'];
  const ps = [];
  for (let i = 0; i < 62; i++) {
    const p = JSON.parse(JSON.stringify(base[i % base.length])); p.id = 'x' + i;
    p.name = names[i % names.length] + (i >= names.length ? ' ' + i : '');
    p.posRaw = ['D (RLC), WB (RL), DM', 'M (RLC), AM (RLC), ST (C)', 'GK', 'DM, M (C), AM (C)'][i % 4]; p.pos = parsePositions(p.posRaw); p.age = 15 + (i % 17); ps.push(p);
  }
  ST.squad = { ...EMPTY_SQUAD(), players: ps, source: 'Beerschot - Época 2027-28 (save muito comprido).fm' };
  ST.myClub = 'K Beerschot Voetbalclub Antwerpen'; autoPickXI(ST.tactic, ps);
  ST.tactic.name = 'Uma tática com um nome bastante comprido para testar os cabeçalhos da aplicação';
}"""

def main():
    problems, errors = {}, []
    with sync_playwright() as p:
        b = p.chromium.launch()
        for w in WIDTHS:
            for mode in MODES:
                pg = b.new_page(viewport={'width': w, 'height': 900})
                pg.on('pageerror', lambda e: errors.append(str(e)))
                pg.goto(APP); pg.wait_for_timeout(150)
                pg.evaluate(f"() => {{ ST.lang = LANG = '{LANG}'; }}")
                pg.evaluate(SETUP)
                pg.evaluate("t => ingestSquadText(t, 'rival.csv', { force: 'opp' })", RIVAL)
                pg.evaluate("() => { genOpts(); A.genRun(); }"); pg.wait_for_timeout(900)
                pg.evaluate("t => A.scoutIngest(t, { save: 'db.csv' })", SCOUT_DB); pg.wait_for_function("SCOUT.db", timeout=20000)
                if mode == 'menu recolhido': pg.evaluate("() => { ST.ui.navMini = true; render(); }")
                for v, t in VIEWS:
                    pg.evaluate(f"A.go('{v}'{', ' + repr(t) if t else ''})"); pg.wait_for_timeout(100)
                    if mode == 'painéis fechados': pg.evaluate("A.collapseAll(true)"); pg.wait_for_timeout(60)
                    r = pg.evaluate(CHECK)
                    if r: problems[(w, mode, v, t)] = r
                    if v == 'scout':          # também com a ficha do jogador aberta
                        pg.evaluate("() => { A.scoutPreset('upgrade'); A.scoutOpen(scoutResults()[0].id); }"); pg.wait_for_timeout(80)
                        r = pg.evaluate(CHECK.replace("main = document.querySelector('main')", "main = document.querySelector('.modal')"))
                        if r: problems[(w, mode, 'scout-card', None)] = r
                        pg.evaluate("A.scoutClose()")
                        pg.evaluate("A.focusOpen()"); pg.wait_for_timeout(80)          # janela 'Observação no FM'
                        r = pg.evaluate(CHECK.replace("main = document.querySelector('main')", "main = document.querySelector('.modal')"))
                        if r: problems[(w, mode, 'focus-dialog', None)] = r
                        pg.evaluate("A.focusClose()")
                # barra de topo (menu, estado e dados do jogo), nesta largura e neste modo
                r = pg.evaluate(CHECK.replace("main = document.querySelector('main')", "main = document.querySelector('#nav')"))
                if r: problems[(w, mode, 'topbar', None)] = r
                if mode == 'painéis fechados': pg.evaluate("() => { ST.ui.collapsed = {}; save(); }")
                pg.close()
        b.close()
    for k, v in problems.items(): print(k, *v, sep='\n   ')
    print(f'[{LANG}] {len(WIDTHS)} larguras × {len(MODES)} modos × {len(VIEWS)} ecrãs = {len(WIDTHS) * len(MODES) * len(VIEWS)} verificações')
    print('ecrãs com problemas:', len(problems), '| erros JS:', len(errors))
    for e in errors[:5]: print('  JS:', e)
    return 1 if problems or errors else 0

if __name__ == '__main__':
    sys.exit(main())
