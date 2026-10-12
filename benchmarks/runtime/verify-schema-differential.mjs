// Untimed S11 differential evidence. One native compilation covers the whole corpus.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {spawnSync} from 'node:child_process';
import {schemaCases,scenarioModes,differences,nativeSource,referencePaths,nativePaths,normalizedPaths} from './schema-differential-cases.mjs';
const require=createRequire(import.meta.url);
const packageInfo=JSON.parse(fs.readFileSync(new URL('./package.json',import.meta.url),'utf8'));
const lock=JSON.parse(fs.readFileSync(new URL('./package-lock.json',import.meta.url),'utf8'));
const installed=require('zod/package.json');
assert.match(packageInfo.devDependencies.zod,/^\d+\.\d+\.\d+$/);
assert.equal(installed.version,packageInfo.devDependencies.zod);
assert.equal(lock.packages['node_modules/zod'].version,installed.version);
assert(process.env.TSRUST,'Set TSRUST to the verified compiler');
assert.equal(process.env.CARGO_INCREMENTAL,'0','Disable incremental compilation');
assert(process.env.CARGO_TARGET_DIR,'Reuse the shared Cargo target');
const hash=source=>createHash('sha256').update(source).digest('hex');
const directory=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-schema-differential-'));
function run(args){
 const child=spawnSync(process.env.TSRUST,args,{encoding:'utf8',timeout:60000,maxBuffer:32*1024*1024});
 assert.equal(child.status,0,child.stderr||child.stdout||String(child.error));return child.stdout;
}
try{
 const file=path.join(directory,'corpus.tsr');fs.writeFileSync(file,nativeSource());
 run(['format',file]);const source=fs.readFileSync(file,'utf8');run(['format',file]);assert.equal(fs.readFileSync(file,'utf8'),source,'Formatting must be idempotent');
 console.log(JSON.stringify({reference:'zod',version:installed.version,integrity:lock.packages['node_modules/zod'].integrity,sourceSha256:hash(source),casesSha256:hash(fs.readFileSync(new URL('./schema-differential-cases.mjs',import.meta.url))),timed:false,differences}));
 const rows=new Map();
 for(const line of run(['run',file]).trim().split('\n')){
  const match=/^(\S+) (\d+) (owned|fused) (ok|err) (.*)$/.exec(line);assert(match,`Unexpected output: ${line}`);
  const key=match.slice(1,4).join(':');assert(!rows.has(key),`Duplicate observation: ${key}`);rows.set(key,{ok:match[4]==='ok',value:JSON.parse(match[5])});
 }
 let observations=0;
 for(const scenario of schemaCases){
  const schema=scenario.reference();
  for(const [index,text]of scenario.inputs.entries()){
   const reference=schema.safeParse(JSON.parse(text));
   for(const mode of scenarioModes(scenario)){
    const key=`${scenario.id}:${index}:${mode}`;const native=rows.get(key);assert(native,`Missing ${key}`);
    assert.equal(native.ok,reference.success,`${key}: acceptance`);
    if(reference.success)assert.deepEqual(JSON.parse(JSON.stringify(native.value)),JSON.parse(JSON.stringify(reference.data)),`${key}: output`);
    else assert.deepEqual(normalizedPaths(nativePaths(native.value)),normalizedPaths(referencePaths(reference.error.issues)),`${key}: failure locations`);
    observations++;
   }
  }
  console.log(JSON.stringify({case:scenario.id,inputs:scenario.inputs.length,modes:scenarioModes(scenario),status:'pass',timed:false}));
 }
 assert.equal(rows.size,observations,'No unaccounted native observations');
 console.log(JSON.stringify({status:'pass',families:schemaCases.length,observations,timed:false}));
}finally{fs.rmSync(directory,{recursive:true,force:true});}
