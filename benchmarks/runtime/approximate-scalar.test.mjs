// Source-only, UNEXECUTED. These are protocol/binding fixtures, not accepted
// fabricated numerical references. Positive numerical cases need real MPFR files.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {validationProtocol,approvedPolicyForWorkload,validateReferenceBundle,validateApproximateMeasurement,validateApproximateAllocation,APPROXIMATE_SCALAR_PROTOCOL} from './approximate-scalar.mjs';
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const bytes=value=>Buffer.from(JSON.stringify(value));
const copy=value=>JSON.parse(JSON.stringify(value));
const approvedPolicyBytes=fs.readFileSync(new URL('./approved-policy.fixture.json',import.meta.url));
const inputRecipeBytes=fs.readFileSync(new URL('./input-recipes.fixture.json',import.meta.url));
const policy=JSON.parse(approvedPolicyBytes),id='math_sin',entry=policy.policyByWorkload[id];
const row={id,result:'number',resultEncoding:'approximate-f64-scalar-v1',approximateValidation:{protocol:APPROXIMATE_SCALAR_PROTOCOL,method:entry.method,operationArity:entry.operationArity,policySha256:hash(approvedPolicyBytes),referencePending:true}};
const context={approvedPolicyBytes,inputRecipeBytes,tsSha256:entry.tsSha256,tsrSha256:entry.tsrSha256};
for(const exact of [{result:'int'},{result:'number'},{result:'number[]'},{result:'string[][]',resultEncoding:'array-shape-scalar-v1'},{result:'number[][]',resultEncoding:'ieee-f64-tree-v1'}])assert.equal(validationProtocol(exact),'legacy-exact');
for(const invalid of [{result:'number',resultEncoding:'unknown'},{result:'number',approximateValidation:{}},{result:'number[]',resultEncoding:'approximate-f64-scalar-v1',approximateValidation:{protocol:APPROXIMATE_SCALAR_PROTOCOL}},{result:'number',resultEncoding:'approximate-f64-scalar-v1'},{result:'number',resultEncoding:'approximate-f64-scalar-v1',approximateValidation:{protocol:'typo'}}])assert.throws(()=>validationProtocol(invalid),/Approximate scalar protocol/);
assert.deepEqual(approvedPolicyForWorkload(row,context),entry); // Policy only, no numeric acceptance.
assert.throws(()=>approvedPolicyForWorkload(row,{...context,approvedPolicyBytes:bytes({...policy,predeclared:false})}),/artifact identity mismatch/);
assert.throws(()=>approvedPolicyForWorkload(row,{...context,inputRecipeBytes:bytes({})}),/recipe artifact identity mismatch/);
assert.throws(()=>approvedPolicyForWorkload(row,{...context,tsSha256:'0'.repeat(64)}),/source identity/);
for(const change of [p=>{p.predeclared=false;},p=>{p.policyByWorkload[id].policy.predeclared=false;},p=>{p.policyByWorkload[id].policy.maxOperationRankSteps=64;},p=>{p.policyByWorkload[id].operationArity=2;},p=>{p.policyByWorkload[id].method='cos';}]){
 const modified=copy(policy);change(modified);const modifiedBytes=bytes(modified),modifiedRow=copy(row);modifiedRow.approximateValidation.policySha256=hash(modifiedBytes);
 assert.throws(()=>approvedPolicyForWorkload(modifiedRow,{...context,approvedPolicyBytes:modifiedBytes}),/approved|arity|method/);
}
for(const change of [p=>{p.operationArity=3;},p=>{p.policy.maxOperationRankSteps=8;},p=>{p.policy.maxOperationRankSteps=64;}]){
 const modified=copy(policy);change(modified.policyByWorkload.math_hypot);const modifiedBytes=bytes(modified),h=modified.policyByWorkload.math_hypot;
 const hrow={...row,id:'math_hypot',approximateValidation:{...row.approximateValidation,method:'hypot',operationArity:h.operationArity,policySha256:hash(modifiedBytes)}};
 assert.throws(()=>approvedPolicyForWorkload(hrow,{...context,approvedPolicyBytes:modifiedBytes,tsSha256:h.tsSha256,tsrSha256:h.tsrSha256}),/arity|8\/32/);
}
assert.throws(()=>validateReferenceBundle(bytes({}),row,context),/generation pending/);
// Intentionally incomplete binding-only JSON: rejected before provenance/points.
// It is never a passing directed reference, and has no numerical result fixture.
const incomplete={schemaVersion:1,protocol:APPROXIMATE_SCALAR_PROTOCOL,workloadId:id,method:entry.method,operationArity:entry.operationArity,inputRecipeSha256:entry.inputRecipeSha256,argumentBounds:entry.argumentBounds,tsSha256:entry.tsSha256,tsrSha256:entry.tsrSha256,policySha256:hash(approvedPolicyBytes),policy:entry.policy,domain:entry.domain};
for(const change of [b=>{b.policy.maxOperationRankSteps=64;},b=>{b.policy.predeclared=false;},b=>{b.domain.maxSeed++;},b=>{b.operationArity=2;},b=>{b.argumentBounds[0].upper.numerator='100';},b=>{b.method='cos';}]){
 const invalid=copy(incomplete);change(invalid);const referenceBytes=bytes(invalid),actualRow=copy(row);actualRow.approximateValidation.referencePending=false;actualRow.approximateValidation.referenceSha256=hash(referenceBytes);
 assert.throws(()=>validateReferenceBundle(referenceBytes,actualRow,context),/actual approved policy/);
}
assert.throws(()=>validateApproximateMeasurement({}, {},{pointProbeAccepted:false}),/point probe/);
assert.throws(()=>validateApproximateAllocation({}, {},{acceptedNativePointProbe:undefined}),/native all-point probe/);
// After genuine fixture generation: clone actual reference bytes and recompute
// their test-only hash while tampering point ID/inputBits, case initialBits or
// swapping two step IDs. Each must reject approved recipe/order before numeric
// certificate checks. Also test disposal!=0, injected durations/checksum and
// per-output bounds against actual generated cases; no fake MPFR fixtures here.
