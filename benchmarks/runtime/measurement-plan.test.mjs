import test from 'node:test';
import assert from 'node:assert/strict';
import {measurementPlan} from './measurement-plan.mjs';
const manifest={seed:7,startupSamples:11,processes:3,warmups:8,samples:31};
test('execution-only retains every independent validation case and all execution batches',()=>{
 const full=measurementPlan([],manifest), execution=measurementPlan(['--execution-only'],manifest);
 assert.deepEqual(execution.correctnessCases(20000),[[0,0],[1,7],[2,19],[17,3],[20000,7]]);
 assert.deepEqual(execution.correctnessCases(20000),full.correctnessCases(20000));
 assert.equal(execution.processes*execution.samples,93);
 assert.equal(execution.processes*execution.warmups,24);
 assert.equal(execution.processes,full.processes);assert.equal(execution.samples,full.samples);assert.equal(execution.warmups,full.warmups);
 assert.equal(execution.measurementScope,'execution-only');assert.equal(execution.startupSamples,0);assert.equal(execution.compilation,false);
 assert.equal(full.startupSamples,11);assert.equal(full.compilation,true);assert.equal(full.measurementScope,'runtime-startup-compilation');
});
test('verification has no measured batches and conflicting scope flags fail before preparation',()=>{
 const verify=measurementPlan(['--verify'],manifest);
 assert.equal(verify.correctnessCases(17).length,5);assert.equal(verify.processes,0);assert.equal(verify.samples,0);
 assert.equal(verify.startupSamples,0);assert.equal(verify.compilation,false);assert.equal(verify.measurementScope,'correctness-only');
 assert.throws(()=>measurementPlan(['--verify','--execution-only'],manifest),/cannot be combined/);
});
test('Node checkpoint subset preserves sample counts and explicitly identifies measured engines',()=>{
 const plan=measurementPlan(['--execution-only','--engines=node-tsc,native'],manifest);
 assert.deepEqual(plan.engineIDs,['native','node-tsc']);
 assert.equal(plan.engineIDs.length*plan.processes*plan.samples,186);
 assert.equal(plan.correctnessCases(2048).length*plan.engineIDs.length,10);
 assert.equal(measurementPlan([],manifest).engineIDs.length,8);
 for(const argv of [['--engines='],['--engines=missing'],['--engines=native,native'],['--engines=native','--engines=node-tsc']]) assert.throws(()=>measurementPlan(argv,manifest),/engines/);
});
test('unknown verification flags cannot silently start measurement',()=>{
 for(const flag of ['--verify-only','--verfy','--execution-onl']) {
  assert.throws(()=>measurementPlan([flag],manifest),/Unknown runtime option/);
 }
 const verify=measurementPlan(['--verify','--only=nested_function_rest','--engines=native,node-tsc'],manifest);
 assert.equal(verify.processes,0);assert.equal(verify.samples,0);
});
