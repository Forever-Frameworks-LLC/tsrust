import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {compilerSourceInputs} from './compiler-source-inputs.mjs';
test('compiler identities include nested native backends and checked library sources',()=>{
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-source-inputs-'));
 try{
  fs.mkdirSync(path.join(directory,'stdlib/testing'),{recursive:true});
  fs.writeFileSync(path.join(directory,'lib.rs'),'mod stdlib;');
  fs.writeFileSync(path.join(directory,'stdlib/testing/runtime.rs'),'native backend');
  const surface=path.join(directory,'stdlib/testing/assertions.tsr');fs.writeFileSync(surface,'checked surface');
  const first=compilerSourceInputs(directory);
  assert.deepEqual(first.map(([name])=>name),['lib.rs','stdlib/testing/assertions.tsr','stdlib/testing/runtime.rs']);
  fs.writeFileSync(surface,'changed surface');const second=compilerSourceInputs(directory);
  assert.notEqual(first[1][1],second[1][1]);assert.equal(first[0][1],second[0][1]);assert.equal(first[2][1],second[2][1]);
 }finally{fs.rmSync(directory,{recursive:true,force:true});}
});
