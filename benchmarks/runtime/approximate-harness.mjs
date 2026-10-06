// Source-only integration draft. Native helper ABI/probe execution and all
// wrappers still require real acceptance; these signature checks are fail-closed.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {validationProtocol,validateReferenceBundle,APPROXIMATE_SCALAR_PROTOCOL} from './approximate-scalar.mjs';
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const sha=/^[0-9a-f]{64}$/;
export const approximateAdapterSources=Object.fromEntries(['approximate-driver.rs','approximate-driver.ts','approximate-allocation-driver.rs'].map(name=>[name,fs.readFileSync(new URL(name,import.meta.url),'utf8')]));
export function isApproximate(workload){return validationProtocol(workload)===APPROXIMATE_SCALAR_PROTOCOL;}
function artifact(directory,digest){
 if(!sha.test(digest))throw Error('Approximate artifact SHA256 required');
 const filename=path.join(directory,digest+'.json');
 const stat=fs.statSync(filename);if(!stat.isFile()||stat.size<1||stat.size>16*1024*1024)throw Error('Approximate artifact size/type');
 const bytes=fs.readFileSync(filename);if(hash(bytes)!==digest)throw Error('Approximate artifact identity');return {filename,bytes,sha256:digest};
}
export function loadApproximateReference(workload,source,{directory=new URL('./approximate-references/',import.meta.url)}={}){
 if(!isApproximate(workload))throw Error('Explicit approximate workload required');
 if(workload.approximateValidation.referencePending===true)throw Error('Approximate reference generation pending');
 const dir=directory instanceof URL?directory:path.resolve(directory);
 // path.join requires a path, not a URL. Convert the default only here.
 const location=dir instanceof URL?fileURLToPath(dir):dir;
 const policy=artifact(location,workload.approximateValidation.policySha256);
 const policyDocument=JSON.parse(policy.bytes.toString('utf8')),approved=policyDocument.policyByWorkload?.[workload.id];
 if(!approved||!sha.test(approved.inputRecipeSha256))throw Error('Approved workload input recipe required');
 const recipe=artifact(location,approved.inputRecipeSha256),reference=artifact(location,workload.approximateValidation.referenceSha256);
 const validated=validateReferenceBundle(reference.bytes,workload,{approvedPolicyBytes:policy.bytes,inputRecipeBytes:recipe.bytes,tsSha256:source.tsSha256,tsrSha256:source.tsrSha256});
 return {reference:validated,artifacts:[policy,recipe,reference],snapshot:{policy:policy.bytes.toString('utf8'),recipe:recipe.bytes.toString('utf8'),reference:reference.bytes.toString('utf8')}};
}
export function restoreApproximateReference(workload,source,snapshot){
 if(!snapshot||typeof snapshot.policy!=='string'||typeof snapshot.recipe!=='string'||typeof snapshot.reference!=='string')throw Error('Frozen approximate reference snapshot required');
 return validateReferenceBundle(Buffer.from(snapshot.reference),workload,{approvedPolicyBytes:Buffer.from(snapshot.policy),inputRecipeBytes:Buffer.from(snapshot.recipe),tsSha256:source.tsSha256,tsrSha256:source.tsrSha256});
}
export function assertApproximateArtifactsUnchanged(loaded){for(const item of loaded.artifacts)if(hash(fs.readFileSync(item.filename))!==item.sha256)throw Error('Approximate reference changed during run');}
export function assertApproximateCases(reference,cases){for(const [n,seed]of cases)if(!reference.cases.has(n+':'+seed))throw Error('Approximate case missing from immutable reference');}
export function approximatePointProbeSources(emitted,ts,workload,reference){
 if(!isApproximate(workload))throw Error('Explicit approximate probe required');
 const arity=workload.approximateValidation.operationArity;
 const native=`fn function1(${Array.from({length:arity},(_,i)=>`value${i}: f64`).join(', ')}) -> f64`;
 if(!emitted.includes(native))throw Error('Approximate native operation helper ABI changed');
 const signature=`function operation(${Array.from({length:arity},(_,i)=>`argument${i}: number`).join(', ')}): number`;
 if(!ts.includes(signature))throw Error('Approximate TypeScript operation helper ABI changed');
 if(!emitted.includes('fn main()'))throw Error('Approximate native entrypoint changed');
 const calls=reference.bundle.points.map(point=>`function1(${point.inputBits.map(bits=>`f64::from_bits(0x${bits}_u64)`).join(', ')})`);
 const rs=emitted.replace('fn main()','fn source_main()')+`
fn main(){
 let values=[${calls.join(',')}];
 let point_bits:Vec<std::string::String>=values.into_iter().map(|v|format!("{:016x}",v.to_bits())).collect();
 println!(r#"{{"protocol":"mpfr-directed-f64-scalar-v1","pointBits":{:?}}}"#,point_bits);
}
`;
 const inputs=JSON.stringify(reference.bundle.points.map(point=>point.inputBits));
 const wrapper=`\nconst probeView=new DataView(new ArrayBuffer(8));\nfunction probeFromBits(raw:string):number{probeView.setBigUint64(0,BigInt('0x'+raw),false);return probeView.getFloat64(0,false);}\nfunction probeToBits(value:number):string{probeView.setFloat64(0,value,false);return probeView.getBigUint64(0,false).toString(16).padStart(16,'0');}\nconst pointBits=(${inputs} as string[][]).map(point=>probeToBits(operation(${Array.from({length:arity},(_,i)=>`probeFromBits(point[${i}])`).join(', ')})));\nconsole.log(JSON.stringify({protocol:'mpfr-directed-f64-scalar-v1',pointBits}));\n`;
 return {rs,ts:ts+wrapper,abi:{nativeSignature:native,tsSignature:signature,verification:'emitted/source signature checks; actual compiled ABI and native probe execution required'}};
}
