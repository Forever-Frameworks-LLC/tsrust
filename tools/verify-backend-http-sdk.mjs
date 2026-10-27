// Exercise the actual CLI-created backend; no frontend toolchain or editor control.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import net from 'node:net';
import http from 'node:http';
import { spawn, spawnSync, execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { pathToFileURL } from 'node:url';

const root = path.resolve(process.argv[2] || 'generated-projects/backend-http');
const compiler = path.resolve(process.env.TSRUST || 'target/debug/tsrust.exe');
const config = JSON.parse(fs.readFileSync(path.join(root, 'tsrust.config.json')));
const executable = path.resolve(root, config.build.output);
const base = 'http://127.0.0.1:4318';
// Never connect our test to an owner's already-running service.
await new Promise((resolve, reject) => {
  const probe = net.createServer();
  probe.once('error', reject);
  probe.listen(4318, '127.0.0.1', () => probe.close(resolve));
});
const child = spawn(executable, [], { cwd: root, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
let output = '', errors = '';
child.stdout.on('data', value => { output += value; });
child.stderr.on('data', value => { errors += value; });
const exited = new Promise(resolve => { child.once('error', error => resolve({ error })); child.once('exit', (code, signal) => resolve({ code, signal })); });
const temporary = fs.mkdtempSync(path.resolve('target/backend-sdk-http-'));
const execute = promisify(execFile);
const run = (file, args, cwd = temporary, extra = {}) => execute(file, args, { cwd, windowsHide:true, timeout:180000, maxBuffer:8 * 1024 * 1024, env:{ ...process.env, CARGO_INCREMENTAL:'0', CARGO_BUILD_JOBS:'1', CARGO_TARGET_DIR:path.resolve('target'), ...extra } });
let metadataServer;
try {
  const deadline = Date.now() + 15000;
  while (!output.includes('API ready')) {
    assert.equal(child.exitCode, null, `Backend exited: ${errors}`);
    assert.ok(Date.now() < deadline, `Backend readiness timeout: ${errors}`);
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  const response = await fetch(`${base}/api/description`, { signal: AbortSignal.timeout(5000) });
  assert.equal(response.status, 200);
  assert.equal(response.headers.get('cache-control'), 'no-store');
  const description = await response.text();
  assert.deepEqual(JSON.parse(description), JSON.parse(fs.readFileSync(path.join(root, 'generated/client/http-api.json'))));
  const specification = await fetch(`${base}/api/openapi.json`, { signal: AbortSignal.timeout(5000) });
  assert.equal(specification.status, 200);
  const openapi = await specification.json();
  assert.deepEqual(openapi, JSON.parse(fs.readFileSync(path.join(root, 'generated/client/openapi.json'))));
  const rpcEndpoint = new URL(Object.keys(openapi.paths)[0], base).href;
  fs.writeFileSync(path.join(temporary, 'api.json'), description);
  const generated = spawnSync(compiler, ['api', 'generate', '--description', 'api.json', '--out', 'sdk'], { cwd: temporary, encoding: 'utf8', windowsHide: true, timeout: 30000 });
  assert.ifError(generated.error);
  assert.equal(generated.status, 0, generated.stderr);
  const { createClient } = await import(pathToFileURL(path.join(temporary, 'sdk/index.js')));
  const { createHttpTransport } = await import(pathToFileURL(path.join(temporary, 'sdk/http.js')));
  const api = createClient(createHttpTransport({ url: rpcEndpoint }));
  assert.equal(await api.greet('  backend  '), 'Hello, backend!');
  assert.equal(await api.greet(''), 'Hello, world!');
  await assert.rejects(api.greet(42), error => error.code === 'api-validation');
  await run(compiler, ['api','generate','--url',`${base}/api/description`,'--out','live-sdk']);
  assert.equal(fs.readFileSync(path.join(temporary,'live-sdk/index.js'),'utf8'), fs.readFileSync(path.join(temporary,'sdk/index.js'),'utf8'));
  let redirected = 0, credentialCalls = 0;
  metadataServer = http.createServer(async (request, response) => {
    if (request.url === '/redirect') { response.writeHead(302,{location:'/would-follow'}); response.end(); return; }
    if (request.url === '/would-follow') redirected++;
    if (request.url === '/large') { response.writeHead(200,{'content-type':'application/json','content-length':1048577}); response.end(' '.repeat(1048577)); return; }
    if (request.headers.authorization !== 'Bearer fixture-access') { response.writeHead(403,{'content-type':'application/json'}); response.end('{}'); return; }
    credentialCalls++;
    response.setHeader('content-type','application/json');
    if (request.url === '/invoke') {
      let body='';for await (const chunk of request) body += chunk;
      const value=JSON.parse(body);assert.equal(value.operation,'greet');
      assert.match(request.headers['content-type'], /tsrust-api=1; tsrust-operation=[a-f0-9]{64}/);
      assert.match(request.headers['x-tsrust-contract'], /^[a-f0-9]{64}$/);
      response.end(JSON.stringify({ok:true,value:`Hello, ${value.input.trim()}!`}));return;
    }
    response.end(description);
  });
  await new Promise(resolve => metadataServer.listen(0,'127.0.0.1',resolve));
  const metadataBase=`http://127.0.0.1:${metadataServer.address().port}`;
  await assert.rejects(run(compiler,['api','generate','--url',`${metadataBase}/protected`,'--out','denied']));
  assert.ok(!fs.existsSync(path.join(temporary,'denied')));
  await run(compiler,['api','generate','--url',`${metadataBase}/protected`,'--header-env','Authorization=TSRUST_WITNESS_AUTH','--out','protected'],temporary,{TSRUST_WITNESS_AUTH:'Bearer fixture-access'});
  assert.equal(credentialCalls,1);
  for (const [endpoint, output] of [['redirect','redirected'],['large','oversized']]) {
    await assert.rejects(run(compiler,['api','generate','--url',`${metadataBase}/${endpoint}`,'--out',output]));
    assert.ok(!fs.existsSync(path.join(temporary,output)));
  }
  assert.equal(redirected,0);
  await assert.rejects(run(compiler,['api','generate','--url','http://example.com/description','--out','insecure']));
  await run(compiler,['api','generate','--url',`${base}/api/description`,'--language','rust','--out','rust-sdk']);
  const rust=path.join(temporary,'rust-sdk');
  await run(compiler,['api','generate','--description','api.json','--language','rust','--out','rust-saved']);
  assert.equal(fs.readFileSync(path.join(rust,'src/lib.rs'),'utf8'),fs.readFileSync(path.join(temporary,'rust-saved/src/lib.rs'),'utf8'));
  const consumer=path.join(temporary,'rust-consumer');
  fs.mkdirSync(path.join(consumer,'src'),{recursive:true});
  fs.writeFileSync(path.join(consumer,'Cargo.toml'),'[package]\nname="tsrust-http-consumer"\nversion="0.1.0"\nedition="2024"\n[workspace]\n[dependencies]\napi={package="tsrust-app-api",path="../rust-sdk"}\n');
  fs.writeFileSync(path.join(consumer,'src/main.rs'), `use api::{Client,HttpTransport,HeaderMap,HeaderValue};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let api=Client::new(HttpTransport::new(${JSON.stringify(rpcEndpoint)})?);
 assert_eq!(api.api().greet("  Rust  ".into())?,"Hello, Rust!");
 assert_eq!(api.api().greet("".into())?,"Hello, world!");
 let transport=HttpTransport::new(${JSON.stringify(metadataBase+'/invoke')})?.with_header_provider(|| {
   let mut headers=HeaderMap::new();headers.insert("authorization",HeaderValue::from_static("Bearer fixture-access"));Ok(headers)
 });
 let protected=Client::new(transport);assert_eq!(protected.api().greet("protected".into())?,"Hello, protected!");
 println!("Rust live HTTP named calls passed");Ok(())
}`);
  await run('rustup',['run','1.96.1','cargo','generate-lockfile','--offline'],consumer);
  const rustRun=await run('rustup',['run','1.96.1','cargo','run','--offline','--locked','--bin','tsrust-http-consumer'],consumer);
  assert.match(rustRun.stdout,/Rust live HTTP named calls passed/);
  assert.equal(credentialCalls,2);
  fs.mkdirSync(path.join(consumer,'src/bin'));
  fs.writeFileSync(path.join(consumer,'src/bin/reject.rs'), `fn main() {
    let api=api::Client::new(api::HttpTransport::new("http://127.0.0.1:1/api").unwrap());
    let _=api.api().greet(42);
    let _:i64=api.api().greet(String::new()).unwrap();
  }`);
  await assert.rejects(run('rustup',['run','1.96.1','cargo','check','--offline','--locked','--bin','reject'],consumer), error => /E0308/.test(error.stderr));
  console.log('Fresh backend: live/saved TS and Rust SDK generation, real native named calls, auth header providers, redirect/size/insecure-URL rejection, OpenAPI and input validation passed.');
} finally {
  if (metadataServer) { metadataServer.closeAllConnections(); await new Promise(resolve => metadataServer.close(resolve)); }
  if (child.exitCode === null) child.kill();
  let timer;
  try { await Promise.race([exited, new Promise((_, reject) => { timer = setTimeout(() => reject(Error('Owned backend did not exit')), 5000); })]); }
  finally { clearTimeout(timer); }
  assert.equal(path.dirname(fs.realpathSync(temporary)), fs.realpathSync('target'));
  assert.ok(path.basename(temporary).startsWith('backend-sdk-http-'));
  fs.rmSync(temporary, { recursive: true, force: true });
}
