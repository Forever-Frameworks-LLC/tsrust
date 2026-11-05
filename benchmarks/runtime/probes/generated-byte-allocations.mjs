// Untimed generated-code acceptance profile. No compiler build, timing or DB ingestion.
// Run only under the coordinated BUILD lease with a verified candidate TSRUST.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {adaptRustDriver,assertABI} from '../adapters.mjs';
const here=path.dirname(fileURLToPath(import.meta.url)),runtime=path.resolve(here,'..');
const pins=JSON.parse(fs.readFileSync(path.join(runtime,'versions.json'),'utf8'));
if(process.version!==pins.node)throw Error('Run this preparation/control with canonical Node24.11.1.');
const compiler=process.env.TSRUST;
if(!compiler||!path.isAbsolute(compiler))throw Error('Set TSRUST to the verified candidate compiler absolute path. This script never builds the compiler.');
const target=process.env.CARGO_TARGET_DIR;
if(!target||!path.isAbsolute(target))throw Error('Use the coordinated absolute shared CARGO_TARGET_DIR.');
const build=path.join(target,'generated-byte-allocation-prototype');fs.mkdirSync(build,{recursive:true});
const hash=value=>createHash('sha256').update(value).digest('hex');
const run=(command,args)=>{const child=spawnSync(command,args,{encoding:'utf8',cwd:runtime,timeout:180000,maxBuffer:4*1024*1024,env:{...process.env,CARGO_INCREMENTAL:'0',CARGO_BUILD_JOBS:'1'}});if(child.error||child.status!==0)throw Error(`${command}: ${child.error??child.stderr+child.stdout}`);return child.stdout.trim();};
const cases=[];
for(const mode of ['inline_little','inline_big','stored'])for(const seed of mode==='stored'?[0,7]:[mode.endsWith('little')?0:7]){
 cases.push({id:`packet_${mode}_${seed}`,source:`packet_${mode}.tsr`,reference:'packet_floor.rs',seed,input:'packet',iterations:[0,1,256],fieldCalls:18,kind:'packet'});
 for(const direction of ['encode','decode'])for(const corpus of ['mixed','cjk'])for(const size of ['token','4k','1m'])cases.push({id:`utf16_${direction}_${mode}_${seed}_${corpus}_${size}`,source:`utf16_${direction}_${mode}.tsr`,reference:`utf16_${direction}_floor.rs`,seed,input:`${corpus}-${size}`,iterations:[0,1,8],fieldCalls:1,kind:`utf16_${direction}`});
}
const selected=process.argv.find(argument=>argument.startsWith('--only='))?.slice(7).split(',');
if(selected?.some(id=>!cases.some(row=>row.id===id)))throw Error('Unknown generated-byte allocation case.');
const active=cases.filter(row=>!selected||selected.includes(row.id));
const driver=fs.readFileSync(path.join(runtime,'allocation-driver.rs'),'utf8');
const contract={family:'byte-operations',result:'int',parameters:['int','int','string'],inputText:'allocation-probe'};
const artifacts=new Map(),rows=[];
function prepare(file,generated,opt){
 const key=`${generated?'native':'rust'}_${path.basename(file,path.extname(file))}_o${opt}`;
 if(artifacts.has(key))return artifacts.get(key);
 const source=fs.readFileSync(file,'utf8');const emitted=generated?run(compiler,['emit',file]):source;
 assertABI(emitted,{...contract,id:key},{source:generated?source:undefined});
 if(!emitted.includes('fn main()')||emitted.split('fn main()').length!==2)throw Error('Generated/source entrypoint changed.');
 if(generated&&/\bunsafe\b/.test(emitted))throw Error('Production emitted source contains unsafe.');
 let instrument=adaptRustDriver(driver,{...contract,id:key},{probe:true,emitted,source:generated?source:undefined});
 const marker='    let n: i64';if(!instrument.includes(marker)||!instrument.includes('std::hint::black_box(7)'))throw Error('Allocation driver markers changed.');
 instrument=instrument.replace(marker,'    let seed: i64 = arguments[3].parse().unwrap();\n'+marker).replaceAll('std::hint::black_box(7)','std::hint::black_box(seed)');
 const instrumented=emitted.replace('fn main()','fn source_main()')+'\nfn probe_checksum(value:&i64)->i64{*value}\n'+instrument;
 const sourceFile=path.join(build,`${key}.rs`),binary=path.join(build,key+(process.platform==='win32'?'.exe':''));fs.writeFileSync(sourceFile,instrumented);
 const flags=['--edition=2024','-C',`opt-level=${opt}`,'-C','overflow-checks=on','-A','warnings'];
 let safeCommand;
 if(generated){const safeSource=path.join(build,`${key}-production.rs`),safeBinary=path.join(build,key+'-production'+(process.platform==='win32'?'.exe':''));fs.writeFileSync(safeSource,emitted);safeCommand=['rustup','run',pins.rust,'rustc',...flags,'-F','unsafe_code',safeSource,'-o',safeBinary];run(safeCommand[0],safeCommand.slice(1));run(safeBinary,[]);}
 const command=['rustup','run',pins.rust,'rustc',...flags,sourceFile,'-o',binary];run(command[0],command.slice(1));
 const record={key,source:file,generated,opt,binary,sourceSha256:hash(source),emittedSha256:hash(emitted),instrumentedSha256:hash(instrumented),binarySha256:hash(fs.readFileSync(binary)),command,safeProductionCommand:safeCommand};artifacts.set(key,record);return record;
}
function expected(row,n){
 if(row.kind==='packet')return n*([65,-12345,-1234567,123456789,255,65535,4294967295].reduce((sum,value)=>sum+value*4,0)-3);
 const size=row.input.endsWith('1m')?1048576:row.input.endsWith('4k')?4096:32,cjk=row.input.startsWith('cjk'),little=row.seed%2===0;
 if(row.kind==='utf16_decode')return n*((cjk?size*1.5:size)+2);
 return n*(cjk?size+(little?164:195):size*1.25+(little?233:98));
}
for(const opt of [0,3])for(const row of active){
 const sources={native:prepare(path.join(here,'generated-bytes',row.source),true,opt),rust:prepare(path.join(here,'generated-bytes',row.reference),false,opt)};
 const samples=[];
 for(const n of row.iterations)for(let sample=0;sample<3;sample++)for(const engine of sample%2?['rust','native']:['native','rust']){
  const data=JSON.parse(run(sources[engine].binary,[String(n),row.input,String(row.seed)]));
  if(data.checksum!==expected(row,n)||data.liveBytesAfterOutputDrop!==0||data.liveBytesAfterDrop!==0)throw Error(`Generated-byte correctness/drop failure: ${row.id}/${engine}/${n}`);
  samples.push({engine,n,sample,...data});
 }
 for(const engine of ['native','rust'])for(const n of row.iterations)for(const key of ['allocations','reallocations','requestedBytes','retainedRequestedBytes','peakRequestedBytes'])if(new Set(samples.filter(sample=>sample.engine===engine&&sample.n===n).map(sample=>sample[key])).size!==1)throw Error(`Nondeterministic generated allocation count: ${row.id}/${engine}/${n}/${key}`);
 const last=row.iterations.at(-1),point=(engine,n)=>samples.find(sample=>sample.engine===engine&&sample.n===n),delta=engine=>point(engine,last).allocations-point(engine,0).allocations;
 const extraAllocationsPerIteration=(delta('native')-delta('rust'))/last;
 const difference=key=>((point('native',last)[key]-point('native',0)[key])-(point('rust',last)[key]-point('rust',0)[key]))/last;
 const extraReallocationsPerIteration=difference('reallocations'),extraRequestedBytesPerIteration=difference('requestedBytes');
 rows.push({...row,opt,samples,extraAllocationsPerIteration,extraAllocationsPerField:extraAllocationsPerIteration/row.fieldCalls,extraReallocationsPerIteration,extraRequestedBytesPerIteration,needsAllocationReview:extraAllocationsPerIteration!==0||extraReallocationsPerIteration!==0});
 console.log(`Profiled generated bytes ${row.id} O${opt}: extra allocations/iteration=${extraAllocationsPerIteration}`);
}
const result={kind:'generated-byte-allocations-v1',untimed:true,date:new Date().toISOString(),compiler,compilerVersion:run(compiler,['--version']),compilerSha256:hash(fs.readFileSync(compiler)),nodeVersion:process.version,rustVersion:run('rustup',['run',pins.rust,'rustc','--version']),driverSha256:hash(driver),artifacts:[...artifacts.values()],rows};
const output=path.join(build,'results.json');fs.writeFileSync(output,JSON.stringify(result));console.log(output);
