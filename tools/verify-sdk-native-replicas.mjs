// Cross-language boundary witness: generated SDKs call actual native replicas.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {spawn, execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {pathToFileURL} from 'node:url';

const root = path.resolve(import.meta.dirname, '..');
const compiler = path.resolve(process.env.TSRUST || path.join(root, 'target/debug/tsrust' + (process.platform === 'win32' ? '.exe' : '')));
const execute = promisify(execFile);

test('generated SDKs preserve auth and compatibility across native replicas', {timeout: 300000}, async () => {
  const temporary = fs.mkdtempSync(path.join(root, 'target/sdk-native-replicas-'));
  const services = [];
  const env = {...process.env, CARGO_INCREMENTAL: '0', CARGO_BUILD_JOBS: '1', CARGO_TARGET_DIR: path.join(root, 'target'), TSRUST_SDK_FIXTURE_AUTH: 'Bearer fixture-access'};
  const run = (file, args, cwd = temporary) => execute(file, args, {cwd, env, timeout: 180000, maxBuffer: 8 * 1024 * 1024, windowsHide: true});
  async function service(name, previous) {
    const project = path.join(temporary, name);
    await run(compiler, ['new', project, '--app']);
    const configPath = path.join(project, 'tsrust.config.json');
    const config = JSON.parse(fs.readFileSync(configPath));
    config.api.package.version = previous ? '0.2.0' : '0.1.0';
    if (previous) {
      fs.writeFileSync(path.join(project, 'compatibility.json'), JSON.stringify(previous.history));
      config.api.compatible = ['compatibility.json'];
      config.api.operations.push({name: 'preview', module: 'app/api/operations.tsr', export: 'preview', kind: 'query'});
    }
    fs.writeFileSync(configPath, JSON.stringify(config));
    fs.writeFileSync(path.join(project, 'app/api/operations.tsr'), `export function greet(name: string): string {
      console.log('effect:greet'); return '${name}:' + name;
    }
    export function preview(): string { console.log('effect:preview'); return 'preview'; }
    `);
    fs.writeFileSync(path.join(project, 'app/api/access.tsr'), `import {AppContext} from './context.tsr';
    export async function authorize(context: Mutable<AppContext>): Task<boolean> {
      return context.req.header('authorization') === 'Bearer fixture-access';
    }
    `);
    const mainPath = path.join(project, 'app/main.tsr');
    fs.writeFileSync(mainPath, fs.readFileSync(mainPath, 'utf8').replace('127.0.0.1:4318', '127.0.0.1:0'));
    await run(compiler, ['format', project, '--write']);
    await run(compiler, ['format', project, '--check']);
    await run(compiler, ['build'], project);
    const child = spawn(path.resolve(project, config.build.output), [], {cwd: temporary, env, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe']});
    const record = {child, stdout: '', stderr: ''};
    record.exited = new Promise(resolve => { child.once('error', error => resolve({error})); child.once('close', code => resolve({code})); });
    services.push(record);
    child.stdout.on('data', data => { record.stdout += data; });
    child.stderr.on('data', data => { record.stderr += data; });
    const deadline = Date.now() + 10000;
    while (!record.stdout.includes('API ready at ')) {
      assert.equal(child.exitCode, null, record.stderr);
      assert.ok(Date.now() < deadline, `Native service startup timed out: ${record.stderr}`);
      await new Promise(resolve => setTimeout(resolve, 25));
    }
    const endpoint = record.stdout.match(/API ready at (http:\/\/127\.0\.0\.1:\d+\/api)/)[1];
    const denied = await fetch(endpoint + '/description', {signal: AbortSignal.timeout(5000)});
    assert.equal(denied.status, 403);
    assert.equal((await denied.json()).error.code, 'api-denied');
    const response = await fetch(endpoint + '/description', {headers: {authorization: 'Bearer fixture-access'}, signal: AbortSignal.timeout(5000)});
    assert.equal(response.status, 200);
    return {endpoint, description: await response.json(), record};
  }
  try {
    const old = await service('old');
    const modern = await service('modern', old.description);
    const endpoints = [old.endpoint, modern.endpoint];
    for (const [name, replica] of [['old', old], ['modern', modern]]) {
      fs.writeFileSync(path.join(temporary, `${name}.json`), JSON.stringify(replica.description));
      await run(compiler, ['api', 'generate', '--description', `${name}.json`, '--out', `${name}-js`]);
      await run(compiler, ['api', 'generate', '--url', replica.endpoint + '/description', '--header-env', 'Authorization=TSRUST_SDK_FIXTURE_AUTH', '--language', 'rust', '--out', `${name}-rust`]);
    }
    for (const name of ['old', 'modern']) {
      const {createClient} = await import(pathToFileURL(path.join(temporary, `${name}-js/index.js`)));
      const {createHttpTransport} = await import(pathToFileURL(path.join(temporary, `${name}-js/http.js`)));
      for (const [index, endpoint] of endpoints.entries()) {
        const client = createClient(createHttpTransport({url: endpoint, headers: {authorization: 'Bearer fixture-access'}}));
        assert.equal(await client.greet('JS'), `${index ? 'modern' : 'old'}:JS`);
        await assert.rejects(client.greet(42), error => error.code === 'api-validation');
        const denied = createClient(createHttpTransport({url: endpoint, headers: {authorization: 'Bearer wrong'}}));
        await assert.rejects(denied.greet('denied'), error => error.code === 'api-denied');
        if (name === 'modern') {
          if (index) assert.equal(await client.preview(), 'preview');
          else await assert.rejects(client.preview(), error => error.code === 'api-operation');
        }
      }
    }
    const consumer = path.join(temporary, 'consumer');
    fs.mkdirSync(path.join(consumer, 'tests'), {recursive: true});
    fs.writeFileSync(path.join(consumer, 'Cargo.toml'), `[package]
name="native-replica-consumer"
version="0.1.0"
edition="2024"
[workspace]
[dependencies]
old={package="tsrust-app-api",path="../old-rust"}
modern={package="tsrust-app-api",path="../modern-rust"}
tokio={version="=1.53.3",features=["rt","time","net"]}
`);
    fs.writeFileSync(path.join(consumer, 'tests/replicas.rs'), `
use modern::{Client, HttpTransport, HeaderMap, HeaderValue, Transport, Request};
fn transport(url: &str, token: &'static str) -> HttpTransport {
  HttpTransport::new(url).unwrap().with_header_provider(move || async move {
    tokio::task::yield_now().await;
    let mut headers=HeaderMap::new();headers.insert("authorization",HeaderValue::from_static(token));Ok(headers)
  })
}
#[test]
fn real_native_replicas() { tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
  for (index, url) in [${endpoints.map(JSON.stringify).join(',')}].iter().enumerate() {
    let expected=if index==0 {"old:Rust"} else {"modern:Rust"};
    let api=Client::new(transport(url,"Bearer fixture-access"));
    assert_eq!(api.api().greet("Rust".into()).await.unwrap(),expected);
    let denied=Client::new(transport(url,"Bearer wrong"));
    assert_eq!(denied.api().greet("denied".into()).await.unwrap_err().code,"api-denied");
    let absent=Client::new(HttpTransport::new(url).unwrap());
    assert_eq!(absent.api().greet("denied".into()).await.unwrap_err().code,"api-denied");
    if index==0 { let error=api.api().preview().await.unwrap_err(); assert_eq!(error.code,"api-operation"); assert!(error.is_compatibility()); }
    else { assert_eq!(api.api().preview().await.unwrap(),"preview"); }
    let transport=std::sync::Arc::new(transport(url,"Bearer fixture-access"));
    let wrong_major=Client::new(move |mut request:Request| { let transport=transport.clone(); async move {
      request.api_version=2; transport.request(request).await
    }});
    let error=wrong_major.api().greet("must not execute".into()).await.unwrap_err();
    assert_eq!(error.code,"api-incompatible");assert!(error.is_compatibility());
    let old_transport=old::HttpTransport::new(url).unwrap().with_header_provider(|| async {
      let mut headers=old::HeaderMap::new();headers.insert("authorization",old::HeaderValue::from_static("Bearer fixture-access"));Ok(headers)
    });
    assert_eq!(old::Client::new(old_transport).api().greet("Rust".into()).await.unwrap(),expected);
  }
}); }
`);
    await run('rustup', ['run', '1.96.1', 'cargo', 'generate-lockfile', '--offline'], consumer);
    const checked = await run('rustup', ['run', '1.96.1', 'cargo', 'test', '--offline', '--locked', '--test', 'replicas'], consumer);
    assert.match(checked.stdout, /test result: ok/);
    // Rejected calls must not reach mutation bodies; successful writes execute once.
    for (const replica of [old, modern]) assert.equal(replica.record.stdout.split('\n').filter(line => line === 'effect:greet').length, 4);
    assert.equal(old.record.stdout.split('\n').filter(line => line === 'effect:preview').length, 0);
    assert.equal(modern.record.stdout.split('\n').filter(line => line === 'effect:preview').length, 2);
  } finally {
    for (const record of services) {
      if (record.child.exitCode === null) record.child.kill();
      let timer;
      try { await Promise.race([record.exited, new Promise((_, reject) => { timer=setTimeout(() => reject(Error('Owned replica did not exit')), 5000); })]); }
      finally { clearTimeout(timer); }
    }
    fs.rmSync(temporary, {recursive: true, force: true});
  }
});
