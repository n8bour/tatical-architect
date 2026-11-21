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

Still to capture (set pieces): penalty takers order, long throw takers, crossed free-kick takers, corner positions
(near post / far post / edge / stays back …), defensive marking (zonal / man-marking / stay up), captain and vice.
Corner target zone: plain number, no label (confirmed). 1 = short, 3 = far post; other values unconfirmed.

## Using FM27 Assistant's plugin instead
If FM27 Assistant is installed, its FM27Bridge (port 27027) speaks the same protocol. The app tries 27029 first, then 27027,
so no extra install is needed. Settings › Integrations shows which one answered.
