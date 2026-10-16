// One integrated acceptance executable. Reuses accepted SQL driver/migration evidence.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import https from 'node:https';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {spawn,execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const linux=process.argv.includes('--linux');
if(process.argv.slice(2).some(value=>value!=='--linux'))throw Error('Use --linux or no options.');
const compiler=process.env.TSRUST;if(!compiler)throw Error('Set TSRUST to the current compiler.');
const target=process.env.CARGO_TARGET_DIR;if(!target)throw Error('Set the one shared CARGO_TARGET_DIR.');
const scratch=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-service-'));
const name=`tsrust-service-${process.pid}`;
const run=(command,args,options={})=>{try{return execFileSync(command,args,{cwd:root,encoding:'utf8',timeout:600000,maxBuffer:8*1024*1024,...options});}catch(error){if(error.stderr)process.stderr.write(error.stderr);throw Error(`${path.basename(command)} failed (status ${error.status})`);}};
const ca=fs.readFileSync(path.join(root,'crates/tsrust-compiler/tests/fixtures/tls/ca.pem'));
let child,container=false,output='',errors='';
const admin=crypto.randomBytes(32).toString('hex');
const environment={...process.env,SERVICE_ADMIN_TOKEN:admin,SERVICE_ORIGIN:'https://localhost',SERVICE_ADDRESS:'127.0.0.1:0',DATABASE_URL:`sqlite:${scratch}/service.db?mode=rwc`,SERVICE_CERT:path.join(root,'crates/tsrust-compiler/tests/fixtures/tls/server.pem'),SERVICE_KEY:path.join(root,'crates/tsrust-compiler/tests/fixtures/tls/server.key')};
async function until(predicate,timeout=20000){const deadline=Date.now()+timeout;while(Date.now()<deadline){const result=predicate();if(result)return result;await new Promise(resolve=>setTimeout(resolve,25));}throw Error('Service readiness/shutdown timed out');}
function request(port,method,url,body='',headers={}){
 return new Promise((resolve,reject)=>{const input=https.request({host:'127.0.0.1',port,servername:'localhost',ca,agent:false,method,path:url,headers:{...headers,'content-length':Buffer.byteLength(body)},timeout:10000},response=>{const chunks=[];response.on('data',chunk=>chunks.push(chunk));response.on('end',()=>resolve({status:response.statusCode,headers:response.headers,body:Buffer.concat(chunks).toString('utf8')}));});input.on('timeout',()=>input.destroy(Error('HTTP timeout')));input.on('error',error=>reject(Error(`${method} ${url}: ${error.message}`)));if(headers.expect==='100-continue'){input.on('continue',()=>input.end(body));input.flushHeaders();}else{input.end(body);}});
}
async function start(){
 output='';errors='';
 if(linux){run('docker',['run','--detach','--name',name,'--publish','127.0.0.1::8443','--volume',`${scratch}:/work`,'--volume',`${target}:/target:ro`,'--workdir','/work','--env-file',path.join(scratch,'service.env'),'rust:1.96.1-slim-bookworm','/target/aarch64-unknown-linux-gnu/debug/program']);container=true;
  await until(()=>{output=run('docker',['logs',name]);return output.includes('"kind":"ready"');});return Number(run('docker',['port',name,'8443/tcp']).trim().split(':').at(-1));
 }
 child=spawn(path.join(scratch,'service'),[],{cwd:root,env:environment,stdio:['ignore','pipe','pipe']});child.stdout.on('data',chunk=>{output+=chunk;});child.stderr.on('data',chunk=>{errors+=chunk;});
 return await until(()=>{if(child.exitCode!==null)throw Error(`Service exited before readiness: ${errors}`);const line=output.split('\n').find(line=>line.includes('"kind":"ready"'));return line?Number(JSON.parse(line).address.split(':').at(-1)):false;});
}
async function stop(signal){
 if(linux){run('docker',['kill','--signal',signal,name]);await until(()=>run('docker',['inspect','--format','{{.State.Running}}',name]).trim()==='false');output=run('docker',['logs',name]);assert.equal(run('docker',['inspect','--format','{{.State.ExitCode}}',name]).trim(),'0');run('docker',['rm',name]);container=false;}else{child.kill(signal);await until(()=>child.exitCode!==null||child.signalCode!==null);assert.equal(child.exitCode,0,errors);}
}
try{
 if(linux){
  const registry=path.join(process.env.CARGO_HOME||path.join(os.homedir(),'.cargo'),'registry');
  const mounts=['--volume',`${root}:/source:ro`,'--volume',`${scratch}:/work`,'--volume',`${target}:/target`];
  run('docker',['run','--rm','--network','none',...mounts,'--volume',`${registry}:/cargo/registry:ro`,'--workdir','/source','--env','CARGO_HOME=/cargo','--env','CARGO_TARGET_DIR=/target','--env','CARGO_INCREMENTAL=0','rust:1.96.1-slim-bookworm','cargo','build','--offline','--locked','--jobs','1','--target','aarch64-unknown-linux-gnu','--bin','tsrust','--bin','tsrust-lsp']);
  const linuxBin='/target/aarch64-unknown-linux-gnu/debug';
  run('docker',['run','--rm','--network','none',...mounts,'--workdir','/source','--env','CARGO_HOME=/work/empty-compiler-home','rust:1.96.1-slim-bookworm',`${linuxBin}/tsrust`,'emit','examples/service/acceptance.tsr','--package','-o','/work/package']);
  const messages=[{jsonrpc:'2.0',id:1,method:'initialize',params:{processId:null,rootUri:'file:///source',capabilities:{}}},{jsonrpc:'2.0',method:'initialized',params:{}},{jsonrpc:'2.0',id:2,method:'shutdown',params:null},{jsonrpc:'2.0',method:'exit',params:null}].map(value=>{const body=JSON.stringify(value);return `Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`;}).join('');
  const protocol=run('docker',['run','--rm','--network','none','--interactive',...mounts,'--workdir','/source','rust:1.96.1-slim-bookworm',`${linuxBin}/tsrust-lsp`],{input:messages});assert.ok(protocol.includes('capabilities'),'Linux editor server did not initialize');
  run(process.execPath,['tools/package-toolchain.mjs',path.join(target,'aarch64-unknown-linux-gnu/debug/tsrust'),path.join(target,'aarch64-unknown-linux-gnu/debug/tsrust-lsp'),'aarch64-unknown-linux-gnu',path.join(scratch,'toolchain')]);

  fs.mkdirSync(path.join(scratch,'examples/service'),{recursive:true});fs.cpSync(path.join(root,'examples/service/migrations'),path.join(scratch,'examples/service/migrations'),{recursive:true});
  fs.copyFileSync(environment.SERVICE_CERT,path.join(scratch,'cert.pem'));fs.copyFileSync(environment.SERVICE_KEY,path.join(scratch,'key.pem'));
  fs.writeFileSync(path.join(scratch,'service.env'),Object.entries({...environment,SERVICE_ADDRESS:'0.0.0.0:8443',DATABASE_URL:'sqlite:/work/service.db?mode=rwc',SERVICE_CERT:'/work/cert.pem',SERVICE_KEY:'/work/key.pem'}).filter(([key])=>key.startsWith('SERVICE_')||key==='DATABASE_URL').map(([key,value])=>`${key}=${value}`).join('\n'),{mode:0o600});
  // Empty Cargo home plus network isolation proves the exported package is self-contained.
  run('docker',['run','--rm','--network','none','--volume',`${scratch}:/work`,'--volume',`${target}:/target`,'--workdir','/work/package','--env','CARGO_HOME=/work/empty-cargo-home','--env','CARGO_TARGET_DIR=/target','--env','CARGO_INCREMENTAL=0','rust:1.96.1-slim-bookworm','cargo','build','--offline','--locked','--jobs','1','--target','aarch64-unknown-linux-gnu']);
 }else if(process.env.SERVICE_BINARY){fs.copyFileSync(process.env.SERVICE_BINARY,path.join(scratch,'service'));}else{run(compiler,['build','examples/service/acceptance.tsr','-o',path.join(scratch,'service')]);}
 let port=await start();
 assert.equal((await request(port,'GET','/health')).body,'ready');
 assert.equal((await request(port,'GET','/notes')).status,401);
 assert.equal((await request(port,'POST','/session','',{authorization:'Bearer invalid'})).status,401);
 const login=await request(port,'POST','/session','',{authorization:`Bearer ${admin}`,origin:'https://localhost'});assert.equal(login.status,201);
 const setCookie=login.headers['set-cookie'][0];for(const flag of ['Secure','HttpOnly','SameSite=Strict','Path=/'])assert.ok(setCookie.includes(flag));
 const cookie=setCookie.split(';')[0],headers={cookie,origin:'https://localhost'};
 assert.equal(JSON.parse((await request(port,'GET','/notes','',headers)).body).body,'ready');
 assert.equal((await request(port,'POST','/notes','body=persisted+%C3%A9',{...headers,'content-type':'application/x-www-form-urlencoded'})).status,200);
 assert.equal((await request(port,'POST','/notes','body=%FF',{...headers,'content-type':'application/x-www-form-urlencoded'})).status,400);
 assert.equal((await request(port,'POST','/notes','body=a&body=b',{...headers,'content-type':'application/x-www-form-urlencoded'})).status,400);
 assert.equal((await request(port,'POST','/notes','body=a',{...headers,origin:'https://evil.invalid','content-type':'application/x-www-form-urlencoded'})).status,403);
 assert.equal((await request(port,'POST','/notes','x'.repeat(65537),{...headers,expect:'100-continue'})).status,413);
 const multipart='--x\r\nContent-Disposition: form-data; name="attachment"; filename="../private"\r\nContent-Type: application/octet-stream\r\n\r\nabc\r\n--x--\r\n';
 assert.equal(JSON.parse((await request(port,'POST','/upload',multipart,{...headers,'content-type':'multipart/form-data; boundary=x'})).body).bytes,3);
 assert.equal((await request(port,'GET','/../private','',headers)).status,404);
 await stop('SIGTERM');
 for(const secret of [admin,cookie,'persisted','evil.invalid','secret-body','secret-token','secret-cookie','secret-query','must-not-leak'])assert.ok(!output.includes(secret),'secret leaked into structured output');
 {const events=output.split('\n').filter(line=>line.includes('"kind":"event"')).map(JSON.parse);assert.ok(events.length>0);for(const event of events)assert.deepEqual(Object.keys(event).sort(),['elapsedMs','kind','operation','phase','requestId','site','status']);}
 port=await start();assert.equal(JSON.parse((await request(port,'GET','/notes','',headers)).body).body,'persisted é');
 assert.equal((await request(port,'DELETE','/session','',headers)).status,204);assert.equal((await request(port,'GET','/notes','',headers)).status,401);
 await stop('SIGINT');
 console.log(JSON.stringify({kind:'integrated-service',platform:linux?'linux-aarch64':process.platform,verified:true,checks:['TLS','authentication','session-revocation','persistence','forms','multipart','CORS','body-limit','static-allowlist','redaction','SIGTERM','SIGINT'],offlinePackage:linux,linuxToolchain:linux?JSON.parse(fs.readFileSync(path.join(scratch,'toolchain/manifest.json'),'utf8')):undefined,containerImage:linux?run('docker',['image','inspect','--format','{{.Id}}','rust:1.96.1-slim-bookworm']).trim():undefined,compilerSha256:crypto.createHash('sha256').update(fs.readFileSync(compiler)).digest('hex')}));
}finally{
 if(container)try{run('docker',['rm','--force',name]);}catch{}
 if(child&&child.exitCode===null&&child.signalCode===null){child.kill('SIGKILL');await until(()=>child.exitCode!==null||child.signalCode!==null).catch(()=>{});}
 fs.rmSync(scratch,{recursive:true,force:true});
}
