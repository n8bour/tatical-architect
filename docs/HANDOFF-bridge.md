# Handoff: finish the FM27 bridge on the Windows machine

Read this first, then `bridge/README.md` and `src/ui/fmbridge.js` (`FMB_IDS` holds every FM id).
Previous work was done on a Linux server with no game, so **nothing below was ever run against FM27**.

## What the app is
FM27 Tactical Bible: Tauri 2 app (Rust shell + one-file JS UI built by `build.mjs` into `app/index.html`).
Squad data comes from fmsave (save file) or F9 export. Version now: **4.13.4** (bump in 4 places, see "Releasing").

## The optional bridge (what this task is about)
A BepInEx 6 (IL2CPP) plugin inside FM27 that the app talks to over `127.0.0.1` (one line = one command).
- Our own plugin: `bridge/TacticalBibleBridge/Plugin.cs`, port **27029** (trimmed from fm27-assistant's FM27Bridge, MIT).
- Fallback: fm27-assistant's FM27Bridge (separate repo `n8bour/fm27-assistant`, port **27027**, same protocol, has `SAMPLE`).
- Rust client: `bridge_call` in `src-tauri/src/main.rs` (tries 27029 then 27027).
- JS: `src/ui/fmbridge.js` (status, read, send, applyXI, applySetPieces, recorder, dumpSetPieces).
- UI: Settings › Integrations card; Tactic › Set pieces › "Apply in FM".

## Setup on this machine
1. FM27 installed (Steam), BepInEx 6 IL2CPP already working (fm27-assistant's installer does it). Launch FM once so `BepInEx\interop` exists.
2. Install: Git, Node 22, **.NET 6+ SDK**. (Rust only if building the Tauri app locally; CI builds releases.)
3. `git clone git@github.com:n8bour/tatical-architect.git`
4. Build the plugin (copies itself into the game):
   `dotnet build bridge/TacticalBibleBridge -c Release -p:GamePath="<FM27 folder>"`
   It was **never compiled** — expect small compile errors; fix them. Do not run it together with FM27Bridge on the same
   hooks if they conflict (different ports/Harmony ids, should coexist; verify).
5. `npm test` and `node build.mjs` work anywhere.

## Tasks, in order
1. **Compile the plugin and get `PING`/`VERSION`/`REFS` answering** on 27029 (test with a tiny PowerShell/Node TCP client).
2. **Verify what already ships** against a throwaway save (note: user plays on a real save; never test on it without asking):
   - Place my XI in FM (`FMB.applyXI`).
   - Set pieces › Apply in FM (`FMB.applySetPieces`): corner + direct-FK takers per foot, corner swing, corner aim-for.
   Fix anything wrong; record real behaviour in `bridge/README.md`.
3. **Slot layout / routine read (the open problem).** Reading `SetPieceReference` props `1867869768` (routine) and
   `1786266473` (takers) through our top-level `READ` returns FM's *default empty routine on every tab* (see
   "What we learned"). FM27Bridge's `SAMPLE` did capture the real data because it logs the UI's own channel traffic.
   Idea to try: the UI opens channels as **child nodes** (`AddNode(key, parentKey, prop)`), not under `RootKey`. Log the UI's
   `OpenChannel`/`AddNode` sequence while the Set Pieces screen is open (FM27Bridge had `bridge_seq.txt` via `Plugin.Seq`) and
   replicate the parent chain in `Plugin.Read`. Also remove the 4000-char cut in sampling if you use it.
4. Only if 3 works: wire penalties order (`1649488964`), long/throw-in takers (kind 9), per-slot corner instructions
   (`1633039705`). Otherwise leave them manual and remove the "📋 Copy set-piece state" button.
5. Bundle our plugin (or not): installer currently ships no plugin. fm27-assistant keeps `payload/bepinex.zip`; decide with the user.

## What we learned (from recordings, FM27 EN)
- Order `1249537108` = corner aim-for: record `{label(1903577654), number(1886680684)}`: Mixed 0, Short 1, Near Post 2, Far Post 3, Edge of Area 4, Centre 5. (The plain-number form of the same event is another setting.)
- Takers: see `FMB_IDS.sp.ev` (add L `1213227378`, add R `1416066355`, remove `910380099`), entry `{kind(1398883404), list(961376122), sel(1883858041), person}`; kinds: corner 5, free kick 6, long throw 9.
- Tab type channel `1668375608` (writable): Corners 0, Shooting Range (free kicks) 1, Attacking Third (throw-ins) 2.
- Corner slot instruction: ev `1633039705` `{label, {group 1114920816, slot 1883858041, option 1750627616, kind 1398883404 (5 att, 0 def), 1279880290: 0}}`.
  Known options: attacking Go Forward 4, Attack Near Post 5, Attack Ball from Edge 7; defending Go Back 1, Stay Forward (Centre) 2, Player Mark 6, Zonal Six-Yard Far Post 11, Edge of Area 12.
  Groups: attacking Aerial Threats 4, Box Threats 7 (in the box), Creators 6, Recovery Defenders 5 (out of the box); defending groups 0/1/2.
- Penalty/list move: ev `1649488964` / `1144009829` = `{inner{current index, person}, 1782997593: new index}` — needs the current order.
- `bridge/README.md` has the full decoded table.

## Rules of the road
- Don't touch the user's saves; ask before anything that writes to the game on a real save.
- Don't modify `fm27-assistant` without asking (separate repo, user's other project).
- The user's own FM27Bridge may be installed: keep ports/Harmony ids separate.
- No new JS dependencies; the UI is plain JS concatenated by `build.mjs` (order in `ORDER`).

## Releasing (only when the user asks)
Bump the version in `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `src/ui/state.js` (CI fails if they differ),
edit `.github/RELEASE_NOTES.md`, `node build.mjs && npm test`, push to `main`. CI builds only if release `v<version>` doesn't
exist; UI checks (Playwright) run in parallel with a 10-minute limit. Releases contain only the two `.exe` files.

## Open questions for the user
1. Does FM27 have one free-kick taker list or separate direct/crossed lists?
2. In the long-throw recording, what was the plain 0/1 toggled on event `1249537108`?
3. Ship our own plugin in the installer, or rely on FM27 Assistant's?
