import assert from 'node:assert/strict';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
export const lexicalPathIds = ['std_path_dirname_ascii', 'std_path_extname_ascii'];
export function assertLexicalPathSource(source, workload) {
    const method = workload.id.split('_')[2];
    for (const snippet of ['let path = "/tmp/report.txt";', 'if ((i + seed) % 2 === 0)', 'path = "/var/log/archive.tar.gz/";', `values.push(${method}(path));`, 'i = i + 1;', 'return values;']) {
        assert(containsCodeSnippet(source, snippet), `${workload.id}: missing ${snippet}`);
    }
}
export function lexicalPathChecksum(id, n, seed) {
    let digest = 0;
    for (let i = 0; i < n; i++) {
        const even = (i + seed) % 2 === 0;
        const text = id === 'std_path_dirname_ascii' ? (even ? '/var/log' : '/tmp') : (even ? '.gz' : '.txt');
        for (const code of [...Buffer.from(text), 255]) digest = (digest * 131 + code) % 1000000007;
    }
    return digest;
}
