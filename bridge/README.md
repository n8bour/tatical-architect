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

## Protocol (one line = one command)
`PING` · `VERSION` · `REFS` · `READ type|pairs|prop ;; …` · `SENDJ <ev> <type> <pairs|-> <json>` · `WRITE type|pairs|prop <json>` · `EVENTS` · `RECORD 0|1`

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
| Corner slot instruction | `1633039705` | {label, {group `1114920816`, slot `1883858041`, option `1750627616`, kind `1398883404` (5 att, 0 def)}} | needs slot layout |
| Swap two corner slots | `1449477207` | full slot state | not replayed |
| Captains | — | — | left manual |

Slots are filled by FM from its own rankings (Aerial Threats, Box Threats, Trackers…), so the app sets the instruction per
slot, not per player. To wire the rest we need FM's current state: with FM27Bridge, use **Sample screen data** in Settings,
open every set-piece tab in FM, stop, and share `%APPDATA%\FM27Scout\bridge_data.txt`.
