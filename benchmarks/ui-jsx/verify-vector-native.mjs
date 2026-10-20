import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
const log=process.argv[2];assert(log,'Pass the native vector log');const directory=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-vector-oracle-'));
try{
 const compiler=spawnSync(process.execPath,[new URL('../runtime/node_modules/typescript/bin/tsc',import.meta.url).pathname,'--target','es2022','--lib','es2022','--skipLibCheck','--outDir',directory,new URL('./vector-reference.ts',import.meta.url).pathname],{encoding:'utf8'});assert.equal(compiler.status,0,compiler.stdout+compiler.stderr);
 const reference=spawnSync(process.execPath,[path.join(directory,'vector-reference.js')],{encoding:'utf8'});assert.equal(reference.status,0,reference.stderr);const expected=JSON.parse(reference.stdout).native;
 const actual=fs.readFileSync(log,'utf8').split('\n').flatMap(line=>{const at=line.indexOf('{');if(at<0)return [];try{const value=JSON.parse(line.slice(at));return value.case==='vector-viewport'?[value]:[];}catch{return [];}});assert.equal(actual.length,expected.length);
 for(const item of actual){const target=expected.find(value=>value.policy===item.policy);assert(target,item.policy);assert.equal(item.width,target.width);assert.equal(item.height,target.height);for(let i=0;i<4;i++)assert(Math.abs(item.corners[i]-target.corners[i])<.00001,JSON.stringify({item,target}));}
 console.log(JSON.stringify({case:'vector-native-oracle',status:'pass',viewports:actual.length,timed:false}));
}finally{fs.rmSync(directory,{recursive:true,force:true});}
