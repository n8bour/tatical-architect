# Tactical Bible Bridge (optional)

A minimal BepInEx 6 (IL2CPP) plugin that runs inside FM27 and lets FM27 Tactical Bible **apply** things in the game
(currently: place your XI into the active tactic). Windows only. The app works fully without it.

Trimmed from FM27Bridge (fm27-assistant, MIT, © 2026 arress03). Listens on `127.0.0.1:27029` (different from
FM27Bridge's 27027, so both can be installed). Data folder: `%APPDATA%\FM27TacticalBible` (`bridge.log`, `bridge_refs.txt`).

## Build (needs FM27 + BepInEx 6 IL2CPP, launched once so `BepInEx\interop` exists)
```
dotnet build bridge/TacticalBibleBridge -c Release -p:GamePath="C:\...\Football Manager 27"
```
The build copies the DLL to `BepInEx\plugins\TacticalBibleBridge`. CI can't build it (the interop DLLs come from the game).
Any .NET SDK ≥ 6 works (tested with 8.0.428). Restart FM after building: the plugin is loaded at launch.

## Testing without the app
- `node bridge/ping.mjs [port] [commands…]`: tiny TCP client (default `PING`, `VERSION`, `REFS` on 27029).
- `bridge/fmb-node.mjs`: runs the app's real `src/ui/fmbridge.js` (`FMB`) from Node over TCP, for scripted tests
  (`readTactic`, `personsOf`, `applyXI`, `applySetPieces`…). Writes go to the running game: use a save you won't keep.

## Verified on FM27 (27.1.0.2505117, Windows, plugin 0.1.0)
- Loads next to FM27Bridge (27027) and other BepInEx plugins; no Harmony conflicts.
- `PING` says `waiting` until FM's UI opens its first channel (main menu is enough), then `ready`.
- References are learned only once FM's UI has used them: after loading a save, `CalendarReference` (enough for
  `FMB.context()`), `TeamTacticReference` and `TeamTrainingReference` are known. **`SetPieceReference` is only known after
  opening Tactics › Set Pieces once**; before that every set-piece `READ` returns `null`.
- **Place my XI** ✅: tactic name, 11 slots + persons, selection order and person↔uid map read correctly; placing a bench
  player in a slot and putting the original back restores the exact original state.
- **Set pieces › Apply in FM** ✅ (checked on FM's screen): corner + direct free-kick takers per foot replaced (old
  entries removed, no duplicates), corner technique (inswinging) and aim for (far post) applied.
- **Corner routines** ✅ (app `FMB.applyRoutine`, plugin ≥ 0.3): with Tactics › Set Pieces › Corners open in FM, each slot gets
  the instruction the app planned for the player FM put there (attacking and defending; the app asks the user to click
  Attacking/Defending). Verified on both, with read-back and restore (`bridge/dev/_routine.mjs`).
- Read-back limits: `1786266473` (takers) returns just one 2-entry list `{sel, person}` with no kind/foot, unchanged by
  the apply. A plain `READ` of `1867869768` (routine) returns FM's empty default: use `READAT auto` (see below).

## Protocol (one line = one command)
`PING` · `VERSION` · `REFS` · `READ type|pairs|prop ;; …` · `SENDJ <ev> <type> <pairs|-> <json>` · `WRITE type|pairs|prop <json>` · `EVENTS` · `RECORD 0|1`

