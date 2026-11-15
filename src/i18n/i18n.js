// ============================================================
// Idiomas: inglês por defeito, português como alternativa.
//  • L(pt, en): texto dinâmico escrito nas duas línguas na origem (motor, mensagens com nomes)
//  • T(texto): dicionário PT→EN para texto fixo (interface, Bíblia); números são substituídos
//    por '#' na chave e repostos na tradução (ex.: 'Plantel: # jogadores')
//  • translateDOM(raiz): aplica T a todos os textos e atributos visíveis depois de desenhar
// ============================================================
let LANG = 'en';
const L = (pt, en) => (LANG === 'en' ? en : pt);
const EN_DICT = new Map();
const EN_NUM = /[+−-]?\d+(?:[.,]\d+)?/g;
function addEN(obj) { for (const [k, v] of Object.entries(obj)) EN_DICT.set(k, v); }
function T(s) {
  if (LANG !== 'en' || s == null) return s;
  const str = String(s), m = str.match(/^(\s*)([\s\S]*?)(\s*)$/), core = m[2];
  if (!core) return str;
  let t = EN_DICT.get(core);
  if (t === undefined) {   // prefixo simbólico: '💡 texto', '→ texto', '— texto', '· texto'…
    const pm = core.match(/^([💡→✓■▶—·•+!▲❚]+\s*)([\s\S]+)$/u);
    if (pm) { const rest = T(pm[2]); if (rest !== pm[2]) return m[1] + pm[1] + rest + m[3]; }
  }
  if (t === undefined && /\d/.test(core)) {
    const nums = []; const key = core.replace(EN_NUM, x => { nums.push(x); return '#'; });
    const tpl = EN_DICT.get(key);
    if (tpl !== undefined) { let i = 0; t = tpl.replace(/#/g, () => (i < nums.length ? nums[i++] : '#')); }
  }
  return t === undefined ? str : m[1] + t + m[3];
}
const TR_ATTRS = ['title', 'placeholder', 'aria-label'];
function translateDOM(root) {
  if (LANG !== 'en' || !root) return;
  const w = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, { acceptNode: n => (n.parentNode && /^(SCRIPT|STYLE|TEXTAREA)$/.test(n.parentNode.nodeName)) ? NodeFilter.FILTER_REJECT : NodeFilter.FILTER_ACCEPT });
  const nodes = []; while (w.nextNode()) nodes.push(w.currentNode);
  for (const n of nodes) { const v = n.nodeValue; if (v && /\S/.test(v)) { const t = T(v); if (t !== v) n.nodeValue = t; } }
  root.querySelectorAll('[title],[placeholder],[aria-label]').forEach(el => TR_ATTRS.forEach(a => { const v = el.getAttribute(a); if (v) { const t = T(v); if (t !== v) el.setAttribute(a, t); } }));
}
const LOCALE = () => (LANG === 'en' ? 'en-GB' : 'pt-PT');
