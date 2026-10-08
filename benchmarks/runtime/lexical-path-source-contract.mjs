import assert from 'node:assert/strict';
import {containsCodeSnippet,findCodeSnippet} from './source-contract-tokens.mjs';
export const lexicalPathCases = {
    std_path_dirname_ascii: {method:'dirname',odd:'/tmp/report.txt',even:'/var/log/archive.tar.gz/',oddResult:'/tmp',evenResult:'/var/log'},
    std_path_extname_ascii: {method:'extname',odd:'/tmp/report.txt',even:'/var/log/archive.tar.gz/',oddResult:'.txt',evenResult:'.gz'},
    std_path_is_absolute_ascii: {method:'isAbsolute',odd:'/tmp/report.txt',even:'reports/../archive/',oddResult:'true',evenResult:'false'},
    std_path_normalize_ascii: {method:'normalize',odd:'/tmp//reports/../report.txt',even:'reports/../archive/',oddResult:'/tmp/report.txt',evenResult:'archive/'},
};
export const lexicalPathIds=Object.keys(lexicalPathCases);
export function assertLexicalPathSource(source, workload) {
    const c=lexicalPathCases[workload.id];
    assert(c, 'Unknown lexical path workload');
    assert(containsCodeSnippet(source, `import { ${c.method} } from "std:path";`) || containsCodeSnippet(source, `const ${c.method} = require("node:path").posix.${c.method};`), `${workload.id}: wrong path binding`);
    const call=c.method+'(path)'+(c.method==='isAbsolute'?'.toString()':'');
    assert.equal(findCodeSnippet(source, `values.push(${call});`).length, 1, 'Exactly one path operation per iteration');
    for (const snippet of [`let path = "${c.odd}";`, 'if ((i + seed) % 2 === 0)', `path = "${c.even}";`, `values.push(${call});`, 'i = i + 1;', 'return values;']) {
        assert(containsCodeSnippet(source, snippet), `${workload.id}: missing ${snippet}`);
    }
}
export function lexicalPathChecksum(id, n, seed) {
    const c=lexicalPathCases[id];
    assert(c, 'Unknown lexical path oracle');
    let digest = 0;
    for (let i = 0; i < n; i++) {
        const text=(i+seed)%2===0?c.evenResult:c.oddResult;
        for (const code of [...Buffer.from(text), 255]) digest = (digest * 131 + code) % 1000000007;
    }
    return digest;
}
