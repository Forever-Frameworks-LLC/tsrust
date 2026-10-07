import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {expectedGenericAdapterValues} from './generic-value-adapters-oracle.mjs';
import {assertGenericAdapterSource} from './generic-value-adapters-source-contract.mjs';

const here=path.dirname(new URL(import.meta.url).pathname);
const root=path.resolve(here,'../..');
test('consuming borrowed alternatives retain matched native and compiled TypeScript results',()=>{
    const manifest=JSON.parse(fs.readFileSync(path.join(here,'manifest.json'),'utf8'));
    const row=manifest.workloads.find(row=>row.id==='generic_value_adapters');
    const cases=[[0,0],[1,7],[2,19],[17,3],[row.iterations,manifest.seed]];
    const body=cases.map(([n,seed])=>`for(const row of workload(${n},${seed},"adapters")){console.log(row.join(","));}`).join('\n');
    const expected=cases.flatMap(([n,seed])=>expectedGenericAdapterValues(row.id,n,seed,'adapters').map(values=>values.join(','))).join('\n')+'\n';
    const temporary=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-consuming-borrowed-'));
    try {
        for(const [kind,suffix] of [['native','tsr'],['typescript','ts']]) {
            const source=fs.readFileSync(path.join(here,'workloads/generic_value_adapters.'+suffix),'utf8');
            assertGenericAdapterSource(source,row,kind);
            const modified=source.replace(/function main\(\)\s*\{\s*\}/,`function main(){${body}}`);
            assert.notEqual(source,modified);
            if(kind==='native') {
                const file=path.join(temporary,'main.tsr');fs.writeFileSync(file,modified);
                const compiler=process.env.TSRUST||path.join(root,'target/editor/debug/tsrust');
                const formatted=execFileSync(compiler,['format',file],{encoding:'utf8'});
                fs.writeFileSync(file,formatted);
                assert.equal(execFileSync(compiler,['format',file],{encoding:'utf8'}),formatted);
                const rust=execFileSync(compiler,['emit',file],{encoding:'utf8'});
                const rustFile=path.join(temporary,'main.rs');fs.writeFileSync(rustFile,rust);
                const executable=path.join(temporary,'native');
                execFileSync('rustc',['--edition=2024','-A','warnings',rustFile,'-o',executable],{stdio:'pipe'});
                assert.equal(execFileSync(executable,[],{encoding:'utf8',maxBuffer:16*1024*1024}),expected);
            } else {
                const file=path.join(temporary,'main.ts');fs.writeFileSync(file,modified+'\nmain();\n');
                execFileSync(process.execPath,[path.join(here,'node_modules/typescript/bin/tsc'),'--target','ES2022','--lib','ES2023,DOM','--module','commonjs','--strict','--skipLibCheck','--outDir',temporary,file],{stdio:'pipe'});
                assert.equal(execFileSync(process.execPath,[path.join(temporary,'main.js')],{encoding:'utf8',maxBuffer:16*1024*1024}),expected);
            }
        }
    } catch(error) {throw new Error(String(error.stdout||'')+String(error.stderr||'')+'\n'+error.message,{cause:error});} finally {fs.rmSync(temporary,{recursive:true,force:true});}
});
