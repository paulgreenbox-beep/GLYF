import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { evaluateBackend, verifyProfilePointer } from './check-gui-backend.mjs';

const policy = JSON.parse(fs.readFileSync(new URL('../verification/gui-backend.json', import.meta.url), 'utf8'));
const readTools = policy.required_observation_tools.filter(name => name !== 'windows_find_window');

// These profile documents and paths are synthetic UNIT fixtures in OS temp
// directories, not operator admission or runtime evidence.
function tempRoot(t) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'glyf-gui-profile-unit-'));
  t.after(() => {
    assert.ok(path.resolve(root).startsWith(path.resolve(os.tmpdir()) + path.sep + 'glyf-gui-profile-unit-'));
    fs.rmSync(root, { recursive: true, force: true });
  });
  return root;
}

function profileDocument(root) {
  const identity = name => ({
    path: path.resolve(root, 'synthetic-bin', name),
    sha256: 'a'.repeat(64),
  });
  return {
    schema: 'glyf-source-bound-uia-operator-profile-v1',
    backend: 'EXPLICIT_WINDOWS_UIA',
    silent_remote_fallback: false,
    arbitrary_foreign_windows: false,
    app: identity('glyf-editor.exe'),
    server: identity('toolkit-gui-remote-canary.exe'),
    codex: identity('codex.exe'),
  };
}

function writeProfile(root, relative = 'operator-profile.json') {
  const file = path.join(root, relative);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, JSON.stringify(profileDocument(root), null, 2));
  const bytes = fs.readFileSync(file);
  return {
    file,
    sha256: crypto.createHash('sha256').update(bytes).digest('hex').toUpperCase(),
  };
}

test('configured canary with only read tools cannot manufacture a window ID or GUI PASS', () => {
  const result = evaluateBackend(policy, { backend: policy.backend, tools: readTools, connection_probe: { ok: true } }, false);
  assert.equal(result.status, 'BLOCKED_ENVIRONMENT');
  assert.ok(result.blockers.some(item => item.code === 'TOOL_NOT_EXPOSED' && item.tool === 'windows_find_window'));
  assert.ok(!result.blockers.some(item => item.code === 'MAKEPAD_REMOTE_RUNTIME_ABSENT'), 'Explicit UIA does not require the unrelated Remote runtime');
  assert.equal(result.gui_executed, false);
  assert.equal(result.legacy_fallback_used, false);
});

test('missing or another connection catalog remains unknown, not accepted', () => {
  for (const catalog of [null, { backend: 'legacy-toolkit', tools: readTools }]) {
    const result = evaluateBackend(policy, catalog, true);
    assert.ok(result.blockers.some(item => item.code === 'CAPABILITIES_NOT_OBSERVED'));
  }
});

test('tool presence and a source file do not certify operator profile or runtime acceptance', () => {
  const result = evaluateBackend(policy, { backend: policy.backend, tools: policy.required_observation_tools, connection_probe: { ok: true } }, true);
  assert.equal(result.status, 'BLOCKED_ENVIRONMENT');
  assert.ok(result.blockers.some(item => item.code === 'OWNED_GLYF_PROFILE_NOT_VERIFIED'));
});

test('legacy fallback and removal of discovery gate are rejected', () => {
  assert.throws(() => evaluateBackend({ ...policy, legacy_fallback: true }, null, true), /without legacy fallback/);
  assert.throws(() => evaluateBackend({ ...policy, required_observation_tools: readTools }, null, true), /discovery gate/);
});

test('catalog without successful same-connection probe is not live evidence', () => {
  const result = evaluateBackend(policy, { backend: policy.backend, tools: policy.required_observation_tools }, true);
  assert.ok(result.blockers.some(item => item.code === 'CANARY_CONNECTION_NOT_PROBED'));
});

test('inspection tools cannot authorize the input integration scope', () => {
  const p = { ...policy, operator_admission: { profile_path: 'synthetic-only.json', sha256: 'A'.repeat(64) } };
  const catalog = { backend: policy.backend, tools: policy.required_observation_tools, connection_probe: { ok: true } };
  assert.equal(evaluateBackend(p, catalog, false, true, 'inspection').status, 'READY_FOR_OPERATOR');
  assert.equal(evaluateBackend(p, catalog, false, true, 'integration').status, 'BLOCKED_ENVIRONMENT');
});

test('explicit Makepad Remote still requires its own runtime, not silent UIA fallback', () => {
  const result = evaluateBackend({ ...policy, route: 'makepad-remote' }, null, false);
  assert.ok(result.blockers.some(item => item.code === 'MAKEPAD_REMOTE_RUNTIME_ABSENT'));
  assert.equal(result.legacy_fallback_used, false);
});

test('successful static checks still cannot claim executed GUI or acceptance', () => {
  const p = { ...policy, operator_admission: { profile_path: 'synthetic-only.json', sha256: 'A'.repeat(64) } };
  const tools = [...policy.required_observation_tools, ...policy.required_input_tools];
  const result = evaluateBackend(p, { backend: policy.backend, tools, connection_probe: { ok: true } }, false, true);
  assert.equal(result.status, 'READY_FOR_OPERATOR');
  assert.equal(result.gui_accepted, false);
  assert.equal(result.gui_executed, false);
  assert.equal(result.requires_fresh_toolkit_runtime_admission, true);
});

test('verifyProfilePointer accepts the matching pinned profile inside the consumer root', t => {
  const root = tempRoot(t);
  const profile = writeProfile(root);
  assert.equal(verifyProfilePointer(root, {
    profile_path: path.relative(root, profile.file),
    sha256: profile.sha256,
  }), true);
});

test('verifyProfilePointer rejects a profile whose pinned hash does not match', t => {
  const root = tempRoot(t);
  const profile = writeProfile(root);
  assert.equal(verifyProfilePointer(root, {
    profile_path: path.relative(root, profile.file),
    sha256: profile.sha256 === '0'.repeat(64) ? '1'.repeat(64) : '0'.repeat(64),
  }), false);
});

test('verifyProfilePointer rejects a profile path outside the consumer root', t => {
  const base = tempRoot(t);
  const root = path.join(base, 'consumer');
  fs.mkdirSync(root);
  const profile = writeProfile(base, 'outside/operator-profile.json');
  assert.throws(() => verifyProfilePointer(root, {
    profile_path: path.relative(root, profile.file),
    sha256: profile.sha256,
  }), /inside this consumer root/);
});

test('verifyProfilePointer returns false for an absent profile', t => {
  const root = tempRoot(t);
  assert.equal(verifyProfilePointer(root, {
    profile_path: 'absent-profile.json',
    sha256: 'a'.repeat(64),
  }), false);
});

test('verifyProfilePointer rejects a profile path reported as a symlink by lstat', t => {
  const root = tempRoot(t);
  const profile = writeProfile(root);
  const realLstatSync = fs.lstatSync;
  // Inject symlink metadata because this environment denies creating OS symlinks.
  t.mock.method(fs, 'lstatSync', file => path.resolve(file) === path.resolve(profile.file)
    ? { isSymbolicLink: () => true }
    : realLstatSync(file));
  assert.equal(verifyProfilePointer(root, {
    profile_path: path.basename(profile.file),
    sha256: profile.sha256,
  }), false);
});
