import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
const log=process.argv[2];assert(log,'Pass the compiler lazy-child log');const directory=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-lazy-oracle-'));
try{
 const compiler=spawnSync(process.execPath,[new URL('../runtime/node_modules/typescript/bin/tsc',import.meta.url).pathname,'--target','es2022','--lib','es2022','--skipLibCheck','--outDir',directory,new URL('./lazy-children-reference.ts',import.meta.url).pathname],{encoding:'utf8'});assert.equal(compiler.status,0,compiler.stdout+compiler.stderr);
 const reference=spawnSync(process.execPath,[path.join(directory,'lazy-children-reference.js')],{encoding:'utf8'});assert.equal(reference.status,0,reference.stderr);const expected=JSON.parse(reference.stdout).nativeTrace;
 const actual=fs.readFileSync(log,'utf8').split('\n').flatMap(line=>{const at=line.indexOf('{');if(at<0)return [];try{const value=JSON.parse(line.slice(at));return value.case==='lazy-children-native'?[value]:[];}catch{return [];}});assert.equal(actual.length,1);assert.equal(actual[0].trace.at(-1),'Lazy children passed');assert.deepEqual(actual[0].trace.slice(0,-1),expected);console.log(JSON.stringify({case:'lazy-children-native-oracle',status:'pass',trace:expected,timed:false}));
}finally{fs.rmSync(directory,{recursive:true,force:true});}