Diagnostics (0.3+): `READAT <parentHex|auto> …` (read under FM's own screen node) · `TAP [filter]` · `TAPON <types>` ·
`UIKEYS [filter]` · `WRITEUI type|-|prop <json>` · `SEQ 0|1` (full channel log to `bridge_seq.txt`) · experimental (0.4.1):
`DATAHOOK 1`, `DATA [filter]`, `SETDATA <index:version> <json>`. Full list at the top of `Plugin.cs`.

## After an FM27 patch
1. Launch FM; check `BepInEx\LogOutput.log` and `bridge.log` — the plugin must load.
2. Settings › Integrations › *Check connection* → "connected", FM version shown.
3. On a **test save**: *Place my XI in FM* and compare with the game's tactic screen.
4. If it fails: send `EVENTS`, do the action by hand in FM, compare the ids FM sent with `FMB_IDS` in `src/ui/fmbridge.js`
   (every event/property/key id lives in that one block) and update them.

## Capturing a new action
Settings › Integrations › Tactical Bible Bridge › **Record an action in FM** → change *one* option by hand in FM →
**Stop recording** → **Copy**. The JSON has every order FM sent (event id, reference, payload). Record each option on
its own, ideally twice with two different values, so the id and the value field are easy to tell apart.

### Set pieces: what is decoded (FM27 EN, recorded through FM27Bridge)
| Setting | Event | Payload | In the app |
|---|---|---|---|
| Corner / FK takers per foot | add L `1213227378`, add R `1416066355`, remove `910380099` | entry {kind, list, sel, person} | ✅ |
| Corner technique | `927877996` | DynamicNumber 0 in / 1 out | ✅ |
| Corner aim for | `1249537108` | {label, number}: Mixed 0, Short 1, Near Post 2, Far Post 3, Edge of Area 4, Centre 5 | ✅ |
| Penalty order | `1649488964` | {inner {sel: current index, person}, `1782997593`: new index} | needs current order |
| Move in a taker list | `1144009829` | {inner {kind, list, sel: current index, person}, `1782997593`: new index}; kind 9 = long throws | needs current order |
| Corner slot instruction | `1633039705` | {`1903577654` label, `1886680684` {group `1114920816`, slot `1883858041`, option `1750627616`, kind `1398883404`, `1279880290` 0}} | ✅ verified (plugin ≥ 0.3) |
| Move an instruction to another category / position | `1449477207` | SI.Core.Record {`929121399` (source): {`929121399`: slot {slot, group}, `1769491558` 0, `857762170` 1, `877999731` 0}, `944851554` (target): {`944851554`: group {group} (drop into the group) or slot {slot, group} (drop on that slot), same 3 flags}} | ✅ verified with minimal records; the moved slot's instruction resets to a default, so set it again after |
| Captains | — | — | left manual |

**Routine kinds** (`1398883404`): attacking corners 5, defending corners 0, attacking free kicks 6, defending free kicks 1,
throw-ins 9. Routine field `1919054668`: 1 attacking, 2 defending. Tab channel `1668375608` {number, label}: Corners 0,
Shooting Range 1, Attacking Third 2 (the full tab list, with Wide 3, Deep 4, Middle Third 5, Defensive Third 6, is in `1466193512`).

### Reading the routine (task 3, solved on FM27 27.1.0.2505117)
- The routine (`1867869768`) depends on the **view context of FM's Set Pieces screen**, not only on the reference. A channel
  opened under the root gets FM's empty default (kind -1). Opened under the parent node of the channel FM's own screen has
  open, it gets the real routine: `READAT auto FM.UI.SetPieceReference|<pairs>|1867869768` (plugin ≥ 0.3). So **the Set
  Pieces screen must be open in FM**. `TAP` returns the last data FM's screen itself received (same content).
- The routine read = the view selected on FM's screen (tab + attacking/defending). Each slot has: person `1886157170`,
  name `1851878757` (a1, b2…), current instruction `1750627616` {label, {group, slot, option, kind}}, and **its own option
  list** `1833723970` (items {`1383036740`: {label, {…, option}}}): pick options by label from it.
  Groups: in the box `1750096198`, out of the box `945304884`; each {group `1114920816`, slots `1752528242`}.
- Options seen (FM27 EN). Attacking corners: Go Forward 4, Attack Near Post 5, Attack Far Post 6, Attack Ball from Edge of
  Area 7, Lurk at Near Post 8, Lurk at Far Post 9, Mark Keeper 11, Lurk Outside Area (Centre) 10 / (Near Side) 14 /
  (Far Side) 15, Come Short 12, Stay Back 2, Stay Back If Needed 3. Defending corners: Go Back 1, Stay Forward (Centre) 2,
  Mark Near Post 3, Mark Far Post 4, Player Mark 6, Zonally Mark Six Yard Box Near Post 7 / Near Centre 8 / Centre 9 /
  Far Centre 10 / Far Post 11, Edge of Area 12, Stay Forward (Near Side) 13 / (Far Side) 14.
- Instruction writes carry the kind, so they **don't depend on the selected view** (verified: a defending-corner slot changed
  while FM showed attacking).
- Attacking/defending on FM's screen is a UI context value (SetData on a session-specific data slot, e.g. `7600:4` = 1/2).
  `DATAHOOK 1` + `SETDATA` flip the engine's view, but FM's screen does not follow, so the app should ask the user to click
  Attacking/Defending instead. **Never patch SetData/AddData/SetTarget at plugin load: that crashed FM27 before the main
  menu (0.4.0).** `DATAHOOK 1` applied mid-game worked.
- Writing the tab through FM's own channel (`WRITEUI`) changes the stored value but neither FM's screen nor the routine.
