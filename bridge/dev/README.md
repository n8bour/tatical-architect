# Bridge dev scripts (manual tests against a running FM27)

Scratch scripts used to decode and verify the bridge. They talk to the plugin on 127.0.0.1:27029 through `../fmb-node.mjs`, which runs
the app's real `src/ui/fmbridge.js` (`FMB`) from Node (`FMB_CORE=1` also loads the app's core files, so `evalIn('TEMPLATES')` etc. work).
Most of them WRITE to the running game: use a save you won't keep. Ids inside (slot numbers, session) belong to one session: adjust.
The plugin needs `SEQ 1` (+ `DATAHOOK 1` for screen text) for the ones that read `bridge_seq.txt`; switch `SEQ 0` off afterwards.

Sets pieces
- `_sp.mjs run` · `_slot.mjs <group> <slot> <option> <label> [kind]` · `_move.mjs …` · `_def.mjs <1|2>` · `_routine.mjs` (end-to-end routine apply + restore).

Tactic
- `_applytactic.mjs [templateId] [--new]` applies an app template to the active tactic (`--new` creates one first).
- `_applypi.mjs` same with player instructions on some slots. `_formation.mjs`, `_role.mjs`, `_tiset.mjs` set one thing each.
- `_tactic.mjs` dumps tactic props · `_tsnap.mjs save|diff` snapshots every prop (set `TSNAP_AT=1` to read via the screen context).
- `_tienum.mjs` / `_pienum.mjs` / `_pidbg.mjs` step team / player instruction categories and read the labels FM's screen shows.

Recruitment and opponent
- `_search.mjs "<name>" [unique_id]` · `_recruit.mjs` (find person, shortlist, create + delete a focus) · `_focus*.mjs` (focus fields) · `_fixture.mjs` (next fixture).
