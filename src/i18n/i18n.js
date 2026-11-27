// ============================================================
// Idiomas: inglês por defeito, português como alternativa.
//  • L(pt, en): texto dinâmico escrito nas duas línguas na origem (motor, mensagens com nomes)
//  • T(texto): dicionário PT→EN para texto fixo da interface; números são substituídos
//    por '#' na chave e repostos na tradução (ex.: 'Plantel: # jogadores')
//  • translateDOM(raiz): aplica T a todos os textos e atributos visíveis depois de desenhar
// ============================================================
let LANG = 'en';
const L = (pt, en) => (LANG === 'en' ? en : pt);
const EN_DICT = new Map();
const EN_NUM = /[+−-]?\d+(?:[.,]\d+)?/g;
function addEN(obj) { for (const [k, v] of Object.entries(obj)) EN_DICT.set(k, v); }
// Em português: os termos do FM (papéis, instruções, opções…) passam aos nomes portugueses, também dentro de frases.
let PT_RE = null;
function ptTerms(s) {
  if (typeof PT_TERMS === 'undefined') return s;
  const tr = s.trim(); if (PT_EXACT_ONLY[tr]) return s.replace(tr, PT_EXACT_ONLY[tr]);
  if (!PT_RE) {
    const ks = Object.keys(PT_TERMS).sort((a, b) => b.length - a.length).map(k => k.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'));
    PT_RE = new RegExp('(?<![A-Za-z\\-])(' + ks.join('|') + ')(?![A-Za-z])', 'g');
  }
  return s.replace(PT_RE, m => PT_TERMS[m]).replace(/\(default\)/g, '(padrão)');
}
function T(s) {
  if (s == null) return s;
  if (LANG === 'pt') return ptTerms(String(s));
  if (LANG !== 'en') return s;
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
  if (t === undefined) { for (const [re, rep] of EN_RX) { if (re.test(core)) { t = core.replace(re, rep); break; } } }
  return t === undefined ? str : m[1] + t + m[3];
}
// mensagens com partes variáveis que não vêm de L() (erros devolvidos pela parte nativa da app)
const EN_RX = [
  [/^Não consegui correr o fmsave\. ([\s\S]*)$/, 'Could not run fmsave. $1'],
  [/^Não consegui copiar o save: ([\s\S]*)$/, 'Could not copy the save: $1'],
  [/^Save não encontrado: ([\s\S]*)$/, 'Save not found: $1'],
  [/^Não encontrei a pasta do FM27$/, 'FM27 folder not found'], [/^O BepInEx não está instalado no FM27$/, 'BepInEx is not installed in FM27'],
  [/^Feche o FM27 e tente outra vez \(o plugin está em uso\)$/, 'Close FM27 and try again (the plugin is in use)'],
  [/^Caminho inválido$/, 'Invalid path'], [/^Método inválido$/, 'Invalid method'], [/^Link inválido$/, 'Invalid link'],
];
const TR_ATTRS = ['title', 'placeholder', 'aria-label'];
function translateDOM(root) {
  if (!root || (LANG !== 'en' && LANG !== 'pt')) return;
  // em português, os blocos marcados data-gameterms="en" (Ficha para o FM27) mantêm os nomes como no jogo em inglês
  const keep = (n) => LANG === 'pt' && n.parentElement && n.parentElement.closest('[data-gameterms="en"]');
  const w = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, { acceptNode: n => ((n.parentNode && /^(SCRIPT|STYLE|TEXTAREA)$/.test(n.parentNode.nodeName)) || keep(n)) ? NodeFilter.FILTER_REJECT : NodeFilter.FILTER_ACCEPT });
  const nodes = []; while (w.nextNode()) nodes.push(w.currentNode);
  for (const n of nodes) { const v = n.nodeValue; if (v && /\S/.test(v)) { const t = T(v); if (t !== v) n.nodeValue = t; } }
  root.querySelectorAll('[title],[placeholder],[aria-label]').forEach(el => !(LANG === 'pt' && el.closest('[data-gameterms="en"]')) && TR_ATTRS.forEach(a => { const v = el.getAttribute(a); if (v) { const t = T(v); if (t !== v) el.setAttribute(a, t); } }));
}
const LOCALE = () => (LANG === 'en' ? 'en-GB' : 'pt-PT');
