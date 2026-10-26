# Publication verification

Checked on 2026-10-27 (Moscow date), Windows x86_64-pc-windows-msvc,
with rustc 1.97.1 and Cargo 1.97.1. The workspace used the committed public
Makepad and Designing Makepad Git dependencies, without a local path override
or Toolkit dependency.

| Command | Result |
| --- | --- |
| `node scripts/check-editor-modules.mjs` | Passed: 171 entry-point lines, 161 adapter methods |
| `node --test scripts/check-gui-backend.test.mjs` | Passed: 13 tests |
| `cargo check --workspace --all-targets --offline --locked -j1` | Passed |
| `cargo test --workspace --offline --locked -j1` | Passed: 51 tests across core, pack and editor; no failures |
| `cargo build --workspace --offline --locked -j1` | Passed |
| CLI `validate`, `info --json`, `list`, `extract` on `examples/basic` | Passed |

The example contains one image, one group, one part and two annotations.
The resource extracted from `basic.glyp` matches `sample.png` by SHA-256.
It is a synthetic format example, not a live Editor self-demo.

Makepad is pinned to `105158e08c89d47f57af4621420ed68eb3a147ce`.
Designing Makepad is pinned to `396865d7a6b20db85186da7becb1674e6432b9af`.
The Makepad checkout had no tracked source modifications. Cargo reports
duplicate upstream bitflags/cfg-if package warnings and unused editor
methods when the optional UIA bridge is disabled; these did not prevent
checking, testing or linking.

The optional local Toolkit integration host and live GUI interaction were
NOT RUN for this publication. Static backend checks, Rust tests and builds
do not establish pointer, focus, popup, DPI or graphics-driver acceptance.
Linux and macOS builds were not tested. No executable/font bundle or captured
user screenshots are published. Fresh-clone and GitHub Actions results are
recorded separately against the public commit after the initial push.
