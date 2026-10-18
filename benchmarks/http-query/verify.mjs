// Untimed process-boundary parity; no argument-throughput claim.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
const here=path.dirname(fileURLToPath(import.meta.url));
assert(process.env.TSRUST,'Set TSRUST to the candidate compiler');
function run(command,args){const result=spawnSync(command,args,{encoding:'utf8',timeout:120000});assert.equal(result.status,0,result.stderr||String(result.error));return result.stdout;}
const temporary=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-query-'));
try {
 const native=path.join(temporary,'arguments');
 run(process.env.TSRUST,['build',path.join(here,'query.tsr'),'-o',native]);
 run(process.execPath,[path.join(here,'../runtime/node_modules/typescript/bin/tsc'),'--target','es2022','--skipLibCheck','--outDir',temporary,path.join(here,'query.ts')]);
 for(const [engine,command,args] of [['native',native,[]],['node-tsc',process.execPath,[path.join(temporary,'query.js')]]]){
  assert.equal(run(command,args),'one two|é\ntrue\n0\na=b\n');
  console.log(JSON.stringify({engine,status:'pass',timed:false}));
 }
 for(const name of ['query.tsr','query.ts'])console.log(JSON.stringify({source:name,sha256:createHash('sha256').update(fs.readFileSync(path.join(here,name))).digest('hex')}));
} finally {fs.rmSync(temporary,{recursive:true,force:true});}
