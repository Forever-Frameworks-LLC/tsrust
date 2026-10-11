import fs from 'node:fs';import path from 'node:path';import os from 'node:os';import assert from 'node:assert/strict';import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url));
const compiler=process.env.TSRUST;
if(!compiler)throw Error('Set TSRUST to the verified compiler executable');
const pins=JSON.parse(fs.readFileSync(root+'/versions.json'));
const {prepareDataImport}=await import(root+'/typed-data-imports.mjs');
const {assertABI,adaptRustDriver,adaptTSDriver}=await import(root+'/adapters.mjs');
const {expectedChecksum}=await import(root+'/oracle.mjs');
const row=JSON.parse(fs.readFileSync(root+'/manifest.json')).workloads.find(w=>w.id==='native_json_import');
const build=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-data-benchmark-'));
const run=(command,args)=>execFileSync(command,args,{encoding:'utf8',timeout:60000});
try {
 const source=fs.readFileSync(root+'/'+row.source,'utf8');
 const emitted=run(compiler,['emit',root+'/'+row.source]);
 assertABI(emitted,row,{source});
 fs.writeFileSync(build+'/native.rs',emitted.replace('fn main()','fn source_main()')+'\n'+adaptRustDriver(fs.readFileSync(root+'/driver.rs','utf8'),row,{emitted,source}));
 run('rustup',['run',pins.rust,'rustc','--edition=2024','-A','warnings',build+'/native.rs','-o',build+'/native']);
 const options=prepareDataImport(row,build);
 fs.writeFileSync(build+'/native_json_import.ts',fs.readFileSync(root+'/'+row.counterpart,'utf8')+'\n'+adaptTSDriver(fs.readFileSync(root+'/driver.ts','utf8'),row));
 fs.writeFileSync(build+'/tsconfig.json',JSON.stringify({extends:root+'/tsconfig.json',files:[build+'/native_json_import.ts'],compilerOptions:{outDir:build,...options}}));
 run(process.execPath,[root+'/node_modules/typescript/bin/tsc','--project',build+'/tsconfig.json']);
 for(const [n,seed] of [[0,0],[1,7],[2,19],[17,7],[2000,19]]){
  for(const [exe,args] of [[build+'/native',[]],[process.execPath,[build+'/native_json_import.js']]]){
   const result=JSON.parse(run(exe,[...args,String(n),String(seed),'1','0']));assert.equal(result.checksum,expectedChecksum(row.id,n,seed));assert.deepEqual(result.durations,[]);
  }
 }
 console.log('PASS typed imported data: native and Node after tsc; five cases; zero timing samples');
} finally {fs.rmSync(build,{recursive:true,force:true});}
