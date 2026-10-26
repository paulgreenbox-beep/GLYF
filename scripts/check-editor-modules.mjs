// Cheap structural regression check; deliberately no Cargo, GUI or network.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const src = path.join(root, 'apps/glyf-editor/src');
const read = name => fs.readFileSync(path.join(src, name), 'utf8').replaceAll('\r\n', '\n');
const owners = ['documents', 'project_io', 'history', 'settings', 'input', 'canvas', 'source_clip', 'parts', 'annotations', 'shell'];
const main = read('main.rs');
assert.ok(main.split('\n').length <= 240, 'Keep the entry point small');
assert.ok(!main.includes('impl App {') && !main.includes('script_mod!'), 'Behavior/UI must live in their owners');
for (const owner of [...owners, 'model', 'helpers', 'events', 'ui']) assert.ok(main.includes(`mod ${owner};`));

const methods = new Map();
for (const file of [...owners.map(name => `${name}.rs`), 'accessibility/mod.rs', 'accessibility/actions.rs', 'accessibility/tree.rs']) {
  for (const match of read(file).matchAll(/^    pub\(crate\) fn (\w+)\(/gm)) {
    assert.ok(!methods.has(match[1]), `Duplicate method ${match[1]} in ${file}`);
    methods.set(match[1], file);
  }
}
assert.equal(methods.get('save_project'), 'project_io.rs');
assert.equal(methods.get('record_history'), 'history.rs');
assert.equal(methods.get('sync_canvas_view'), 'canvas.rs');
assert.equal(methods.get('refresh_accessibility'), 'accessibility/mod.rs');
assert.equal(methods.get('accessibility_tree'), 'accessibility/tree.rs');
assert.equal(methods.get('handle_accessibility_action'), 'accessibility/actions.rs');
const productionModel = read('model.rs').split('#[cfg(test)]')[0];
assert.ok(!/toolkit_windows_automation|windows_uia|WidgetRef|std::fs|super::\*/.test(productionModel), 'Document state must not depend on UI/automation/IO');
assert.ok(read('events.rs').includes('crate::ui::script_mod(vm)'), 'Register the extracted UI, not another script module');
assert.equal((read('ui.rs').match(/startup\(\) do #\(App::script_component\(vm\)\)/g) ?? []).length, 1);
const policy = JSON.parse(fs.readFileSync(path.join(root, 'verification/gui-backend.json'), 'utf8'));
assert.equal(policy.backend, 'toolkit-gui-remote-canary');
assert.equal(policy.legacy_fallback, false);
console.log(JSON.stringify({ status: 'passed', entry_point_lines: main.split('\n').length, adapter_methods: methods.size, gui_backend: policy.backend }));
