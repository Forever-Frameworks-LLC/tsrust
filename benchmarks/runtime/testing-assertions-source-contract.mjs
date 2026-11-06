import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertTestingAssertionSource(source,workload,engine){
 if(!['testing_scalar_assertions','testing_expectation_predicates','testing_deep_data','testing_deep_collections'].includes(workload.id)||workload.family!=='testing-assertions')throw Error('Unknown assertion workload');
 const hash=createHash('sha256').update(source).digest('hex');
 if(hash!==workload[engine==='native'?'sourceSha256':'counterpartSha256'])throw Error('Assertion workload source fingerprint changed');
 for(const expression of [...workload.sourceRequired[engine],...workload.methodExpressions[engine]])if(!containsCodeSnippet(source, expression))throw Error('Assertion operation contract changed: '+expression);
 if(/function\s+(ok|strictEqual|notStrictEqual|deepStrictEqual|notDeepStrictEqual|expect)\b/.test(source))throw Error('Assertion APIs must not be replaced with stand-ins');
}

export const testingDeepReferenceOperations={
 testing_deep_data:['struct Row','left.key==right.key&&left.label==right.label','left.values.len()==right.values.len()','a.to_bits()==b.to_bits()||(a.is_nan()&&b.is_nan())','assert!(same(&left[at],&right[at]))','assert!(!same(&left[at],&right[(at+1)%8]))','checksum=(checksum*131+left[at].key)%1000000007'],
 testing_deep_collections:['use std::collections::{HashMap,HashSet}','left.len()==right.len()&&left.iter().all(|(key,value)|right.get(key)==Some(value))','assert!(same(&first,&second))','assert!(set_a==set_b)','assert!(!same(&first,&different))','checksum=(checksum*131+first[&at][0])%1000000007'],
};
export function assertTestingDeepReference(source,row){if(!Object.hasOwn(testingDeepReferenceOperations,row.id))return;if(row.family!=='testing-assertions')throw Error('Deep assertion family changed');if(createHash('sha256').update(source).digest('hex')!==row.rustReferenceSha256)throw Error('Deep assertion reference fingerprint changed');for(const operation of ['fn function0(value0: i64, value1: i64) -> i64','for i in 0..value0','(i*13+value1)%8',...testingDeepReferenceOperations[row.id]])if(!containsCodeSnippet(source,operation))throw Error('Deep assertion reference operation changed: '+operation);if(!/checksum\s*}\s*fn main\(\)\s*{\s*}/.test(source))throw Error('Deep assertion reference must return its observed checksum');}
