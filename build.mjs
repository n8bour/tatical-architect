// Junta src/ num único app/index.html autónomo (sem dependências externas nem rede)
import { readFileSync, writeFileSync } from 'node:fs';
const r = (f) => readFileSync(new URL('./src/' + f, import.meta.url), 'utf8');
export const ORDER = [
  'core/roles.js', 'core/tactics_data.js', 'core/engine.js', 'core/insights.js', 'core/sim_model.js',
  'ui/state.js', 'ui/components.js', 'ui/sim_view.js',
  'ui/views_home.js', 'ui/views_team.js', 'ui/views_tactic.js', 'ui/views_training.js', 'ui/views_bible.js', 'ui/views_settings.js',
  'ui/actions.js', 'ui/main.js',
];
const js = ORDER.map(f => `// ---- ${f}\n` + r(f)).join('\n;\n');
const html = `<!doctype html>
<html lang="pt">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<title>FM27 Tactical Bible</title>
<link rel="icon" href="data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><text y='.9em' font-size='90'>⚽</text></svg>">
<style>
${r('styles.css')}
</style>
</head>
<body>
<div class="app"><aside id="nav"></aside><main id="main"></main></div>
<div id="dropov">Largue o ficheiro — plantel (CSV/HTML/JSON), tática .fmf ou cópia .json</div>
<div class="toast" id="toast" role="status" aria-live="polite"></div>
<script>
${js}
</script>
</body>
</html>
`;
writeFileSync(new URL('./app/index.html', import.meta.url), html);
console.log('app/index.html', (html.length / 1024).toFixed(0) + ' KB');
