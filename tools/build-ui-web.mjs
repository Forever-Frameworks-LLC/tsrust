// Bounded Wasm counter backend: the checked entry is unchanged from native.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const [entry, output] = process.argv.slice(2);
if (!entry || !output || !process.env.TSRUST || !process.env.WASM_BINDGEN || !process.env.CARGO_TARGET_DIR) {
  throw new Error('Usage: TSRUST=... WASM_BINDGEN=... CARGO_TARGET_DIR=... node tools/build-ui-web.mjs entry.tsr new-output-directory');
}
let projectRoot = path.dirname(fs.realpathSync(entry));
while (!fs.existsSync(path.join(projectRoot, 'tsrust.config.json'))) {
  const parent = path.dirname(projectRoot);
  if (parent === projectRoot) throw new Error('Web UI needs tsrust.config.json ui.target.web=true. Run tsrust ui add tokens@1.0.0 --target web.');
  projectRoot = parent;
}
const projectConfig = JSON.parse(fs.readFileSync(path.join(projectRoot, 'tsrust.config.json'), 'utf8'));
if (projectConfig.ui?.target?.web !== true) throw new Error('Web target disabled. Set tsrust.config.json ui.target.web=true or run tsrust ui add tokens@1.0.0 --target web.');
const destination = path.resolve(output);
if (fs.existsSync(destination)) throw new Error('Output already exists; choose a new directory.');
const scratch = fs.mkdtempSync(path.join(os.tmpdir(), 'tsrust-ui-web-'));
function run(command, args, cwd) {
  const result = spawnSync(command, args, {cwd, stdio: 'inherit', timeout: 300000, env: {...process.env, CARGO_INCREMENTAL: '0', CARGO_BUILD_JOBS: '1'}});
  if (result.error || result.status !== 0) throw result.error || new Error(`${command} failed (${result.status})`);
}
try {
  const pkg = path.join(scratch, 'package');
  if (projectConfig.documentation) run(process.env.TSRUST, ['docs', 'content', '--cwd', projectRoot]);
  run(process.env.TSRUST, ['emit', path.resolve(entry), '--package', '-o', pkg]);
  const source = path.join(pkg, 'src/main.rs');
  const rust = fs.readFileSync(source, 'utf8');
  const modules = [...rust.matchAll(/mod (tsrust_native_\w+)\s*\{/g)].map(match => match[1]);
  if (!modules.includes('tsrust_native_ui') || modules.some(name => !['tsrust_native_ui', 'tsrust_native_json'].includes(name))) {
    throw new Error('This bounded browser backend supports UI and JSON data; native service, file and task runtimes are unsupported.');
  }
  fs.appendFileSync(source, '\n#[wasm_bindgen::prelude::wasm_bindgen]\npub fn launch()->Result<js_sys::Array,wasm_bindgen::JsValue>{tsrust_native_ui::browser::launch(||main()==std::process::ExitCode::SUCCESS)}\n');
  const manifest = path.join(pkg, 'Cargo.toml');
  fs.writeFileSync(manifest, fs.readFileSync(manifest, 'utf8').replace(/^default = .*$/m, 'default = ["ui-web"]') + '\n[lib]\npath = "src/main.rs"\ncrate-type = ["cdylib"]\n');
  run('cargo', ['build', '--offline', '--locked', '--release', '--target', 'wasm32-unknown-unknown', '--lib'], pkg);
  fs.mkdirSync(destination);
  if (fs.existsSync(path.join(pkg,'attributions'))) fs.cpSync(path.join(pkg,'attributions'),path.join(destination,'attributions'),{recursive:true});
  run(process.env.WASM_BINDGEN, [path.join(process.env.CARGO_TARGET_DIR, 'wasm32-unknown-unknown/release/tsrust_export.wasm'), '--target', 'web', '--out-dir', destination, '--out-name', 'app', '--no-typescript']);
  fs.writeFileSync(path.join(destination, 'index.html'), `<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Tsrust UI</title><style>html,body{margin:0;padding:0}</style></head><body><script type="module">
import init, {launch} from './app.js';
await init();
const mounts = launch();
window.ui = {mounts, dispose() {for (const mount of mounts) mount.free(); mounts.length = 0;}};
addEventListener('pagehide', () => window.ui.dispose(), {once:true});
</script></body></html>`);
  const files = ['app.js', 'app_bg.wasm'].map(name => {const bytes=fs.readFileSync(path.join(destination,name));return {name,bytes:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex')};});
  console.log(JSON.stringify({backend:'bounded-ui-wasm',files,timed:false}));
} finally { fs.rmSync(scratch, {recursive:true,force:true}); }
