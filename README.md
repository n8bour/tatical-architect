# FM Tactical Architect v15

Desktop-first tactical architect for Football Manager 27.

## Product goal

Read the squad, model player/role fit, generate multiple tactical candidates, score them decomposably, and explain every recommendation.

## Current status

- Fully implemented local tactical scoring engine in vanilla TypeScript/JavaScript.
- Tactic Builder UI with phase-aware roles and instruction diagnostics.
- Squad DNA analysis with explainable scores.
- Candidate generation for several formations.
- FMF file import shell that reads the native FMF container safely but does not invent proprietary TAC fields that are not decoded yet.
- Tauri 2 project skeleton for a standalone desktop app.
- Native FM27 memory reader is intentionally isolated as a future backend module.

## Important

This project does NOT claim to decode every proprietary FM27 `.tac` field yet. Unknown binary fields are surfaced as "not decoded" rather than guessed.

## Desktop build

The source is ready for Tauri 2. A Rust toolchain is required only on the build machine, not on the end user's PC.
