# Stabilization fix report

Base: `9fe510722e1706a0c2236bbdd06698f3601eb213` on
`fix/desktop-stabilization`. Windows x86_64-pc-windows-msvc. This report
distinguishes source changes from runtime acceptance.

| ID | Files / functions | Priority | Finding and change | Verification / remaining risk |
| --- | --- | --- | --- | --- |
| A1 | `apps/glyf-editor/src/file_dialog.rs:29,56,59,103`; `project_io.rs:6` | P1 | Replace blocking PowerShell process with the owner-bound Windows Common Item Dialog; distinguish selected path, cancellation, and failure; preserve localized names and filters. | `cargo check` and dialog unit tests passed. GUI dialog, focus, DPI, Unicode paths and failure cases are **NOT RUN**; no GUI acceptance claimed. Other platforms return an explicit unsupported error. Owner lookup uses the existing title/PID adapter; multiple running Editor instances remain unverified. |
| A2 | `verification/gui-backend.json`; Toolkit canary | P1 | Public GUI edit/save/reopen must use the admitted current route. | **BLOCKED_ENVIRONMENT**: current tool catalog does not expose `windows_find_window`; no approved operator admission is configured. `windows_get_evidence` answered, proving only connection. |
| A3 | `apps/glyf-editor/src/desktop.rs:412,453,1211,1272`; `settings.rs:572` | P2 | Persist the full `StyleProfile` as authoritative; project the legacy `ThemeChoice` for compatibility, migrate legacy-only settings. | Dark, light, Neon, custom and legacy migration tests passed. Real restart and UI inspection **NOT RUN**. Legacy System maps to the existing production dark appearance on migration. |
| A4 | `apps/glyf-editor/src/overlay_host.rs:35,59`; `shell.rs` | P2 | Existing menu/overlay implementation retained; no physical defect reproduced in this iteration. | Physical menu, outside click, Escape, submenu placement and focus **NOT RUN**; this is an unverified risk, not a fix. |
| A5/C1 | `crates/glyf-core/tests/manifest_round_trip.rs:12,36,65,96`; `crates/glyf-pack/tests/container_round_trip.rs:46`; `.github/workflows/ci.yml` | P2 | Add manifest round trips and exact GLYP resource bytes; Windows CI validates sample and compares extracted image SHA-256. No format grammar changes. | New round-trip tests passed as part of 62 workspace tests; sample validate/list/extract and SHA-256 comparison passed locally. Modified GitHub CI is **NOT RUN**. |

Fresh public `main` clones were checked, tested and built separately against
their recorded base commits; see local stabilization evidence. The working
branches are not those published commits. Physical GUI acceptance remains
blocked, and screenshots are not synthesized.
