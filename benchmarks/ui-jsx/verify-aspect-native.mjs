import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const here=path.dirname(fileURLToPath(import.meta.url));
const root=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-aspect-oracle-'));
try {
 const result=spawnSync(process.execPath,[path.join(here,'../runtime/node_modules/typescript/bin/tsc'),'--target','es2022','--lib','es2022','--skipLibCheck','--outDir',root,path.join(here,'aspect-ratio-reference.ts')],{encoding:'utf8',timeout:30000});assert.equal(result.status,0,result.stdout||result.stderr);
 const reference=spawnSync(process.execPath,[path.join(root,'aspect-ratio-reference.js')],{encoding:'utf8',timeout:30000});assert.equal(reference.status,0,reference.stderr);
 const expected=JSON.parse(reference.stdout);
 const line=fs.readFileSync(process.argv[2],'utf8').split('\n').find(line=>line.includes('ASPECT_GEOMETRY '));assert(line,'Native geometry observation is missing');
 const actual=JSON.parse(line.slice(line.indexOf('ASPECT_GEOMETRY ')+16));assert.equal(actual.cases.length,expected.cases.length);
 for(const [i,box] of expected.cases.entries()){
  assert.equal(actual.cases[i].name,box.name);
  for(const axis of ['width','height'])assert(Math.abs(actual.cases[i][axis]-box[axis])<.02,JSON.stringify({box,actual:actual.cases[i]}));
 }
 console.log(JSON.stringify({case:'native-aspect-geometry',status:'pass',cases:expected.cases.length,reference:'independent Node after tsc',timed:false}));
} finally {fs.rmSync(root,{recursive:true,force:true});}
