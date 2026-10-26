# GLYF development

- GUI development and inspection use `toolkit-gui-remote-canary`, selected in
  `verification/gui-backend.json`. Do not silently use the old GUI MCP, a direct
  UIA script, private HTTP endpoints or synthetic screenshots as a fallback.
- Check current capabilities before any GUI scenario. A successful evidence
  query proves connection only, not Editor launch or Remote compatibility.
  Without public `windows_find_window` and an approved GLYF target/profile,
  record `BLOCKED_ENVIRONMENT`; do not invent
  a window ID or call read tools with another connection's identifiers.
  The selected route is explicit Windows UIA inside that canary, as reviewed
  with Makepad_Toolkit. Real ValuePattern/InvokePattern are not a direct-UIA
  script or a legacy MCP fallback. Makepad Remote runtime support is required
  only if a separately admitted `makepad-remote` route is selected.
  For a registered Remote target `windows_find_window` can launch the owned
  profile. For UIA use only the operator's exact-PID allowlist. The consumer
  preflight never discovers/launches an application or replaces Toolkit admission.
- The shared Rust `toolkit-windows-automation` bridge and Makepad path dependency
  are not the selected MCP server. Preserve them until a separately reviewed
  runtime/profile migration is approved. Do not edit sibling Toolkit/Makepad
  sources or global Codex configuration as a side effect of app changes.
- `main.rs` owns App state and composition only. Domain/UI adapters live in
  documents, project_io, history, settings, input, canvas, source_clip, parts,
  annotations and shell. Events, Splash UI and Windows accessibility/platform
  integration have separate modules. Keep format logic in glyf-core/glyf-pack.
- CHEAPEST_VERIFICATION_PATH: `node scripts/check-editor-modules.mjs`,
  `node --test scripts/check-gui-backend.test.mjs`, then focused
  `cargo check --offline --locked -p glyf-editor` and
  `cargo test --offline --locked -p glyf-editor`. GUI proof remains a separate
  required integration check; static/unit checks cannot replace it.
- Preserve existing dirty work, user documents, historical evidence and archives.
