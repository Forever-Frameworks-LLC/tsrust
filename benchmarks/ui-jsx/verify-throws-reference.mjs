import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const here=path.dirname(fileURLToPath(import.meta.url));
const root=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-throws-reference-'));
try {
 const runtime=path.resolve(here,'../runtime/node_modules');
 const result=spawnSync(process.execPath,[path.join(runtime,'typescript/bin/tsc'),'--target','es2022','--module','commonjs','--esModuleInterop','--lib','es2022,esnext.disposable','--skipLibCheck','--typeRoots',path.join(runtime,'@types'),'--outDir',root,path.join(here,'../../crates/tsrust-compiler/tests/fixtures/testing-throws/assertions.ts')],{encoding:'utf8',timeout:30000});assert.equal(result.status,0,result.stdout||result.stderr);
 const reference=spawnSync(process.execPath,[path.join(root,'assertions.js')],{encoding:'utf8',timeout:30000});assert.equal(reference.status,0,reference.stderr);assert.equal(reference.stdout,'throws passed\n');
 console.log(JSON.stringify({case:'throws-control-flow-reference',status:'pass',engine:'Node after tsc',checks:['callback throw','explicit disposal during throw','normal-return assertion failure'],timed:false}));
} finally {fs.rmSync(root,{recursive:true,force:true});}
