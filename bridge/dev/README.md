# Bridge dev scripts (manual tests against a running FM27)

Scratch scripts used to decode and verify set pieces. They WRITE to the running game: use a save you won't keep.
All talk to the plugin on 127.0.0.1:27029 through `../fmb-node.mjs`. The pairs/ids inside are from one save (manager,
team): adjust before use.

- `_sp.mjs run` — read routine kind / tab, switch the tab through FM's channel (`WRITEUI`).
- `_slot.mjs <group> <slot> <option> <label> [kind]` — set one slot instruction (`1633039705`), then print the routine.
- `_move.mjs <srcGroup> <srcSlot> group <dstGroup>` / `… slot <dstGroup> <dstSlot>` — move an instruction (`1449477207`).
- `_def.mjs <1|2> [opts]` — read the attacking (1) / defending (2) routine. Needs `DATAHOOK 1`; data slot `7600:4` is
  session-specific (find it with `DATA`).
