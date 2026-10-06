import {createHash} from 'node:crypto';
export function assertNestedOwnedSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.id!=='nested_function_owned_escape'||row.family!=='nested-function-owned-escape')throw Error('Owned nested family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Owned nested fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One retained nested owner scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Owned nested denominator');
 for(const text of ['const actions = [make(value), make(value + 10)];','const again = actions[0]();','const recurse = recursive([value]);','return base[0];','return base[0] + 1;','const take = owner(value + 20);','const item = take();','try { fail(); } catch { caught = 1; }','output.push([first, second, again, recurse(4), recurse(3), item.value, caught, input.length]);','return even;','return take;','return fail;'])if(!source.includes(text))throw Error('Owned nested scenario: '+text);
}
