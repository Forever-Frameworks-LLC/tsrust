import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
const shared=['first = call(holder);','stepped = step.next();','both = pair.left.run() + call(pair.right);','replace(holder, value + 7);','replaced = holder.run();','holder.run = (): int => base[1] * 2;','const run = holder.run;','moved = run();','base[0] = 0;','[first, stepped, both, replaced, moved, count, base[0] + base[1]]','return output;'];
const native=['run: ReadOnly<Function<int>>;','next: Mutable<Function<int>>;','function call(holder: ReadOnly<Holder>): int','function replace(holder: Mutable<Holder>, amount: int): void'];
const typescript=['type Holder = { run: () => int; };','type Step = { next: () => int; };','function call(holder: Holder): int','function replace(holder: Holder, amount: int): void'];
export function assertScopedCallableRecordsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='scoped_callable_records'||row.family!=='scoped-callable-records')throw Error('Scoped callable record family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Scoped callable record fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete scoped callable record scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Scoped callable record denominator');
 for(const text of [...shared,...(kind==='native'?native:typescript)])if(!containsCodeSnippet(source, text))throw Error('Scoped callable record scenario: '+text);
 if((source.match(/step\.next\(\);/g)||[]).length!==2)throw Error('Scoped callable record repeated mutable call');
 if(source.indexOf('base[0] = 0;')<source.indexOf('moved = run();'))throw Error('Scoped callable record release order');
}
