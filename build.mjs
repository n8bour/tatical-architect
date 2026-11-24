// Junta src/ num único app/index.html autónomo (sem dependências externas nem rede)
import { readFileSync, writeFileSync } from 'node:fs';
const r = (f) => readFileSync(new URL('./src/' + f, import.meta.url), 'utf8');
export const ORDER = [
  'i18n/i18n.js', 'core/roles.js', 'core/tactics_data.js', 'core/engine.js', 'core/insights.js', 'core/synergy.js', 'core/generator.js', 'core/gameplans.js', 'core/scouting.js', 'core/sim_model.js',
  'i18n/en_dict.js', 'i18n/pt_terms.js', 'ui/state.js', 'ui/components.js', 'ui/sim_view.js', 'ui/clips.js',
  'ui/views_home.js', 'ui/views_team.js', 'ui/views_gen.js', 'ui/views_synergy.js', 'ui/views_gameplans.js', 'ui/views_scout.js', 'ui/views_squadtable.js', 'ui/views_tactic.js', 'ui/views_setpieces.js', 'ui/views_training.js', 'ui/views_bible.js', 'ui/views_settings.js', 'ui/fmbridge.js',
  'ui/actions.js', 'ui/main.js',
];
const js = ORDER.map(f => `// ---- ${f}\n` + r(f)).join('\n;\n');
const html = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<title>FM27 Tactical Bible</title>
<link rel="icon" href="data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><rect width='100' height='100' rx='22' fill='%23b3131d'/><rect x='15' y='13' width='70' height='74' rx='3' fill='none' stroke='%23f3ead8' stroke-width='2.2'/><g fill='%23f6eedd'><circle cx='50' cy='81' r='4.6'/><circle cx='26' cy='68' r='4.6'/><circle cx='43' cy='70' r='4.6'/><circle cx='57' cy='70' r='4.6'/><circle cx='74' cy='68' r='4.6'/><circle cx='32' cy='52' r='4.6'/><circle cx='50' cy='55' r='4.6'/><circle cx='68' cy='52' r='4.6'/><circle cx='28' cy='32' r='4.6'/><circle cx='50' cy='28' r='4.6'/><circle cx='72' cy='32' r='4.6'/></g></svg>">
<script type="module" src="https://ui-sdk.dev/web.js"></script>
<link rel="stylesheet" href="https://ui-sdk.dev/web.css" />
<style>
${r('styles.css')}
</style>
</head>
<body>
<div class="app"><header id="nav"></header><main id="main"></main></div>
<div id="dropov">Largue o ficheiro — plantel (CSV/HTML/JSON), tática .fmf ou cópia .json</div>
<div id="topbar" aria-hidden="true"><i></i></div>
<div class="toast" id="toast" role="status" aria-live="polite"></div>
<script>
${js}
</script>
</body>
</html>
`;
writeFileSync(new URL('./app/index.html', import.meta.url), html);
console.log('app/index.html', (html.length / 1024).toFixed(0) + ' KB');
