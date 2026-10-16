// Bounded standalone comparison; does not enable the editor extension or run cargo check.
import assert from 'node:assert/strict';
import {spawn, spawnSync} from 'node:child_process';
import {mkdtempSync, mkdirSync, writeFileSync, rmSync, existsSync} from 'node:fs';
import {tmpdir, homedir} from 'node:os';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
const domains = process.argv.includes('--domains');
const binary = process.env.RUST_ANALYZER ?? path.join(homedir(), '.vscode/extensions/rust-lang.rust-analyzer-0.3.3081-darwin-arm64/server/rust-analyzer');
function output(command, args) {
  const result = spawnSync(command, args, {encoding:'utf8', timeout:15000});
  assert.equal(result.status, 0, result.error?.message ?? result.stderr);
  return result.stdout.trim();
}
const version = output(binary, ['--version']);
assert.equal(version, 'rust-analyzer 0.3.3081-standalone (2ad81e72b7 2026-10-11)', 'Review reference inventory before changing rust-analyzer');
const rustc = output('rustc', ['--version', '--verbose']);
const sysroot = output('rustc', ['--print', 'sysroot']);
const sysrootSrc = path.join(sysroot, 'lib/rustlib/src/rust/library');
assert.ok(existsSync(path.join(sysrootSrc, 'std/src/lib.rs')), 'Pinned rust-src is required; inventory will not install it implicitly');
const directory = mkdtempSync(path.join(tmpdir(), 'tsrust-type-inventory-'));
mkdirSync(path.join(directory, 'src'));
writeFileSync(path.join(directory, 'Cargo.toml'), '[package]\nname="type_inventory"\nversion="0.0.0"\nedition="2024"\n[workspace]\n' + (domains ? '[dependencies]\nbigdecimal="=0.4.11"\nchrono={version="=0.4.45",default-features=false,features=["std"]}\nuuid={version="=1.28.0",default-features=false,features=["std"]}\n' : ''));
const file = path.join(directory, 'src/main.rs');
writeFileSync(file, 'fn main() {}\n');
const uri = pathToFileURL(file).href;
const configuration = {cargo:{sysroot,sysrootSrc,buildScripts:{enable:false}},checkOnSave:false,procMacro:{enable:false},cachePriming:{enable:false},numThreads:1};
const child = spawn(binary, [], {cwd:directory, env:{...process.env,CARGO_NET_OFFLINE:'true',CARGO_INCREMENTAL:'0'}, stdio:['pipe','pipe','pipe']});
let buffer = Buffer.alloc(0), sequence = 0, documentVersion = 0, stderr = '', ready;
const pending = new Map();
const initialized = new Promise(resolve => {ready = resolve;});
function send(message) { const data=Buffer.from(JSON.stringify(message)); child.stdin.write(`Content-Length: ${data.length}\r\n\r\n`); child.stdin.write(data); }
function notify(method, params) { send({jsonrpc:'2.0',method,params}); }
function request(method, params) {
  const id=++sequence;
  return new Promise((resolve,reject) => {
    const timer=setTimeout(() => {pending.delete(id);reject(new Error(`Timed out: ${method}`));},60000);
    pending.set(id,{resolve,reject,timer});send({jsonrpc:'2.0',id,method,params});
  });
}
function fail(error) { for(const item of pending.values()){clearTimeout(item.timer);item.reject(error);}pending.clear(); }
child.on('error', fail);
child.stdin.on('error', fail);
child.on('exit', code => fail(new Error(`rust-analyzer exited ${code}: ${stderr.slice(-2000)}`)));
child.stderr.on('data', data => {stderr=(stderr+data).slice(-16000);});
child.stdout.on('data', data => {
  buffer=Buffer.concat([buffer,data]);
  while(true){
    const end=buffer.indexOf('\r\n\r\n');if(end<0)return;
    const length=Number(/Content-Length: (\d+)/i.exec(buffer.subarray(0,end).toString())?.[1]);
    assert.ok(Number.isSafeInteger(length));if(buffer.length<end+4+length)return;
    const message=JSON.parse(buffer.subarray(end+4,end+4+length));buffer=buffer.subarray(end+4+length);
    if(message.method && message.id !== undefined){send({jsonrpc:'2.0',id:message.id,result:message.method==='workspace/configuration'?message.params.items.map(()=>configuration):null});}
    else if(message.id !== undefined){const item=pending.get(message.id);if(item){pending.delete(message.id);clearTimeout(item.timer);message.error?item.reject(new Error(JSON.stringify(message.error))):item.resolve(message.result);}}
    else if(message.method==='experimental/serverStatus' && message.params.quiescent){ready();}
  }
});
const deadline=setTimeout(()=>child.kill('SIGKILL'),180000);
const probes = domains ? [
 ['BigDecimal','let value = bigdecimal::BigDecimal::from(1);'],
 ['NaiveDate','let value = chrono::NaiveDate::from_ymd_opt(2024,2,29).unwrap();'],
 ['NaiveTime','let value = chrono::NaiveTime::from_hms_opt(12,0,0).unwrap();'],
 ['DateTime<Utc>','let value = chrono::DateTime::<chrono::Utc>::from_timestamp(0,0).unwrap();'],
 ['TimeDelta','let value = chrono::TimeDelta::seconds(1);'],
 ['FixedOffset','let value = chrono::FixedOffset::east_opt(0).unwrap();'],
 ['Uuid','let value = uuid::Uuid::nil();'],
] : [
 ...['i8','i16','i32','i64','i128','u8','u16','u32','u64','u128','f32','f64'].map(type=>[type,`let value: ${type} = 1${type};`]),
 ['bool','let value = true;'],['String','let value = String::new();'],['Vec<u8>','let value: Vec<u8> = Vec::new();'],
 ['Option<i64>','let value: Option<i64> = Some(1);'],['Result<i64,String>','let value: Result<i64,String> = Ok(1);'],
 ['Duration','let value = std::time::Duration::from_secs(1);'],['SystemTime','let value = std::time::SystemTime::UNIX_EPOCH;'],
 ['HashMap<String,i64>','let value: std::collections::HashMap<String,i64> = std::collections::HashMap::new();'],
];
function position(source, offset) {const before=source.slice(0,offset).split('\n');return {line:before.length-1,character:before.at(-1).length};}
function change(text){notify('textDocument/didChange',{textDocument:{uri,version:++documentVersion},contentChanges:[{text}]});}
const rows=[];
const scopedDomainMethods=new Set(['with_scale','with_scale_round','as_bigint_and_exponent','to_string','format','checked_add_days','checked_add_months','signed_duration_since','year','month0','day','num_nanoseconds','timestamp','timestamp_millis','timestamp_subsec_nanos','local_minus_utc','as_bytes','to_bytes_le','get_version_num']);
try {
 await request('initialize',{processId:process.pid,rootUri:pathToFileURL(directory).href,capabilities:{experimental:{serverStatusNotification:true},textDocument:{completion:{completionItem:{snippetSupport:true}},hover:{contentFormat:['plaintext']}}},initializationOptions:configuration});
 notify('initialized',{});
 notify('textDocument/didOpen',{textDocument:{uri,languageId:'rust',version:documentVersion,text:'fn main() {}\n'}});
 await Promise.race([initialized,new Promise((_,reject)=>{const timer=setTimeout(()=>reject(new Error('Workspace did not become ready')),60000);timer.unref();})]);
 for(const [type,declaration] of probes){
   const valid=`fn main() { ${declaration} let _ = &value; }\n`;
   change(valid);
   const hover=await request('textDocument/hover',{textDocument:{uri},position:position(valid,valid.lastIndexOf('value'))});
   const source=`fn main() { ${declaration} value. }\n`;change(source);
   const response=await request('textDocument/completion',{textDocument:{uri},position:position(source,source.indexOf('value.')+6)});
   const items=Array.isArray(response)?response:response?.items ?? [];
   rows.push({type,hover: hover?.contents ?? null,actions:items.filter(item=>item.kind===15).map(item=>item.label),members:items.filter(item=>item.kind!==15).map(item=>({name:item.label,kind:item.kind,signature:item.detail ?? item.labelDetails ?? null})).sort((a,b)=>a.name.localeCompare(b.name))});
 }
 if (!domains) assert.ok(rows.find(row=>row.type==='Duration').members.some(member=>member.name.startsWith('as_secs')),'Duration member inventory missing');
 if (!domains) assert.ok(rows.find(row=>row.type==='u8').members.some(member=>member.name.startsWith('count_ones')),'u8 member inventory missing');
 const signatures=[];
 if (domains) assert.ok(rows.every(row=>row.hover && row.members.length>0),'Missing domain declarations');
 for(const expression of domains ? ['chrono::NaiveDate::from_ymd_opt(', 'chrono::FixedOffset::east_opt(', 'uuid::Uuid::parse_str('] : ['std::time::Duration::from_secs(', 'std::time::Duration::new(', 'String::from(', 'Vec::<u8>::with_capacity(']){
   const source=`fn main() { ${expression}0); }\n`;change(source);
   const signature=await request('textDocument/signatureHelp',{textDocument:{uri},position:position(source,source.indexOf(expression)+expression.length)});
   signatures.push({expression,signature});
 }
 if(domains) for(const row of rows) for(const member of row.members) member.classification=scopedDomainMethods.has(member.name.replace(/\(.*/,''))?'partial: related capability with explicit source bounds/ownership; not overload parity':'deferred: not required by current UI/service examples';
 console.log(JSON.stringify({schemaVersion:1,status:'declaration discovery only; native behavior and Tsrust classification require witnesses',engine:'rust-analyzer',version,rustc,features:domains?'BigDecimal 0.4.11, Chrono 0.4.45 std, UUID 1.28.0 std; generated chrono-tz excluded; build scripts, proc macros and cargo check disabled':'standard library only; build scripts, proc macros and cargo check disabled',rows,signatures},null,2));
 await request('shutdown',null);notify('exit');
} finally {
 clearTimeout(deadline);child.kill('SIGTERM');
 await new Promise(resolve=>{if(child.exitCode!==null || child.signalCode!==null){resolve();return;}const timer=setTimeout(()=>{child.kill('SIGKILL');resolve();},1000);child.once('exit',()=>{clearTimeout(timer);resolve();});});
 rmSync(directory,{recursive:true,force:true});
}
