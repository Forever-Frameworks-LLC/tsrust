import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';

const shared = ['const value = i % 17 + seed % 19 + 1;', 'value % 3 === 0', 'start % 4 === 0', 'const first = inspect(result);', 'const second = inspect(result);', 'value += 1; return value;', 'output.push([first, second, owned, callA, callB]);', 'return output;'];
const native = ['function inspect(result: ReadOnly<Result<User, int>>): int', 'return Err(value + 5);', 'return Ok({ value: value });', 'Ok(user) => { return user.value * 2; }', 'Err(error) => { return error * 3; }', 'Ok(user) => { owned = user.value + 7; }', 'Err(error) => { owned = error + 11; }', 'Ok(action) => { callA = action(); callB = action(); }', 'Err(error) => { callA = error; callB = error + 1; }'];
const typescript = ['function inspect(result: Result<User, int>): int', 'return { kind: "Err", error: value + 5 };', 'return { kind: "Ok", value: { value } };', 'case "Ok": return result.value.value * 2;', 'case "Err": return result.error * 3;', 'case "Ok": owned = result.value.value + 7; break;', 'case "Err": owned = result.error + 11; break;', 'case "Ok": callA = callback.value(); callB = callback.value(); break;', 'case "Err": callA = callback.error; callB = callback.error + 1; break;'];

export function assertResultMatchSource(source, row, kind) {
    if (!['native', 'typescript'].includes(kind) || row.id !== 'result_match' || row.family !== 'result-match') throw Error('Result match family');
    if (createHash('sha256').update(source).digest('hex') !== (kind === 'native' ? row.sourceSha256 : row.counterpartSha256)) throw Error('Result match fingerprint');
    if (row.iterations !== 2048 || row.operationsPerIteration !== 1 || row.operationUnit !== 'One Result ownership and callable scenario' || row.resultEncoding !== 'array-shape-scalar-v1') throw Error('Result match denominator');
    for (const text of [...shared, ...(kind === 'native' ? native : typescript)]) if (!containsCodeSnippet(source, text)) throw Error('Result match scenario: ' + text);
}
