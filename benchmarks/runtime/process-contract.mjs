import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {asyncScalarABI,asyncScalarRustDriver,asyncScalarAllocationDriver} from './async-io-contract.mjs';
export const processIDs=['process_launch_wait','process_capture_bytes','process_duplex_bytes','process_duplex_reader'];
let helper=null;
const hash=value=>createHash('sha256').update(value).digest('hex');
export function prepareProcessHelper({directory,rustVersion}){
 const source=new URL('process-helper.rs',import.meta.url);const executable=path.join(directory,'process-helper'+(process.platform==='win32'?'.exe':''));
 const sourceSha256=hash(fs.readFileSync(source));
 const command=['rustup','run',rustVersion,'rustc','--edition=2024','-C','opt-level=3',fileURLToPath(source),'-o',executable];
 const result=spawnSync(command[0],command.slice(1),{encoding:'utf8',timeout:120000});if(result.error||result.status!==0)throw Error(`Process helper preparation failed: ${result.error??result.stderr}`);
 helper={executable,source:fileURLToPath(source),rustVersion,sourceSha256,binarySha256:hash(fs.readFileSync(executable)),command};assertProcessHelperUnchanged();return helper;
}
export function processEnvironment(row,base){if(row?.family!=='child-processes')return base;if(!helper)throw Error('Process helper not prepared');assertProcessHelperUnchanged();return {...base,TSRUST_PROCESS_HELPER:helper.executable};}
// Called outside measured child samples: before and after each engine process,
// after helper compilation, and before publishing aggregate evidence.
export function assertProcessHelperUnchanged(evidence=helper){if(!evidence)return;for(const [kind,file,expected]of [['source',evidence.source,evidence.sourceSha256],['binary',evidence.executable,evidence.binarySha256]])if(hash(fs.readFileSync(file))!==expected)throw Error(`Process helper ${kind} changed`);}
export function processHelperEvidence(){assertProcessHelperUnchanged();return helper;}
export function processChecksum(id,n,seed){if(!processIDs.includes(id)||!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(seed)||seed<0)throw Error('Invalid process oracle domain');const mod=1000000007n;let sum=0n;for(let i=0;i<n;i++){if(id==='process_launch_wait')sum=(sum*131n+(BigInt(seed)+BigInt(i)))%mod;else for(let stream=0;stream<(id==='process_capture_bytes'?2:1);stream++)for(let j=0;j<16384;j++)sum=(sum*131n+((BigInt(seed)+BigInt(j)*(stream===0?1n:3n)+(stream===0?0n:17n))%251n))%mod;}return Number(sum);}
export function assertProcessSource(source,w,engine){if(!processIDs.includes(w.id)||w.family!=='child-processes'||w.result!=='int'||w.parameters.join(',')!=='int,int'||w.iterations!==10)throw Error('Process workload domain changed');const native=engine==='native';if(hash(source)!==w[native?'sourceSha256':'counterpartSha256'])throw Error('Process source identity changed');const snippets=[native?'async function workload(n:int,seed:int):Task<int>':'async function workload(n:number,seed:number):Promise<number>',native?"'TSRUST_PROCESS_HELPER'":'TSRUST_PROCESS_HELPER','sum=(sum*131+'];if(w.id==='process_launch_wait')snippets.push(native?'await child.wait()':"child.once('close'","'silent'",native?'const code=status.code':'if(code!==0)',native?'code===undefined':'resolve(code)','+code+seed+i');else{snippets.push(native?'await run(executable':'Buffer.concat(stdout)','16384',native?'unwrap(at(output.stdout,j))':'output.stdout[j]');if(w.id==='process_capture_bytes')snippets.push(native?'unwrap(at(output.stderr,j))':'output.stderr[j]',"'flood'");else snippets.push(native?'clone(payload)':'Buffer.from(payload)',"'echo'");if(w.id==='process_duplex_reader')snippets.push(native?'reader(clone(payload))':'Readable.from(Buffer.from(payload))');}for(const text of snippets)if(!containsCodeSnippet(source,text))throw Error('Process semantic guard missing '+text);}
export function processABI(emitted,w,source,{reference=false}={}){if(reference){if(!/async fn function0\(n:i64,seed:i64\)->i64/.test(emitted))throw Error('Process reference ABI changed');for(const text of ['"TSRUST_PROCESS_HELPER"','kill_on_drop(true)','child.wait()','sum=(sum*131+'])if(!containsCodeSnippet(emitted,text))throw Error('Process reference guard missing '+text);if(w.id!=='process_launch_wait'){for(const text of ['read_to_end(&mut output)','read_to_end(&mut errors)','for value in output','16384'])if(!containsCodeSnippet(emitted,text))throw Error('Process reference materialization guard missing '+text);if(w.id==='process_capture_bytes'&&!containsCodeSnippet(emitted,'for value in errors'))throw Error('Process stderr observation missing');if(w.id==='process_duplex_reader'&&!containsCodeSnippet(emitted,'tokio::io::copy'))throw Error('Process Reader floor changed');}return {functionName:'function0',call:'function0(n,seed).await',result:'i64',runtimeOnce:true};}assertProcessSource(source,w,'native');return asyncScalarABI(emitted,w);}
export function processRustDriver(w,emitted,source,options){return asyncScalarRustDriver(processABI(emitted,w,source,options));}
export function processAllocationDriver(driver,w,emitted,source,options){return asyncScalarAllocationDriver(driver,processABI(emitted,w,source,options));}
