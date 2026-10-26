# GLYF

GLYF is a small visual-context format and a native desktop editor built with Rust and Makepad. It keeps images, descriptions, annotations and their spatial relationships together so that people and AI readers can understand the same visual document.

The editor does not call an AI service or infer meaning from pixels. You supply the semantic metadata; a separate reader can inspect the manifest and resources.

## Two files, one document

| File | Purpose |
| --- | --- |
| `.glyf` | UTF-8 text manifest: image names, source references, comments, annotation roles and geometry, groups and parts. |
| `.glyp` | Indexed binary resource container. Images can be read directly by name; this is a GLYF-specific format, not a ZIP archive. |

Keep the matching files together. Source images have `name`, `source` and `comment` metadata. An annotation has its own `name`, semantic `role` and `comment`. Parts reference source images by name.

## Editor features

- Import and preview images; edit names and comments.
- Add **Point annotations** with a normalized `rel` position.
- Draw **Region annotations** with normalized `rel1` and `rel2` corners.
- Use **Source Clip** to select a pixel rectangle from an image.
- Place and resize **Parts** on a composition canvas, organize them in groups, change their order, and attach comments or parent references.
- Pan and zoom, select canvas objects, and use undo/redo.
- Work with document tabs, menus, settings and English/Russian interface text.
- Save a manifest and its resources, then reopen the pair.

Spatial coordinates describe the composition rather than OCR results or a generated layout. A source image can be reused by several clipped parts.

## Build and run

Install Rust with the MSVC toolchain on Windows, Visual Studio C++ Build Tools and the Windows SDK. Git retrieves the pinned dependencies.

```powershell
git clone https://github.com/paulgreenbox-beep/GLYF.git
cd GLYF
cargo check --workspace --all-targets --locked
cargo test --workspace --locked
cargo build --workspace --locked
cargo run --locked -p glyf-editor -- examples/basic.glyf
```

The first build downloads dependencies. `Cargo.lock` is committed. Makepad is pinned to compatible upstream revision `105158e08c89d47f57af4621420ed68eb3a147ce`; the shared UI library is an independent [Designing Makepad repository](https://github.com/paulgreenbox-beep/designing-makepad). No checkout at the author's local filesystem paths is required.

The example is a synthetic format fixture, not a screenshot-based UI test report. See [verification results](VERIFICATION.md) for the checked commands and limits. Windows is the publication verification target; Linux and macOS desktop builds have not been verified for this release.

## CLI

The executable is named `glyf`:

```powershell
cargo run --locked -p glyf-cli -- new demo
cargo run --locked -p glyf-cli -- info examples/basic.glyf --json
cargo run --locked -p glyf-cli -- validate examples/basic.glyf
cargo run --locked -p glyf-cli -- list examples/basic.glyp
cargo run --locked -p glyf-cli -- extract examples/basic.glyp sample.png extracted.png
cargo run --locked -p glyf-cli -- pack image-only.glyp examples/sample.png
```

`new demo` creates `demo.glyf` and `demo.glyp`. `pack` creates a container from one image; it replaces its destination rather than appending to an existing document. Normal editor Save writes both the current manifest and resources.

## Workspace

| Package | Responsibility |
| --- | --- |
| `glyf-core` | Document model, text parser/serializer and validation. |
| `glyf-pack` | GLYP indexed binary container reader/writer. |
| `glyf-cli` | Creation, inspection, validation, packing and extraction commands. |
| `glyf-editor` | Makepad desktop editor. |

The editor's `main.rs` owns App state and composition. Documents/history, project IO, canvas, Source Clip, parts, annotations, settings, events, UI and platform adapters live in separate modules. Format logic stays in the core and pack crates. Designing Makepad remains a separate project.

## Optional Toolkit integration

Rust Makepad Toolkit is a separate development and UI automation tool. It is not required to build or use this public editor. Existing Windows semantic/UIA adapter sources are retained for a local integration host; they are excluded from the normal public binary. See [local development](docs/development.md).

GUI inspection must use an explicitly admitted `toolkit-gui-remote-canary` profile. A responding tool or successful Cargo command does not establish Editor GUI acceptance. This publication does not claim a new automated edit/save/reopen GUI run.

## Limits

- This is an MVP with format version 1, not a stable long-term interchange API.
- No OCR, automatic semantic analysis, AI service, collaboration or UI-code export is implemented.
- The editor and native file dialogs are primarily developed for Windows.
- The public binary does not include the optional Toolkit Windows UIA bridge.
- Rust tests and compilation do not cover every pointer, focus, popup, DPI or graphics-driver behavior; current verification scope is recorded separately.

## License

GLYF's own code and documentation are [MIT licensed](LICENSE), copyright 2026 paulgreenbox-beep. Third-party code and assets retain their own licenses. The included Lucide-derived toolbar assets retain ISC and Feather MIT notices; see [third-party notices](THIRD_PARTY_NOTICES.md).
