# FM27 Tactical Bible

Tactical analyser, animated pitch and a bible of roles and instructions for Football Manager 27. Works offline and needs no account.

## How to use (3 steps)

1. **Squad.** There are two ways, both optional and both relying on a free external tool. The cards in **Squad** and **Settings › Integrations** say what each one **requires**, have the download link and show the status on your PC.
   Automatic save reading and automatic F9 loading are **off** by default. Turn them on in the cards, or use "Read the save now" and "Load the latest F9".

   **Squad, read from the save:**
   The app always runs the **latest fmsave** (`uvx fmsave@latest`). When FM27 gets a patch, as soon as an fmsave version that supports it is released, the app uses it automatically. Without internet, it uses the version already cached. If you installed fmsave with pip, update it with `python -m pip install --upgrade fmsave`.
   - **Installed app:** save the game in FM27 and the app reads the save by itself (it checks every 10 seconds). The first time, you need to install the reader:
     ```
     winget install --id=astral-sh.uv -e
     ```
     Restart the app after installing. On the first read, `uv` downloads [fmsave](https://github.com/rhiever/fmsave) and needs internet only that once.
   - **Browser:** open [fmsave online](https://rhiever.github.io/fmsave/app/), drop your `.fm` there (the save is read on your PC and never uploaded), download the squad CSV and drag it into the app.
   - **Alternative:** press F9 in the game with the [FM View Exporter](https://github.com/ElectroHugin/FM-View-Exporter) plugin. In the installed app this is also automatic.
2. **Tactic.** In **My team › Generate tactic** the app builds a complete tactic for your squad. You can also create your own in **Tactic › Analyse**. In **Tactic › Analyse**, click a player to change roles and instructions. **Animated pitch** shows the tactic in motion.
3. **Opponent.** Open the next opponent's squad in FM and press **F9**. The app detects that it's another club and generates the report in **My team › Opponent**. The game plan is applied to the tactic with one click.

To take the tactic into the game, use **Tactic › FM27 sheet**. The sheet follows the order of the tactics screen and has checkboxes to tick as you go.

## What's inside

| Section | What it does |
|---|---|
| Home | Status of the 3 steps, what to fix first, the system that best suits the squad and the opponent summary |
| My team › Squad | Save reading (fmsave) or import (CSV/HTML/JSON from FM View Exporter, FMPT Chief Analyst or FMLibreEditor) and each player's best roles |
| My team › **Generate tactic** | A complete tactic built around the squad. Tests every pair of in-possession and out-of-possession formations, picks the IP/OOP role for each position using balance rules, and the exact XI (Hungarian algorithm). Generates the 27 team instructions, each with its reason, by comparing each unit with the squad average, plus player instructions where the attributes call for them. Includes set pieces, captains, depth per position, where to strengthen, 3 alternatives and training. Takes into account the league objective, mandatory starters, a star player, up to 3 youngsters to develop and the weak foot on the flanks. Buttons to use the tactic, view it on the pitch or copy it as text |
| **Role synergy — all phases** (Generator and Analyser) | Measures whether roles serve and cover each other, not just whether each player is in his best role, using real attributes. **Build-up:** who plays out → who receives, support between the lines, press resistance, long ball → target. **Attack:** creator → finisher, crosses → targets, front pairing, box presence, flanks, creative load. **Defence:** pressers have cover, centre-back pairing (one steps out, the other covers), protection in front of the centre-backs, flank defence (a winger who stays high forces the full-back to hold), line vs. profiles, aerial game. **Transitions:** rest-defence on losing the ball, who launches → who runs in the counter, number of outlets. Two maps (in and out of possession) and problems per phase, each with its fix. The Generator weights 55% suitability, 30% synergy and 15% balance, and in tests it improves every phase without sacrificing defence |
| **Scouting** | Every player in the save, read by fmsave (`export players --all`) from a copy, cached until the save changes. Includes attributes, CA/PA, personality, hidden attributes, contract, wage and value. **11 preset searches:** interested in joining (estimate), cheap wonderkids, wonderkids who will reach their potential, expiring contracts, free agents, bargains, signings for my tactic, free veterans, hidden gems, elite personality and my weakest position. **Filters:** name, age, CA/PA, position (competent or natural), foot, height, club, league country, nationality, value, wage, contract end, club reputation, free agents, loans, personality, hidden attributes, chance of reaching potential, chance of wanting to join your club (estimate), minimum attributes (several) and role with minimum suitability. Sortable, paginated table, player profile (attributes, best roles and where he would fit in your XI), shortlist and CSV export. In the browser, it accepts the all-players CSV from fmsave online. The **chance of wanting to join** is an estimate (fmsave doesn't read the game's "interested" flag). It uses your club's reputation against his club's, his reputation, his contract (free, expiring, squad status), ambition and loyalty, and shows the reasons in the profile |
| My team › **Squad** | The player table has three views: **Roles** (best roles in and out of possession), **Ability and contract** (CA, PA, chance of reaching PA, value, wage, contract end, status) and **Personality and hidden** (professionalism, ambition, pressure, loyalty, temperament, adaptability, consistency, important matches, injury proneness, versatility). Every column is sortable and each player opens the full profile. This data comes from the save read by fmsave; F9 and other CSVs only bring what the view exports |
| My team › Best XI | Suitability per position, better alternative role, **player instructions recommended by attributes**, backup and successor (≤ 21 years old), and 9 systems ranked for the squad |
| My team › Opponent | 9 units compared with your XI, the 3 most dangerous players with opposition instructions and an applicable adjustment plan |
| Tactic › Analyse | In- and out-of-possession formation, roles, instructions and diagnosis (rest-defence, width, box, pressing, transitions, FM27 complexity, suitability) |
| Tactic › Animated pitch | 6 phases of play with each role's movement, plus the effect of the instructions in real time |
| Tactic › **Match situations** | Plan B/C from the current tactic and squad, in five scenarios: **We need goals**, **Hold the result**, **Away against stronger sides**, **Away against an equal side** and **At home against weaker sides** (low block). Each plan says what changes and why, using the players' attributes: mentality, team instructions, roles, player instructions (added and removed), Tactical Tweaks, substitutions from the bench and opposition instructions for their three most dangerous players. Role changes are tested against synergy: in an attacking plan they're only made if they don't weaken the attack, and in a defensive plan only if they don't weaken the defence. The impact shows attacking commitment and defensive security (intensity) and synergy per phase (coherence). You can save the plan as a tactic, use it right away or copy it as text |
| Tactic › Set pieces | Takers (corners with the right foot, direct and crossed free kicks, penalties, long throws), captain and vice-captain, attacking and defending corners in a diagram (zones, marking the opponent's strongest in the air, who stays back and who stays up) and applicable tips |
| Tactic › FM27 sheet | Step-by-step guide to recreate the tactic in the game, including the *Set Pieces & Captains* section |
| Training | Tactical Understanding priorities, typical week, Role Training and focus per player |
| **Apply in FM** (optional, Windows) | With the Tactical Bible Bridge plugin inside FM27 (installed from *Settings › Integrations*; needs BepInEx 6): **Place XI in FM** (Tactic › Analyse), **Apply in FM** in Set pieces (corner and free-kick takers per foot, corner delivery, and the attacking and defending corner routines slot by slot), **Shortlist → FM** and **Scout in FM** (creates a Recruitment Focus with ages, minimum ability/potential stars and the countries of your search results). See `bridge/README.md`. |
| Bible | 77 roles and instructions (including the new FM27 ones), concepts and Tactical Effectiveness. Each role has a **clip** showing that player's individual movement. Each animated instruction has a **dual clip** showing, side by side, the effect of the minimum and maximum option |

You can drag any file anywhere onto the window. Your club's squads go to Squad; another club's squads go to Opponent; `.fmf` files go to the inspector; `.json` backups are restored.

## Install

**Without installing anything.** Double-click `app/index.html`. It has everything except automatic F9 loading.

**Windows app (GitHub Actions, nothing to install on your PC):**
1. Create a repository and push these files **to the root**. Make sure you see `.github/`, `app/`, `src/`, `src-tauri/` and `build.mjs`.
2. The build runs on every push to `main` (or via **Actions › Build desktop app › Run workflow**). It reads the version from `src-tauri/tauri.conf.json` and, if release `v<version>` doesn't exist yet, builds and publishes it with the notes in `.github/RELEASE_NOTES.md`. If that version was already published, nothing is built.
3. Open **Releases** (right-hand column of the repository's main page) and download from *Assets* the `…setup.exe` (installer) or `FM27-Tactical-Bible-portatil.exe` (runs without installing).

**To publish a new version:** bump the version in `package.json`, `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml` (the build fails if they differ), update `.github/RELEASE_NOTES.md` and push to `main`.

Duration: about 6 to 8 minutes the first time and 2 to 3 minutes afterwards, thanks to the cache. By default it only builds for Windows. For macOS and Linux, tick the option in *Run workflow*.

**Locally** (requires Node, Rust and the Visual Studio C++ Build Tools): run `npm run app:build`.

## Language
The app opens in **English** by default. Portuguese is chosen with the **EN/PT** selector at the bottom of the menu, and the choice is remembered.
- **Full translation:** covers the interface, the Bible (descriptions and tips for the 77 roles, instructions, concepts, training) and the messages from the diagnosis, generator, opponent report, and set pieces.
- **Generator:** when switching language, the report's reasons are recalculated in the new language.
- **FM names in Portuguese:** with the app in Portuguese, roles, instructions and their options, mentalities, player and opposition instructions, Tactical Tweaks and training areas appear with Portuguese names, also inside sentences. For roles, 34 names are **confirmed in the Portuguese FM26**, from tactics published by Salão de Jogos, and are marked with ✓ in `src/i18n/pt_terms.js`. The rest follow the game's same naming pattern. FM27 may have occasional differences.
- **FM27 sheet in Portuguese:** by default it keeps the names as they appear in the English game, so the tactic can be recreated without mistakes. The **Português** button shows it with translated names.
- **Bible search:** finds roles by name in both languages.
- **Automated tests:** `tests/ui_i18n.py` fails the build if any text stays in Portuguese with the app in English, including errors and temporary messages. `tests/ui_pt_terms.py` fails if any FM term stays in English with the app in Portuguese. `tests/i18n_audit.py` is a supporting static audit. `tests/ui_lang_audit.py` is the two-way audit: it extracts each language's vocabulary from the code itself and looks for Portuguese with the app in English and English with the app in Portuguese, across every screen and several states. Before running, it checks that the detectors catch known examples.

## Interface
- **Collapsible panels:** click the ▾ (or the title) of any panel to close or open it. The app remembers the state on each screen.
- **Icon-only menu:** « button at the top of the menu or **Ctrl+B**. In narrow windows (e.g. 1366 at 125% scaling) the menu switches to icons automatically.
- **Game data always at hand:** the "Game data" panel in the sidebar has the **📥 Read the save now** and **Latest F9** buttons, and shows how long ago the squad was read. The same buttons appear at the top of Squad, in step 1 of Home and on screens that need the squad. Shortcut: **Ctrl+Shift+L**. In the browser, the button opens the squad file.
- **Drag and drop** any file onto the window.
- **Activity always visible:** anything that takes time shows it's working. A thin bar appears at the top of the window and a card in the sidebar shows what's happening, the elapsed time and, when known, the percentage. The card never covers content. That task's buttons show a spinner and are locked until it finishes. It's hooked up to save reading (with a notice when the first read takes a while), F9, large files, the Generator (pair-by-pair progress) and `.fmf` analysis.
- **UI tests on GitHub:** `ui_layout.py` runs fully in English and in a quick pass in Portuguese. `ui_busy.py` checks the activity indicators and `ui_i18n.py` the language coverage.
- **Layout gate:** `tests/ui_layout.py` goes through the 17 screens at 6 widths and 3 modes (normal, collapsed menu, closed panels) with a 62-player squad with long names. It runs on GitHub in parallel with every build (10-minute limit), and the Release is not published if it finds overlaps or errors.

## Available roles per position
- **Exact slot:** in FM26/27 some roles only exist in the central slot of a line (DM, MC, AMC, ST) or only in the offset slots (DML/DMR, MCL/MCR…). The app respects this in the analyser, animated pitch, Generator and templates. A role the game doesn't allow in that slot doesn't appear in the list and is flagged as invalid.
- **Confirmed:**
  - The Box-To-Box Midfielder and the Box-To-Box Playmaker don't exist in central midfield, only in the DM line.
  - The Box-To-Box Midfielder only exists in the offset DM (DML/DMR).
- **Full validation:** the app has a hook (`ROLE_AVAIL_ENGINE`) to apply an exact per-position role list to all 77 roles, if one becomes available.

## Limitations

- **Save reading.** Reading is done by [fmsave](https://github.com/rhiever/fmsave) (MIT, by Randal S. Olson), always on a **copy** of the save. It never changes the save, doesn't read the game's memory and doesn't need FM open. fmsave marks some FM27 fields as "unconfirmed". The app only uses name, age, club, positions and attributes.
- **Reading the game's memory, like FMPT Chief Analyst.** Not needed: the save has the same data and doesn't break with every patch.
- **`.fmf`:** FM27 encrypts the tactic inside `.fmf`, so the app doesn't read or write it. The tactic goes back into the game via the Sheet (and the XI via the optional bridge).
- **Attribute weights.** Roles marked EST use estimated weights. They can be adjusted in `src/core/roles.js`.
- **Animated pitch.** It's a model of each role's and instruction's intent, not the FM match engine.

## Structure

```
src/i18n/   i18n.js (L/T/translateDOM) · en_dict.js (PT→EN dictionary)
src/core/   roles.js · tactics_data.js · engine.js · insights.js (PIs, opponent, set pieces) · generator.js (generator) · sim_model.js
src/ui/     state.js · components.js · sim_view.js · views_*.js · actions.js · main.js
build.mjs   bundles everything into app/index.html (single file)
tests/      npm test — engine, simulator (473 scenarios), insights, real .fmf and FM View Exporter CSV
src-tauri/  native window: latest save (.fm) → fmsave → squad; latest F9 export; local TCP client for the optional in-game bridge plugin
```

Not affiliated with Sports Interactive or SEGA.
