# AGENTS.md

- Before using `ui-sdk`, follow the `ui-sdk` skill if it's installed, and read https://ui-sdk.dev/web.md.
- Edit `src/` only; never edit `app/index.html` by hand. Rebuild with `node build.mjs`, then `npm test`.
- New UI text needs both languages (`L(pt, en)`); `tests/ui_i18n.py` and `ui_lang_audit.py` fail on leaks.
- Release versions must match in `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `src/ui/state.js`; CI fails otherwise.
- Bridge: read `docs/HANDOFF-bridge.md` and `bridge/README.md` before touching `src/ui/fmbridge.js` or `Plugin.cs`. All FM ids live in `FMB_IDS`. After changing `Plugin.cs`, copy the built DLL to `bridge/dist/` and bump `Plugin.Version` and `PLUGIN_VERSION` in `src-tauri/src/main.rs`.
- Never write to the user's saves or a real game without asking; Rust compiles only on CI (no local toolchain).
- Keep `README.md` (and `bridge/README.md` for bridge changes) updated in the same change as the code.
