// Untimed JSX lowering parity, independent of any browser or UI runtime.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
const here=path.dirname(fileURLToPath(import.meta.url));
assert(process.env.TSRUST,'Set TSRUST to the candidate compiler');
function run(command,args){const result=spawnSync(command,args,{encoding:'utf8',timeout:120000});assert.equal(result.status,0,result.stderr||result.stdout||String(result.error));return result.stdout;}
const temporary=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-jsx-'));
const nativeSource=path.resolve(here,'../../examples/ui-jsx/main.tsrx');
try {
 const native=path.join(temporary,'jsx');
 run(process.env.TSRUST,['build',nativeSource,'-o',native]);
 run(process.execPath,[path.join(here,'../runtime/node_modules/typescript/bin/tsc'),'--target','es2022','--lib','es2022','--skipLibCheck','--jsx','react','--jsxFactory','element','--outDir',temporary,path.join(here,'components.tsx')]);
 const expected='Hello café|🦀 & friends\n';
 for(const [engine,command,args] of [['native',native,[]],['node-tsc',process.execPath,[path.join(temporary,'components.js')]]]) {
  assert.equal(run(command,args),expected);
  console.log(JSON.stringify({engine,status:'pass',timed:false}));
 }
 for(const source of [nativeSource,path.join(here,'components.tsx')])console.log(JSON.stringify({source:path.relative(here,source),sha256:createHash('sha256').update(fs.readFileSync(source)).digest('hex')}));
} finally {fs.rmSync(temporary,{recursive:true,force:true});}
