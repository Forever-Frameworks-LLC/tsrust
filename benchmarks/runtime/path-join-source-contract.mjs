import assert from 'node:assert/strict';
import {containsCodeSnippet,findCodeSnippet} from './source-contract-tokens.mjs';
export const pathJoinIds=['std_path_join_ascii','std_path_join_empty_roots','std_path_join_many_unicode'];
export function pathJoinValues(id,n,seed) {
 assert(pathJoinIds.includes(id)); const slash=process.platform==='win32'?'\\':'/';
 return Array.from({length:n},(_,i)=>id==='std_path_join_ascii'?((i+seed)%2===0?slash+'archive.txt':slash+'tmp'+slash+'reports'+slash+'archive.txt'):id==='std_path_join_empty_roots'?((i+seed)%2===0?slash:'.'):'é😊'+slash+('文'+slash).repeat(63)+'résumé'+slash);
}
export function pathJoinChecksum(id,n,seed) {let digest=0;for(const text of pathJoinValues(id,n,seed))for(const scalar of [...text].map(character=>character.codePointAt(0)).concat(255))digest=(digest*131+scalar)%1000000007;return digest;}
const cases={
 std_path_join_ascii:['let part = "reports";','if ((i + seed) % 2 === 0)','part = "..";','values.push(join("/tmp", part, "archive.txt"));'],
 std_path_join_empty_roots:['let first = "";','if ((i + seed) % 2 === 0)','first = "/";','values.push(join(first, "", "child", "..", ""));'],
 std_path_join_many_unicode:['const paths: string[] = [];','paths.push("é😊");','while (j < 64)','paths.push("文");','j = j + 1;','paths.push("..");','paths.push("résumé/");','values.push(join(...paths));'],
};
export function assertPathJoinSource(source,workload) {
 assert(pathJoinIds.includes(workload.id));
 assert(containsCodeSnippet(source,'import { join } from "std:path";')||containsCodeSnippet(source,'const join = require("node:path").join;'));
 for(const snippet of ['const values: string[] = [];','let i = 0;','while (i < n)','i = i + 1;','return values;',...cases[workload.id]])assert(containsCodeSnippet(source,snippet),snippet);
 assert.equal(findCodeSnippet(source,cases[workload.id].at(-1)).length,1);
}
