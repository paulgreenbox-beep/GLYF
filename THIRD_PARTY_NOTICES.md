# Third-party notices

The root MIT License covers this project's own code, documentation and synthetic
examples. It does not relicense dependencies, fonts, icons or other upstream work.

## Makepad

Makepad is retrieved from its public Git repository at revision
`105158e08c89d47f57af4621420ed68eb3a147ce`. Its package declares
`MIT OR Apache-2.0`; preserve the upstream license and copyright notices when
redistributing its code. Nested libraries and font/image assets may carry
additional terms. They are not copied into this source repository.

## Designing Makepad

The independently maintained Designing Makepad crate is obtained from its
public repository. It retains its own MIT License and third-party notices.
It is not copied into this repository or made part of this workspace.

## Toolbar icons

`apps/glyf-editor/resources/icons/toolbar/` contains PNG adaptations of a
curated Lucide subset. Lucide is ISC licensed; Feather-derived icons also
retain Cole Bemis's MIT notice. The notices are kept beside these
assets in `LICENSE-LUCIDE.txt`. These assets are not claimed as original GLYF
artwork. The small `actual-size` icon is a project drawing.

## Optional Toolkit integration

Rust Makepad Toolkit is separate developer tooling. Only this application's
adapter source is included here; no Toolkit runtime, third-party tool bundle,
private operator profile, user configuration or automation evidence is shipped.
Its availability and license must be checked separately when installing it.

## Dependencies and distributions

Cargo resolves the versions in `Cargo.lock`. Their package metadata and source
license notices remain authoritative; the root MIT License is not a blanket
license for a compiled distribution. Unicode data packages,
Apache-licensed dependencies and upstream fonts require their matching notices
when redistributed. This publication ships source, not a binary/font bundle.

Local archives, unrelated experiments, unverified borrowed media, user documents,
screenshots and captured automation reports are excluded from publication.
