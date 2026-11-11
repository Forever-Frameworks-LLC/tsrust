import test from 'node:test';
import assert from 'node:assert/strict';
import {compilerExecutable} from './compiler-executable.mjs';
test('Benchmark compiler follows the Cargo output directory',()=>{
 assert.equal(compilerExecutable('/repo','/repo/benchmarks/runtime',{},'darwin'),'/repo/target/release/tsrust');
 assert.equal(compilerExecutable('/repo','/repo/benchmarks/runtime',{CARGO_TARGET_DIR:'/private/tmp/warm'},'darwin'),'/private/tmp/warm/release/tsrust');
 assert.equal(compilerExecutable('/repo','/repo/benchmarks/runtime',{CARGO_TARGET_DIR:'../../build'},'darwin'),'/repo/build/release/tsrust');
 assert.equal(compilerExecutable('/repo','/repo/benchmarks/runtime',{},'win32'),'/repo/target/release/tsrust.exe');
});

test('Explicit TSRUST selects a real executable and rejects invalid overrides',()=>{
 assert.equal(compilerExecutable('/repo','/repo/benchmarks/runtime',{TSRUST:process.execPath}),process.execPath);
 assert.throws(()=>compilerExecutable('/repo','/repo/benchmarks/runtime',{TSRUST:'/nonexistent/tsrust-compiler-override'}));
 assert.throws(()=>compilerExecutable('/repo','/repo/benchmarks/runtime',{TSRUST:process.cwd()}),/must name a compiler executable/);
});
