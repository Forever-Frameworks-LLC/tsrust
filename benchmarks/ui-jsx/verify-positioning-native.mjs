import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const here=path.dirname(fileURLToPath(import.meta.url));
const temporary=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-positioning-oracle-'));
try {
 const compile=spawnSync(process.execPath,[path.join(here,'../runtime/node_modules/typescript/bin/tsc'),'--target','es2022','--lib','es2022','--skipLibCheck','--outDir',temporary,path.join(here,'alert-reference.ts')],{encoding:'utf8',timeout:30000});assert.equal(compile.status,0,compile.stdout||compile.stderr);
 const reference=spawnSync(process.execPath,[path.join(temporary,'alert-reference.js')],{encoding:'utf8',timeout:30000});assert.equal(reference.status,0,reference.stderr);
 const expected=JSON.parse(reference.stdout).geometry;
 const prefix='POSITION_GEOMETRY ';
 const line=fs.readFileSync(process.argv[2],'utf8').split('\n').find(line=>line.includes(prefix));assert(line,'Native positioning observation is missing');
 const actual=JSON.parse(line.slice(line.indexOf(prefix)+prefix.length)).cases;assert.equal(actual.length,expected.length);
 for(const [index,box] of expected.entries()){
  assert.equal(actual[index].index,box.index);
  for(const axis of ['x','y','width','height'])assert(Math.abs(actual[index][axis]-box[axis])<.05,JSON.stringify({box,actual:actual[index]}));
 }
 console.log(JSON.stringify({case:'native-parent-positioning',status:'pass',cases:expected.length,reference:'independent Node after tsc',timed:false}));
} finally {fs.rmSync(temporary,{recursive:true,force:true});}
