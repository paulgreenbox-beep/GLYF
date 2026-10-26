# Development

Normal builds use public Git dependencies and the committed lockfile. See
the root README for Windows prerequisites and commands. Do not modify a shared
Makepad checkout to make a publication build pass.

## Optional local Windows UIA host

The editor retains its semantic tree and action adapter in
`apps/glyf-editor/src/accessibility/`. Normal public binaries do not compile
that adapter and do not require Rust Makepad Toolkit.

`integrations/toolkit-uia/Cargo.toml.example` is a separate local-only host
manifest. When a compatible Toolkit checkout is available:

1. Copy the template to `integrations/toolkit-uia/Cargo.toml`.
2. Set its `toolkit-windows-automation` path to that checkout's crate.
3. Copy `apps/glyf-editor/resources/` to
   `integrations/toolkit-uia/resources/`, preserving the license notices.
4. Build with `cargo build --manifest-path integrations/toolkit-uia/Cargo.toml`.

The template defines `toolkit-uia` for that host only. Passing it
to the ordinary editor package is not supported. Generated host files are
ignored; do not commit local paths. This optional host is not part of publication
build verification.

Toolkit ABI/version compatibility, resource admission, exact owned process
identity and the selected canary profile must be reviewed independently. GUI
development and inspection use `toolkit-gui-remote-canary`; no legacy GUI MCP,
private HTTP endpoint, direct UIA script or synthetic screenshot fallback is
permitted. Cargo tests and historical screenshots are not a new GUI result.

## Publication exclusions

The public snapshot omits local reports/archives, heavy evidence, target
trees, user documents and personal settings. Original files remain on the
developer's computer. Existing local history is preserved separately from the
clean public snapshot so archived local paths do not enter public history.
