# Handoff: the FM27 bridge (state at v5.2.0)

Read `bridge/README.md` first (decoded ids and verified behaviour) and `src/ui/fmbridge.js` (`FMB_IDS` holds every FM id; `FMB` is
the client). Everything below was verified against a running FM27 (27.1.0.2505117, EN, Windows) with the user checking FM's screen.

## What the app is
FM27 Tactical Bible: Tauri 2 app (Rust shell + one-file JS UI built by `build.mjs` into `app/index.html`). Squad and the whole player
database come from the save (fmsave). The optional BepInEx 6 plugin `bridge/TacticalBibleBridge` (port 27029, line protocol) lets the
app read and change the running game. The plugin DLL ships inside the app (`bridge/dist`, `include_bytes!`) and is installed from
Settings › Integrations. It is the only plugin the app talks to (the FM27Bridge 27027 fallback was removed).

## Done and verified (all checked on FM's screen)
- Place XI; set pieces (takers per foot, corner delivery/target, attacking + defending corner routines slot by slot).
- **Tactic › Apply in FM / Create new in FM:** in- and out-of-possession formations, XI, roles (68 of 77 app roles have an FM id),
  mentality, 26 team instructions, player instructions. Create new = FM27 Assistant's wizard sequence (no event 90), refuses at 3 tactics.
- Scouting: shortlist to FM (FM search + `unique_id`), Recruitment Focus (scout assignment) from a search.
- Next opponent from FM (fixture → team → players → save database). F9 / FM View Exporter and FM Tactics Lab are gone.

## Gaps still open (none blocks the current release)
1. **Roles without an FM id:** lhk, ocb, cwcb, swcb, hwb, prwb, fr, ss, soam. Click a player in a position where the role exists in FM
   and read the role list (see "Role lists" in `bridge/README.md`; `dev/_tienum.mjs`/`dev/_pienum.mjs` show the technique).
2. **App instructions with no FM tile:** player instructions Hold position, Move into channels, Arrive Late, Attack the Defensive
   Line; team instruction "Play for Set Pieces → Balanced". Hold Up Ball (5), Attack Box Side/Depth, Marking Target options and the
   second option of Marking (12) were never mapped.
3. **Not decoded:** penalty order (`1649488964`), long throws / taker list moves (`1144009829`), opposition instructions, captains,
   Position vs Player scope of player instructions (only Position is written), team-instruction ids 2, 13, 14 (never appeared).
4. **Opposition report and live match data** (what TeknikOda shows): not started. The next fixture is readable; FM's own opposition
   analysis screen and the match tactic references (`MatchTacticReference`, same event ids as the tactic) are the leads.
5. **Not run from the installed app:** every FM-facing button was exercised through `bridge/dev/*.mjs` (the app's own `FMB` code over TCP),
   not by clicking in the installed app. Rust compiles on CI only (no Rust needed locally).
6. **BepInEx is a prerequisite** the app does not install (builds.bepinex.dev, BepInEx 6 IL2CPP).

## Hard-won rules (read before touching the bridge)
- FM27's replayed events: the **‹ event works on any tile, › only on the tile last clicked by hand** (player instructions).
- Team-instruction arrows drop presses that arrive too fast: 150 ms between presses.
- **Never replay the recorded "Create New Tactic" wizard with event 90**: it replaced the active tactic. Use `FMB.createTactic`.
- Never patch SetData/AddData/SetTarget at plugin load (crashed FM before the main menu); `DATAHOOK 1` mid-game is fine.
- `READ` from the plugin gets FM's empty default for screen-scoped data: use `READAT auto` with the screen open.
- Log file `%APPDATA%\FM27TacticalBible\bridge_seq.txt` grows ~100 MB/min: turn `SEQ 0` off when done; it can stall past ~0.5-1.5 GB.
- Don't touch the user's saves; ask before changing a real game; keep the plugin's port 27029 and Harmony ids unique.
- After changing `Plugin.cs`: build (needs the .NET SDK ≥ 6; none is installed now), copy `bin/Release/TacticalBibleBridge.dll` to
  `bridge/dist/`, bump `Plugin.Version` and `PLUGIN_VERSION` in `src-tauri/src/main.rs`, restart FM.

## Releasing (only when the user asks)
Bump the version in `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `src/ui/state.js` (CI fails if they differ),
edit `.github/RELEASE_NOTES.md`, `node build.mjs && npm test`, push to `main`. CI builds only if release `v<version>` doesn't
exist and runs the Playwright UI checks (layout EN/PT, busy, i18n, language audit). Releases contain only the two `.exe` files.
Local tooling is not installed any more: Playwright tests run on CI (`uv run --with playwright==1.56.0 python tests/ui_*.py` locally).
