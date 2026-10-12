import {compilerSourceInputs} from './compiler-source-inputs.mjs';
// Quick correctness/ABI verification. Warmup-only drivers execute without timing samples.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {assertSchemaSource,schemaChecksum} from './schema-contract.mjs';
import {prepareNativeDependencies} from './native-dependencies.mjs';
import {assertABI,adaptRustDriver,adaptTSDriver} from './adapters.mjs';
const here=path.dirname(fileURLToPath(import.meta.url)),root=path.resolve(here,'../..');
const compiler=process.argv.find(arg=>arg.startsWith('--compiler='))?.slice(11)??path.join(root,'target/debug/tsrust');
const output=process.argv.find(arg=>arg.startsWith('--output='))?.slice(9);
const temporary=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-schema-check-'));
const pins=JSON.parse(fs.readFileSync(path.join(here,'versions.json')));
const workload=process.argv.find(arg=>arg.startsWith('--workload='))?.slice(11)??'native_schema_pipeline';
const row=JSON.parse(fs.readFileSync(path.join(here,'manifest.json'))).workloads.find(row=>row.id===workload&&row.family==='native-schema');
if(!row)throw Error('Unknown schema workload: '+workload);
const sha=value=>createHash('sha256').update(value).digest('hex');
const commands=[];
function run(command,args=[]){
 commands.push([command,...args]);
 const result=spawnSync(command,args,{encoding:'utf8',timeout:60000,maxBuffer:16*1024*1024});
 if(result.error||result.status!==0)throw Error([command,...args].join(' ')+': '+(result.error??result.stderr));
 return result.stdout;
}
try{
 const source=fs.readFileSync(path.join(here,'workloads',row.id+'.tsr'),'utf8');
 const counterpart=fs.readFileSync(path.join(here,'workloads',row.id+'.ts'),'utf8');
 assertSchemaSource(source,row,'native');assertSchemaSource(counterpart,row,'typescript');
 const emitted=run(compiler,['emit',path.join(here,'workloads',row.id+'.tsr')]);
 const binding=assertABI(emitted,row,{source});
 const driverRS=fs.readFileSync(path.join(here,'driver.rs'),'utf8'),driverTS=fs.readFileSync(path.join(here,'driver.ts'),'utf8');
 fs.writeFileSync(path.join(temporary,'program.rs'),emitted.replace('fn main()', 'fn source_main()')+'\n'+adaptRustDriver(driverRS,row,{emitted,source}));
 fs.writeFileSync(path.join(temporary,'program.ts'),counterpart+'\n'+adaptTSDriver(driverTS,row));
 fs.writeFileSync(path.join(temporary,'tsconfig.json'),JSON.stringify({extends:path.join(here,'tsconfig.json'),files:[path.join(temporary,'program.ts')],compilerOptions:{outDir:temporary,typeRoots:[path.join(here,'node_modules/@types')]}}));
 const dependencies=prepareNativeDependencies({emitted,compiler,source:path.join(here,row.source),build:temporary,rustVersion:pins.rust,targetDirectory:path.dirname(path.dirname(compiler))});
 run('rustup',['run',pins.rust,'rustc',...dependencies.args,'--edition=2024','-C','opt-level=3','-C','overflow-checks=on','-A','warnings',path.join(temporary,'program.rs'),'-o',path.join(temporary,'program')]);
 run(process.execPath,[path.join(here,'node_modules/typescript/bin/tsc'),'--project',path.join(temporary,'tsconfig.json')]);
 const cells=[];
 for(const [n,seed] of [[0,0],[1,7],[2,19],[17,3],[row.iterations,7]]){
  const expected=schemaChecksum(n,seed,row.inputText);
  for(const [engine,command,args] of [['native',path.join(temporary,'program'),[]],['node-tsc',process.execPath,[path.join(temporary,'program.js')]]]){
   const result=JSON.parse(run(command,[...args,String(n),String(seed),'3','0',row.inputText]));
   if(result.checksum!==expected||result.durations.length!==0)throw Error('Incorrect or timed verification: '+engine+'/'+n);
   cells.push({engine,n,seed,expected,actual:result.checksum});
  }
 }
 const evidence={schemaVersion:1,nativeDependencies:dependencies.evidence,verifiedOnly:true,measured:false,workload:row.id,binding,cells,sourceSha256:sha(source),counterpartSha256:sha(counterpart),emittedSha256:sha(emitted),compilerSha256:sha(fs.readFileSync(compiler)),compilerInputs:compilerSourceInputs(path.join(root,'crates/tsrust-compiler/src')),driverRSSha256:sha(driverRS),driverTSSha256:sha(driverTS),oracleSha256:sha(fs.readFileSync(path.join(here,'schema-contract.mjs'))),guardSha256:sha(fs.readFileSync(path.join(here,'schema-contract.mjs'))),node:process.version,rust:run('rustup',['run',pins.rust,'rustc','--version']).trim(),typescript:JSON.parse(fs.readFileSync(path.join(here,'node_modules/typescript/package.json'))).version,commands};
 if(output){fs.mkdirSync(path.dirname(output),{recursive:true});fs.writeFileSync(output,JSON.stringify(evidence,null,2)+'\n');}
 console.log(cells.length+' native/compiled-Node schema cells passed; no timings collected');
}finally{fs.rmSync(temporary,{recursive:true,force:true});}
