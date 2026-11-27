# AGENTS.md

- Before using `ui-sdk`, follow the `ui-sdk` skill if it's installed, and read https://ui-sdk.dev/web.md.
- Desktop app (native Rust + GPUI): workspace crates live in `crates/core` (`ta-core`), `crates/backend` (`ta-backend`), and `crates/app` (`ta-app`), pinned via `rust-toolchain.toml`. Build and test locally with `cargo test --workspace`, `cargo run -p ta-app`, and `cargo build --release -p ta-app`.
- Single-file browser HTML build: edit `src/` only; never edit `app/index.html` by hand. Rebuild with `node build.mjs`, then `npm test`.
- New UI text needs both languages (EN + PT via `ta_core::data::i18n::l` in Rust and `L(pt, en)` in JS); parity i18n tests and `tests/ui_i18n.py` / `ui_lang_audit.py` fail on leaks.
- Release versions must match in `Cargo.toml` (`[workspace.package] version`), `package.json`, and `src/ui/state.js`; CI fails otherwise.
- Bridge: read `bridge/README.md` (decoded ids, rules, open gaps) before touching `crates/core/src/live/fmbridge.rs`, `src/ui/fmbridge.js`, or `Plugin.cs`. All FM ids live in `FmbIds` / `FMB_IDS`. Keep `TacticalBibleBridge.dll` and the `127.0.0.1:27029` protocol compatible with FM27/BepInEx. After changing `Plugin.cs`, copy the built DLL to `bridge/dist/` and bump `Plugin.Version` and `PLUGIN_VERSION` in `crates/backend/src/plugin.rs`.
- Never write to the user's saves or a real game without asking.
- Keep `README.md` (and `bridge/README.md` for bridge changes) updated in the same change as the code.
