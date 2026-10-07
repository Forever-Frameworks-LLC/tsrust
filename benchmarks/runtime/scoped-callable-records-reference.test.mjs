import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {execFileSync} from 'node:child_process';

const here=path.dirname(new URL(import.meta.url).pathname);
const source=fs.readFileSync(path.resolve(here,'../../crates/tsrust-compiler/tests/scoped_callable_records.rs'),'utf8');
const cases=[...source.matchAll(/project\(\s*r#"([\s\S]*?)"#\s*,?\s*\);/g)].map(match=>match[1]);
test('scoped record source assertions also pass after TypeScript compilation',()=>{
    assert.equal(cases.length,9);
    const temporary=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-scoped-reference-'));
    try {
        for(const [index,body] of cases.entries()) {
            const rewritten=body.replace('import { strictEqual } from "std:testing";',`declare function require(name: string): {strictEqual: (actual: unknown, expected: unknown) => void};
const {strictEqual}=require('node:assert/strict');
type Function<Result, Arguments extends unknown[] = []> = (...inputs: Arguments) => Result;
type ReadOnly<T> = T;
type Mutable<T> = T;`).replace(/\bint\b/g,'number');
            const file=path.join(temporary,`case${index}.ts`);
            fs.writeFileSync(file,rewritten+'\nmain();\nexport {};\n');
        }
        execFileSync(process.execPath,[path.join(here,'node_modules/typescript/bin/tsc'),'--target','ES2022','--lib','ES2023,DOM','--module','commonjs','--strict','--skipLibCheck','--outDir',temporary,...cases.map((_,index)=>path.join(temporary,`case${index}.ts`))],{stdio:'pipe'});
        for(const index of cases.keys()) assert.equal(execFileSync(process.execPath,[path.join(temporary,`case${index}.js`)],{encoding:'utf8'}),'');
    } catch(error) {throw new Error(String(error.stdout||'')+String(error.stderr||'')+'\n'+error.message,{cause:error});}
    finally {fs.rmSync(temporary,{recursive:true,force:true});}
});
