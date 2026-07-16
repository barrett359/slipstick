# Lidar & Point Defense Integration Handoff

## Goal

Integrate the July 16, 2026 Lidar Countermeasure and Point-Defense v1 specification as a new Slipstick page, using the existing Rust `/api/calc/*` and framework-free frontend architecture. Every input and displayed output requires a plain-English accessible tooltip.

## Locked decisions

- New top-level tab: `Lidar & PD`.
- Scenarios are transient UI state; import/export JSON is the persistence mechanism.
- Backend contract: `POST /api/calc/lidar_pd`, SI-only versioned request, 1 MiB limit.
- Installed design lasers and current System Map source/target geometry may seed copied snapshot inputs.
- Missing speckle inputs live under `detector`; default diversity counts are 1, normal single-mode sigma is 1 nrad, dark-rough preset is 5 nrad.
- Tooltips cover every editable field and every displayed output via one centralized metadata map, with keyboard and screen-reader support.

## Current repository state

- Existing app is a single Axum binary with physics in `src/physics.rs`, routes in `src/main.rs`, and embedded frontend in `static/app.js` / `static/style.css`.
- Existing calculator routes use `/api/calc/*` and return `422 {"error":"..."}` for physics validation failures.
- Worktree was clean except for untracked `target/` before this handoff file was created.

## Progress

- [x] Repository and specification inspected.
- [x] Rust input/output types, validation, physics, and initial tests.
- [x] API route, 1 MiB body limit, and UUID response boundary.
- [x] Lidar & PD page, presets, import/export, connected input seeding.
- [x] Central tooltip metadata and accessible tooltip behavior.
- [x] Styling and README documentation.
- [x] Formatting, lint, Rust/API tests, JS syntax, tooltip coverage, and live HTTP verification.
- [ ] Interactive browser verification: blocked because the in-app browser runtime could not attach to this WSL-backed workspace (`sandboxCwd is not a local file URI`).

## Final verification (2026-07-16)

- `cargo test`: 46 passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo fmt -- --check`: passed.
- `git diff --check`: passed.
- Bundled Node `--check static/app.js`: passed.
- Tooltip coverage: 173 scenario and audit value paths covered.
- Live `POST /api/calc/lidar_pd`: HTTP 200 in 0.000873 s for the baseline request.
- Baseline response: UUID `8931ee21-15a1-4098-b134-3c9c1a6f5ede`, 20,224.7 target photons, SNR 3.0442, detector state `saturated` because pre-processing jammer counts exceed full well.

## Verification target

Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, JavaScript syntax checks, a live `curl` request to `/api/calc/lidar_pd`, and an interactive browser pass.
