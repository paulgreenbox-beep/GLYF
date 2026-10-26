// Read-only consumer preflight. Does not launch a process, contact an endpoint,
// change Codex configuration, register a target or replace Toolkit admission.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath, pathToFileURL } from 'node:url';

export function evaluateBackend(policy, capabilities, remoteSourceExists, profileVerified = false, scope = 'integration') {
  if (policy?.schema !== 'glyf-gui-development-v2') throw Error('Unknown GUI policy schema');
  if (policy.backend !== 'toolkit-gui-remote-canary' || policy.legacy_fallback !== false) {
    throw Error('Only toolkit-gui-remote-canary without legacy fallback is permitted');
  }
  if (!Array.isArray(policy.required_observation_tools) || !policy.required_observation_tools.includes('windows_find_window')) {
    throw Error('A public owned-target discovery gate is required');
  }
  if (!['windows-uia', 'makepad-remote'].includes(policy.route)) throw Error('An explicit supported route is required');
  if (!['inspection', 'integration'].includes(scope)) throw Error('Unknown verification scope');
  if (!Array.isArray(policy.required_input_tools) || !policy.required_input_tools.includes('windows_click_automation_id') || !policy.required_input_tools.includes('windows_set_value_to_automation_id')) throw Error('UI interaction gates must remain explicit');
  const blockers = [];
  if (!capabilities || capabilities.backend !== policy.backend || !Array.isArray(capabilities.tools)) {
    blockers.push({ code: 'CAPABILITIES_NOT_OBSERVED', message: 'Supply a fresh tool catalog from this canary connection' });
  } else {
    const required = [...policy.required_observation_tools, ...(scope === 'integration' ? policy.required_input_tools : [])];
    for (const name of required) {
      if (!capabilities.tools.includes(name)) blockers.push({ code: 'TOOL_NOT_EXPOSED', tool: name });
    }
    if (capabilities.connection_probe?.ok !== true) {
      blockers.push({ code: 'CANARY_CONNECTION_NOT_PROBED' });
    }
  }
  if (policy.route === 'makepad-remote' && !remoteSourceExists) blockers.push({ code: 'MAKEPAD_REMOTE_RUNTIME_ABSENT' });
  // These are static consumer preconditions, never the Toolkit's runtime/Job,
  // resource admission, current process birth, lease or GUI acceptance proof.
  if (!policy.operator_admission || !profileVerified) blockers.push({ code: 'OWNED_GLYF_PROFILE_NOT_VERIFIED' });
  return {
    backend: policy.backend,
    route: policy.route,
    scope,
    status: blockers.length ? 'BLOCKED_ENVIRONMENT' : 'READY_FOR_OPERATOR',
    blockers,
    legacy_fallback_used: false,
    gui_executed: false,
    gui_accepted: false,
    requires_fresh_toolkit_runtime_admission: true,
    note: 'READY_FOR_OPERATOR is only a static precheck, not permission to bypass the existing Toolkit operator or a GUI/Core acceptance claim'
  };
}

export function verifyProfilePointer(root, pointer) {
  if (!pointer || typeof pointer.profile_path !== 'string' || !/^[a-f0-9]{64}$/i.test(pointer.sha256 ?? '')) return false;
  const file = path.resolve(root, pointer.profile_path);
  if (!file.startsWith(path.resolve(root) + path.sep)) throw Error('Operator profile must remain inside this consumer root');
  if (!fs.existsSync(file) || fs.lstatSync(file).isSymbolicLink()) return false;
  const bytes = fs.readFileSync(file);
  if (crypto.createHash('sha256').update(bytes).digest('hex').toUpperCase() !== pointer.sha256.toUpperCase()) return false;
  const p = JSON.parse(bytes);
  return p.schema === 'glyf-source-bound-uia-operator-profile-v1' && p.backend === 'EXPLICIT_WINDOWS_UIA'
    && p.silent_remote_fallback === false && p.arbitrary_foreign_windows === false
    && [p.app, p.server, p.codex].every(identity => identity && path.isAbsolute(identity.path) && /^[a-f0-9]{64}$/i.test(identity.sha256 ?? ''));
}

export function inspectBackend(root, capabilitiesFile, scope = 'integration') {
  const policy = JSON.parse(fs.readFileSync(path.join(root, 'verification/gui-backend.json'), 'utf8'));
  const capabilities = capabilitiesFile ? JSON.parse(fs.readFileSync(capabilitiesFile, 'utf8')) : null;
  const remoteSource = path.resolve(root, policy.runtime_remote_source);
  return evaluateBackend(policy, capabilities, fs.existsSync(remoteSource), verifyProfilePointer(root, policy.operator_admission), scope);
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  try {
    const root = fileURLToPath(new URL('../', import.meta.url));
    const args = process.argv.slice(2);
    let catalog, scope = 'integration';
    for (let i = 0; i < args.length; i += 2) {
      if (!args[i + 1]) throw Error('Missing preflight argument');
      if (args[i] === '--capabilities') catalog = path.resolve(args[i + 1]);
      else if (args[i] === '--scope') scope = args[i + 1];
      else throw Error('Usage: node scripts/check-gui-backend.mjs [--capabilities <fresh-catalog.json>] [--scope inspection|integration]');
    }
    const result = inspectBackend(root, catalog, scope);
    console.log(JSON.stringify(result, null, 2));
    process.exitCode = result.status === 'READY_FOR_OPERATOR' ? 0 : 2;
  } catch (error) {
    console.error(JSON.stringify({ status: 'FAIL_TEST_INFRA', message: error.message }));
    process.exitCode = 1;
  }
}
