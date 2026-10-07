import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertTestingAssertionSource(source,workload,engine){
 if(!['testing_scalar_assertions','testing_expectation_predicates'].includes(workload.id)||workload.family!=='testing-assertions')throw Error('Unknown assertion workload');
 const hash=createHash('sha256').update(source).digest('hex');
 if(hash!==workload[engine==='native'?'sourceSha256':'counterpartSha256'])throw Error('Assertion workload source fingerprint changed');
 for(const expression of [...workload.sourceRequired[engine],...workload.methodExpressions[engine]])if(!containsCodeSnippet(source, expression))throw Error('Assertion operation contract changed: '+expression);
 if(/function\s+(ok|strictEqual|notStrictEqual|expect)\b/.test(source))throw Error('Assertion APIs must not be replaced with stand-ins');
}
