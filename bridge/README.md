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

**Shipping:** the app carries the plugin inside its executable (`include_bytes!` in `src-tauri/src/main.rs`) and installs it
from *Settings › Integrations* (`plugin_status` / `plugin_install`). CI can't build the plugin, so after changing `Plugin.cs`:
build, copy `bin/Release/TacticalBibleBridge.dll` to `bridge/dist/`, and bump `PLUGIN_VERSION` in `main.rs` (same as `Plugin.Version`).
BepInEx 6 itself is not installed by the app (FM27 Assistant's installer does it).

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

### Shortlist and recruitment focus (scout assignments): decoded and verified from the bridge (FM27 EN, plugin 0.4.1)
- **Find a player** (any player in the save): `WRITE FM.UI.SearchReference|825307168=98;<session>;1719300978=1;1954115685=0|1851878757 ["System.String","<name>"]`,
  then `READ` the same reference, property `1886547059` → list of `SearchResultReference` whose `1685218662` is the FM
  **person id**. Confirm with `PersonReference|…|1970170212` = fmsave **`unique_id`** (not `uid`). fmsave doesn't export
  the person id, so this lookup is required. Verified (`bridge/dev/_search.mjs`).
- **Shortlist add / remove**: event `44` on `ActionReference|825307168=1;<session>;1633904996=<action>`; add = action
  `1097488485` ("PersonAddPlayersToShortlist"), remove = `1500280650`. Payload {`1853189474` 1, `1214805357` {manager},
  `1685218662` {person}, `1685218636` [{person}], `2004054900` [""], `1936223853` "", `1954115685` 0, `1818719336` 0};
  remove also {`1415930182` {team}, `2003977071` True}. `1313882977` (chief scout) is not needed. Verified on FM's screen.
- **Recruitment focus** (`RecruitmentFocusItemReference|825307168=85;<session>;1348620377=<manager>;1769566810=<team>;1919314276=<id>`,
  id -1 = the one being created; its state lives in the game, so FM's own "create" screen shows what the bridge wrote):
  name `1851878757` (string) · min/max age `1394625906` / `929909872` ({number, label}) · min current/potential ability
  `1346584898` / `1347436866` (value = 10 + 2 × stars; step with events `1517250124` / `1431517038`, 1 up / 0 down) ·
  area `1817143629` {label, {`1685218662` id}} then event `0` adds it to the list `1986161206` (continents `83886080`+n,
  regions `318767104`…`131`, nations `167772160` + fmsave `nation_id`) · "Any position" `1366773845` · pitch position
  toggle event `879773492` {`1265792339` in/out of possession, `1349481321` position bits, `1330991441` slot} ·
  tactic `1750627946` · confirm event `55` (null). List: `ScoutingInfoReference|825307168=97;…|1953983593` (focus refs);
  delete: event `53` on `ScoutingInfoReference` with the id. Create/list/delete verified (`bridge/dev/_focus*.mjs`).

### Next opponent (verified FM27, used by My team › Opponent › Next opponent from FM)
`TeamReference|825307168=28;<session>;1685218662=<my team>` property `1852130921` = next fixture (`1668247142` = the previous
one) → `FixtureReference` whose `1717663572` / `1718119764` are the two teams (`1717859429` the date). Opponent's players:
`TeamReference|…=<their team>` property `926359584` (list of `PersonReference`; `1400137537` is the staff). Each player's
`PersonReference` `1970170212` = fmsave `unique_id` → the save database gives the attributes (52/52 matched for RB Salzburg).

### Tactics (work in progress, FM27 EN, recorded 22 Nov)
Active tactic = `TeamTacticReference|825307168=126;<session>;1651271022=1;1685218662=<team>`.
- **Verified from the bridge** (seen on FM's screen): mentality = `WRITE …|1162170689 {1886680684: n, 1903577654: label}`
  (options in `1280014444`: Very Defensive 1 … Balanced 4, Positive 5 …); move a position = event `21`
  {`929121399` {`929121399` from-bits, `1769491558` in-possession, `857762170` slot}, `944851554` {`944851554` to-bits, `1769491558`}}.
  In-possession bits = the app's `posBits`, plus `2^21` (left of a central pair) / `2^20` (right), e.g. DCL 2097168.
- Slots: `1919628150` (11 records: `1414752632` slot, `1835167823` IP position, `1668237639` OOP position, `1886157170` player).
  Out-of-possession positions use another bit set (32768, 2^35, 2^42 …): mapping still to do.
- **Role (in possession)**: event `1467503190` {`1414752632` slot, `1349481322` {`1886680684` 1, `1903577654` role record
  {`1349481322` role bit, `909713440` name, `1464367445` rating}}, `1265792339` True}. Role bits are global (ST: Deep-Lying
  Forward 1024, Centre Forward 2048, Target Forward 262144, Poacher 524288, Channel Forward 2^31, False Nine 2^40); the
  selected slot's role list (with names) is `2033407086`. Out-of-possession role change produced no event: to redo.
- Team instructions: changing directness / line of engagement / defensive line wrote `1381059181`, `1749303654` and events
  `1515149153`, `879515246`: to decode one at a time. Name `1851878757`, style `1836216183` (presets `1214728517`).

### Tactic › Apply in FM (verified on FM's screen, plugin 0.4.2)
- Left/right of a central pair: **left = 2^20, right = 2^21** (DCL = 16 + 1048576).
- Role event `1467503190` works in and out of possession (`1265792339` True/False); role bits are 64-bit (plugin ≥ 0.4.2).
  Read back: IP role = `930166885`[slot].`1698852728`; OOP role = `1798391651`[slot].`1349481322` minus the flags 2^20..2^23.
  The app's role → FM bits table is `FMB_IDS.tactic.roles` (68 of 77 roles; the others weren't on the positions clicked).
- Team instructions: event `862931569` = › next option, `1249276755` = ‹ previous; value = instruction id (1–30). The arrows
  stop at the ends, and FM drops presses that arrive too fast (150 ms between presses). Table: `FMB_IDS.tactic.ti`
  (instruction id, number of FM options, app option → FM option). Ids 2, 13 and 14 never appeared.
- Not done yet: out-of-possession formation (other position bits), creating a new tactic slot.
