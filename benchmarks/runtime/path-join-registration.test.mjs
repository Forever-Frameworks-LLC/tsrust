import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {assertABI} from './adapters.mjs';
import {assertPathJoinSource,pathJoinIds,pathJoinChecksum,pathJoinValues} from './path-join-source-contract.mjs';
import {expectedChecksum} from './oracle.mjs';
const require=createRequire(import.meta.url);
const ts=require('./node_modules/typescript');
const here=path.dirname(new URL(import.meta.url).pathname);
const root=path.resolve(here,'../..');
const compiler=process.env.TSRUST || path.join(root,'target/editor/debug/tsrust');
const manifest=JSON.parse(fs.readFileSync(path.join(here,'manifest.json')));
for(const id of pathJoinIds) test(id+' preserves source, ABI and independent results',()=>{
    const w=manifest.workloads.find(row=>row.id===id);
    const source=fs.readFileSync(path.join(here,w.source),'utf8');
    const counterpart=fs.readFileSync(path.join(here,w.counterpart),'utf8');
    for(const [text,hash] of [[source,w.sourceSha256],[counterpart,w.counterpartSha256]]){
        assert.equal(createHash('sha256').update(text).digest('hex'),hash);
        assertPathJoinSource(text,w);
        assert.throws(()=>assertPathJoinSource(text.replace('i = i + 1','i = i + 2'),w));
        assert.throws(()=>assertPathJoinSource(text.replace('std:path','std:fs').replace('node:path','node:fs'),w));
    }
    assertABI(execFileSync(compiler,['emit',path.join(here,w.source)],{encoding:'utf8'}),w,{source});
    const js=ts.transpileModule(counterpart,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}}).outputText;
    const evaluate=new Function('require','n','seed',js+';return workload(n,seed);');
    const dir=fs.mkdtempSync(path.join(root,'target/editor/path-join-'));
    try{for(const [n,seed] of [[0,7],[2,7],[17,0]]){

        const expected=pathJoinValues(id,n,seed);
        assert.deepEqual(evaluate(require,n,seed),expected);
        assert.equal(expectedChecksum(id,n,seed),pathJoinChecksum(id,n,seed));
        const file=path.join(dir,'main.tsr');
        fs.writeFileSync(file,source.slice(0,source.indexOf('function main()'))+`function main(){const values=workload(${n},${seed});for(const value of values){console.log(value);}}`);
        assert.equal(execFileSync(compiler,['run',file],{encoding:'utf8'}),expected.map(s=>s+'\n').join(''));
    }}finally{fs.rmSync(dir,{recursive:true,force:true});}
});
