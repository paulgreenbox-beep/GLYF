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

## Stabilization branch, 2026-10-27

Base commit `9fe510722e1706a0c2236bbdd06698f3601eb213`; subsequent
changes in `fix/desktop-stabilization` were checked on Windows x86_64 with
Rust/Cargo 1.97.1. This is a working-tree result, not a claim about the
published base commit or an uncreated release commit.

| Check | Result |
| --- | --- |
| `node scripts/check-editor-modules.mjs` | Passed |
| `node --test scripts/check-gui-backend.test.mjs` | Passed: 13 |
| `cargo check --offline --locked -p glyf-editor -j 1` | Passed |
| `cargo test --workspace --offline --locked -j 1` | Passed: 62 tests |
| `cargo build --workspace --offline --locked -j 1` | Passed |
| CLI sample validate/list/extract and SHA-256 | Passed; CLI source unchanged from base |
| Fresh public base clone check/test/build | Passed, using the local Cargo cache |
| Windows native dialog, physical GUI and fresh restart | **NOT RUN** |

The file-dialog result and filters have unit coverage; compilation does not
demonstrate modal ownership, focus, Unicode handling, permissions, cancellation
or DPI behavior in a real window. The selected canary connection answered an
evidence request but its current exposed catalog lacks `windows_find_window`
and no operator admission was configured; GUI acceptance is
**BLOCKED_ENVIRONMENT**. No synthetic screenshot or direct-model action is
substituted for physical input. See `reports/FIX_REPORT.md` for the remaining
risks and local ignored `reports/evidence/stabilization/` for receipts.
