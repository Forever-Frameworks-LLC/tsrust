import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertNestedDefaultSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.id!=='nested_function_defaults'||row.family!=='nested-function-defaults')throw Error('Named default family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Named default fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One captured optional/default callable scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Named default denominator');
 const scalar=kind==='native'?'int':'number';
 for(const text of [`function next(step: ${scalar} = defaultStep()): ${scalar} { total += step; return total; }`,`function optional(delta?: ${scalar}): ${scalar} { return value + (delta ?? 4); }`,'hits += 1; return 2;','const action = next;','first = action();','second = action(3);','third = action(undefined);','function take(text: string = saved): string { return text; }','const text = once();','output.push([first, second, third, hits, maybe(), maybe(1), text.length, input.length]);'])if(!containsCodeSnippet(source, text))throw Error('Named default scenario: '+text);
}
