import assert from 'node:assert/strict';
import {containsCodeSnippet,findCodeSnippet} from './source-contract-tokens.mjs';
export const osIdentityIds=['std_os_platform','std_os_arch','std_os_endianness'];
// Independent process binary metadata: no calls to the tested os bindings.
export function osIdentityValue(id) { assert(osIdentityIds.includes(id)); return id==='std_os_platform'?process.platform:id==='std_os_arch'?process.arch:(new Uint8Array(new Uint16Array([1]).buffer)[0]===1?'LE':'BE'); }
export function assertOsIdentitySource(source,workload) {
 const method=workload.id.replace('std_os_',''); assert(osIdentityIds.includes(workload.id));
 assert(containsCodeSnippet(source,`import { ${method} } from "std:os";`) || containsCodeSnippet(source,`const ${method} = require("node:os").${method};`));
 for(const snippet of ['const values: string[] = [];','let i = 0;','while (i < n)',`values.push(${method}());`,'i = i + 1;','return values;']) assert(containsCodeSnippet(source,snippet),snippet);
 assert.equal(findCodeSnippet(source,`values.push(${method}());`).length,1);
}
export function osIdentityChecksum(id,n,seed) {
 let digest=0; for(let i=0;i<n;i++)for(const code of [...Buffer.from(osIdentityValue(id)),255])digest=(digest*131+code)%1000000007; return digest;
}
