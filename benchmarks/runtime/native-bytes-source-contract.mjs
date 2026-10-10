import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';

export function assertNativeBytesSource(source, row, kind) {
    if (!['native', 'typescript'].includes(kind) || row.id !== 'native_bytes' || row.family !== 'native-bytes') throw Error('Native byte family');
    if (createHash('sha256').update(source).digest('hex') !== (kind === 'native' ? row.sourceSha256 : row.counterpartSha256)) throw Error('Native byte fingerprint');
    if (row.iterations !== 2048 || row.operationsPerIteration !== 1 || row.operationUnit !== 'One byte construction and validation scenario' || row.resultEncoding !== 'array-shape-scalar-v1') throw Error('Native byte denominator');
    const shared = ['const construct = fromArray;', 'const observe = length;', 'const byte = (i + seed) % 256;', 'const values = [0, byte, 255];', 'const invalid = i % 2 === 0 ? -1 : 256;', 'output.push([first, second, values[1], empty, offset, codeLength]);', 'return output;'];
    const native = ['import { fromArray, length } from "std:bytes";', 'match (construct(values))', 'first = observe(bytes); second = observe(bytes);', 'match (fromArray([]))', 'match (fromArray([byte, invalid, 0]))', 'offset = error.offset ?? -3; codeLength = error.code.length;'];
    const typescript = ['Uint8Array.from(values)', 'Number.isInteger(values[offset])', 'values[offset] < 0 || values[offset] > 255', 'const result = construct(values);', 'first = observe(result.value); second = observe(result.value);', 'const emptyResult = fromArray([]);', 'const invalidResult = fromArray([byte, invalid, 0]);', 'offset = invalidResult.error.offset ?? -3; codeLength = invalidResult.error.code.length;'];
    for (const text of [...shared, ...(kind === 'native' ? native : typescript)]) if (!containsCodeSnippet(source, text)) throw Error('Native byte scenario: ' + text);
}
