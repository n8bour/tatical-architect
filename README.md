# FM27 Tactical Architect

Native Rust + GPUI tactical analyser, generator, scouting tool and animated pitch for Football Manager 27 (plus a single-file offline browser build). Works offline and needs no account.

## How to use (3 steps)

1. **Squad.** The squad comes from your save, read by the free fmsave tool. The cards in **Squad** and **Settings › Integrations** say what it **requires** and show the status on your PC.
   Automatic save reading is **off** by default. Turn it on in the card, or use "Read the save now".

   **Squad, read from the save:**
   The app always runs the **latest fmsave** (`uvx fmsave@latest`). When FM27 gets a patch, as soon as an fmsave version that supports it is released, the app uses it automatically. Without internet, it uses the version already cached. If you installed fmsave with pip, update it with `python -m pip install --upgrade fmsave`.
   - **Desktop app:** save the game in FM27 and the app reads the save by itself (it checks every 10 seconds). The first time, you need to install the reader:
     ```
     winget install --id=astral-sh.uv -e
     ```
     Restart the app after installing. On the first read, `uv` downloads [fmsave](https://github.com/rhiever/fmsave) and needs internet only that once.
   - **Browser:** open [fmsave online](https://rhiever.github.io/fmsave/app/), drop your `.fm` there (the save is read on your PC and never uploaded), download the squad CSV and drag it into the app.
2. **Tactic.** In **My team › Generate tactic** the app builds a complete tactic for your squad. You can also create your own in **Tactic › Analyse**. In **Tactic › Analyse**, click a player to change roles and instructions. **Animated pitch** shows the tactic in motion.
3. **Opponent.** With FM open and the optional plugin installed, the app finds your **next match** and reads the opponent's squad from the save (**Next opponent from FM**). Without it, drag a CSV/HTML of their squad in. The report is in **My team › Opponent**. The game plan is applied to the tactic with one click.

To take the tactic into the game, use **Tactic › FM27 sheet**. The sheet follows the order of the tactics screen and has checkboxes to tick as you go. **Print/PDF** (on the sheet and in **Tactic › Analyse**, which adds the diagnosis and synergy) opens a printable page in your browser, where you can print it or save it as PDF.

## What's inside

| Section | What it does |
|---|---|
| Home | Status of the 3 steps, what to fix first, the system that best suits the squad and the opponent summary |
| My team › Squad | Save reading (fmsave) or import (CSV/HTML/JSON from fmsave, FMPT Chief Analyst or FMLibreEditor) and each player's best roles |
| My team › **Generate tactic** | A complete tactic built around the squad. Tests every pair of in-possession and out-of-possession formations, picks the IP/OOP role for each position using balance rules, and the exact XI (Hungarian algorithm). Generates the 27 team instructions, each with its reason, by comparing each unit with the squad average, plus player instructions where the attributes call for them. Includes set pieces, captains, depth per position, where to strengthen, 3 alternatives and training. Takes into account the league objective, mandatory starters, a star player, up to 3 youngsters to develop and the weak foot on the flanks. Buttons to use the tactic, view it on the pitch or copy it as text |
| **Role synergy — all phases** (Generator and Analyser) | Measures whether roles serve and cover each other, not just whether each player is in his best role, using real attributes. **Build-up:** who plays out → who receives, support between the lines, press resistance, long ball → target. **Attack:** creator → finisher, crosses → targets, front pairing, box presence, flanks, creative load. **Defence:** pressers have cover, centre-back pairing (one steps out, the other covers), protection in front of the centre-backs, flank defence (a winger who stays high forces the full-back to hold), line vs. profiles, aerial game. **Transitions:** rest-defence on losing the ball, who launches → who runs in the counter, number of outlets. Two maps (in and out of possession) and problems per phase, each with its fix. The Generator weights 55% suitability, 30% synergy and 15% balance, and in tests it improves every phase without sacrificing defence |
| **Scouting** | Every player in the save, read by fmsave (`export players --all`) from a copy, cached until the save changes. Includes attributes, CA/PA, personality, hidden attributes, contract, wage and value. **11 preset searches:** interested in joining (estimate), cheap wonderkids, wonderkids who will reach their potential, expiring contracts, free agents, bargains, signings for my tactic, free veterans, hidden gems, elite personality and my weakest position. **Filters:** name, age, CA/PA, position (competent or natural), foot, height, club, league country, nationality, value, wage, contract end, club reputation, free agents, loans, personality, hidden attributes, chance of reaching potential, chance of wanting to join your club (estimate), minimum attributes (several) and role with minimum suitability. Sortable, paginated table, player profile (attributes, best roles and where he would fit in your XI), shortlist and CSV export. In the browser, it accepts the all-players CSV from fmsave online. The **chance of wanting to join** is an estimate (fmsave doesn't read the game's "interested" flag). It uses your club's reputation against his club's, his reputation, his contract (free, expiring, squad status), ambition and loyalty, and shows the reasons in the profile |
| My team › **Squad** | The player table has three views: **Roles** (best roles in and out of possession), **Ability and contract** (CA, PA, chance of reaching PA, value, wage, contract end, status) and **Personality and hidden** (professionalism, ambition, pressure, loyalty, temperament, adaptability, consistency, important matches, injury proneness, versatility). Every column is sortable and each player opens the full profile. This data comes from the save read by fmsave; other CSVs only bring what they contain |
| My team › Best XI | Suitability per position, better alternative role, **player instructions recommended by attributes**, backup and successor (≤ 21 years old), and 9 systems ranked for the squad |
| My team › Opponent | 9 units compared with your XI, the 3 most dangerous players with opposition instructions and an applicable adjustment plan |
| Tactic › Analyse | In- and out-of-possession formation, roles, instructions and diagnosis (rest-defence, width, box, pressing, transitions, FM27 complexity, suitability) |
| Tactic › Animated pitch | 6 phases of play with each role's movement, plus the effect of the instructions in real time |
| Tactic › **Match situations** | Plan B/C from the current tactic and squad, in five scenarios: **We need goals**, **Hold the result**, **Away against stronger sides**, **Away against an equal side** and **At home against weaker sides** (low block). Each plan says what changes and why, using the players' attributes: mentality, team instructions, roles, player instructions (added and removed), Tactical Tweaks, substitutions from the bench and opposition instructions for their three most dangerous players. Role changes are tested against synergy: in an attacking plan they're only made if they don't weaken the attack, and in a defensive plan only if they don't weaken the defence. The impact shows attacking commitment and defensive security (intensity) and synergy per phase (coherence). You can save the plan as a tactic, use it right away or copy it as text |
| Tactic › Set pieces | Takers (corners with the right foot, direct and crossed free kicks, penalties, long throws), captain and vice-captain, attacking and defending corners in a diagram (zones, marking the opponent's strongest in the air, who stays back and who stays up) and applicable tips |
| Tactic › FM27 sheet | Step-by-step guide to recreate the tactic in the game, including the *Set Pieces & Captains* section |
| Training | Tactical Understanding priorities, typical week, Role Training and focus per player |
| **Apply in FM** (optional, Windows) | With the optional bridge plugin (`TacticalBibleBridge.dll`, kept under that filename for FM27/BepInEx compatibility, installed from *Settings › Integrations*; needs BepInEx 6 IL2CPP installed first, from builds.bepinex.dev; it is the only plugin the app talks to): **Place XI in FM** (Tactic › Analyse), **Apply in FM** for the tactic (formations, XI, roles, mentality, team and player instructions; can also create a new tactic) and for Set pieces (corner and free-kick takers per foot, corner delivery, and the attacking and defending corner routines slot by slot), **Shortlist → FM**, **Scout in FM** (creates a Recruitment Focus), **Next opponent from FM** and **Refresh squad from FM** (live squad). Verified on FM27 27.1.1. See `bridge/README.md`. |

You can drag any file anywhere onto the window. Your club's squads go to Squad; another club's squads go to Opponent; `.fmf` files go to the inspector; `.json` backups are restored.

## Install & Build

**Native desktop app (Rust + GPUI — GitHub Actions, nothing to install on your PC):**
1. Push the repository to GitHub. The workflow in `.github/workflows/build.yml` runs on every push to `main` (or via **Actions › Build desktop app › Run workflow**).
2. It reads `[workspace.package] version` from `Cargo.toml` (verifying it matches `package.json` and `src/ui/state.js`) and, if release `v<version>` doesn't exist yet, runs all Rust & JS tests, compiles the native GPUI desktop binary (`ta-app`), and publishes the release with `.github/RELEASE_NOTES.md`.
3. Open **Releases** and download from *Assets*:
   - `FM27-Tactical-Architect-<version>-setup.exe` (Windows NSIS installer)
   - `FM27-Tactical-Architect-portable.exe` (Windows portable binary, runs without installing)
   - Optional macOS `.dmg` and Linux `.tar.gz` when running with `all_platforms` enabled in *Run workflow*.

**To publish a new version:** bump the version in `Cargo.toml`, `package.json`, and `src/ui/state.js` (CI fails if they differ), update `.github/RELEASE_NOTES.md`, and push to `main`.

**Locally (native Rust + GPUI desktop app):**
The workspace uses the Rust toolchain pinned in `rust-toolchain.toml` (`1.100.1`).
```bash
cargo test --workspace       # run all Rust unit & parity tests (ta-core, ta-backend, ta-app)
cargo run -p ta-app          # launch the native GPUI desktop app
cargo build --release -p ta-app  # build release binary (target/release/fm27-tactical-architect)
```
Convenience npm scripts: `npm run app:dev` (`cargo run -p ta-app`) and `npm run app:build` (`cargo build --release -p ta-app`).

**Single-file offline browser build:**
Double-click `app/index.html` (rebuilt via `node build.mjs`, tested with `npm test`). It has everything except native save polling and the FM plugin, which require the desktop app.

## Language
The app opens in **English** by default. Portuguese is chosen with the **EN/PT** selector at the bottom of the menu, and the choice is remembered.
- **Full translation:** covers the interface, role/instruction descriptions and tips, training, and the messages from the diagnosis, generator, opponent report, and set pieces.
- **Generator:** when switching language, the report's reasons are recalculated in the new language.
- **FM names in Portuguese:** with the app in Portuguese, roles, instructions and their options, mentalities, player and opposition instructions, Tactical Tweaks and training areas appear with Portuguese names, also inside sentences. For roles, 34 names are **confirmed in the Portuguese FM26**, from tactics published by Salão de Jogos, and are marked with ✓ in `src/i18n/pt_terms.js` (and mirrored in `crates/core/src/data/i18n.rs`). The rest follow the game's same naming pattern. FM27 may have occasional differences.
- **FM27 sheet in Portuguese:** by default it keeps the names as they appear in the English game, so the tactic can be recreated without mistakes. The **Português** button shows it with translated names.
- **Automated tests:** `cargo test --workspace` runs the Rust parity and i18n checks plus a headless GPUI render of all 14 desktop screens in both languages (with and without a squad); for the HTML build, `tests/ui_i18n.py`, `tests/ui_pt_terms.py`, and `tests/ui_lang_audit.py` catch untranslated text in both directions across every screen.

## Interface
- **Collapsible panels:** click the ▾ (or the title) of any panel to close or open it. The app remembers the state on each screen.
- **Icon-only menu:** « button at the top of the menu or **Ctrl+B** / **Cmd+B**. In narrow windows the menu switches to icons automatically.
- **Game data always at hand:** the top bar has the **Read the save now** button, and shows how long ago the squad was read. Shortcut: **Ctrl+Shift+L** / **Cmd+Shift+L**.
- **Drag and drop** any file onto the window.
- **Activity always visible:** long-running tasks run off the UI thread and display a top progress bar and floating top-right activity card (never covering content) with elapsed time and percentage.
- **Layout gate (HTML build):** `tests/ui_layout.py` checks the 14 screens at 6 widths and 3 modes on CI in parallel with the Rust build.

## Available roles per position
- **Exact slot:** in FM26/27 some roles only exist in the central slot of a line (DM, MC, AMC, ST) or only in the offset slots (DML/DMR, MCL/MCR…). The app respects this in the analyser, animated pitch, Generator and templates. A role the game doesn't allow in that slot doesn't appear in the list and is flagged as invalid.
- **Confirmed:**
  - The Box-To-Box Midfielder and the Box-To-Box Playmaker don't exist in central midfield, only in the DM line.
  - The Box-To-Box Midfielder only exists in the offset DM (DML/DMR).

## Limitations

- **Save reading.** Reading is done by [fmsave](https://github.com/rhiever/fmsave) (MIT, by Randal S. Olson), always on a **copy** of the save. It never changes the save, doesn't read the game's memory and doesn't need FM open. fmsave marks some FM27 fields as "unconfirmed".
- **Reading the game's memory, like FMPT Chief Analyst.** Not needed: the save has the same data and doesn't break with every patch.
- **`.fmf`:** FM27 encrypts the tactic inside `.fmf`, so the app doesn't read or write it. The tactic goes back into the game via the Sheet (and the optional bridge plugin).
- **Attribute weights.** Roles marked EST use estimated weights. They can be adjusted in `crates/core/src/data/roles.rs` (and `src/core/roles.js`).
- **Animated pitch.** It's a model of each role's and instruction's intent, not the FM match engine.

## Structure

```
crates/
  core/        ta-core: pure-Rust engine (data/, engine/, planning/, live/, types.rs) + parity tests
  backend/     ta-backend: fmsave runner/cache, embedded bridge DLL installer, TCP bridge client (127.0.0.1:27029), native dialogs & JSON state store
  app/         ta-app: native GPUI desktop app (state, custom canvas widgets, 14 bilingual screens)
assets/icons/  application icons (.ico, .icns, PNG, SVG)
packaging/     installer.nsi (Windows NSIS installer script)
src/           offline browser HTML sources (i18n/, core/, ui/, styles.css)
build.mjs      bundles src/ into app/index.html (single file)
tests/         npm test (engine.test.cjs) · sim_canvas.test.cjs · Playwright UI checks ui_*.py
bridge/        TacticalBibleBridge (C# BepInEx plugin source), dist/ (prebuilt DLL embedded in ta-backend), dev/ scripts, README.md (decoded FM ids)
docs/          HANDOFF-bridge.md (state of the bridge, open gaps, rules)
```

Not affiliated with Sports Interactive or SEGA.
