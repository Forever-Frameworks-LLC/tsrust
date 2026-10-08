import assert from 'node:assert/strict';
import {containsCodeSnippet,findCodeSnippet} from './source-contract-tokens.mjs';
export const pathRelativeIds=['std_path_relative'];
export function pathRelativeValues(id,n,seed) {
 assert(pathRelativeIds.includes(id)); const slash=process.platform==='win32'?'\\':'/';
 return Array.from({length:n},(_,i)=>(i+seed)%2===0?'..'+slash+'archive.txt':'2026');
}
export function pathRelativeChecksum(id,n,seed) {let digest=0;for(const text of pathRelativeValues(id,n,seed))for(const scalar of [...text].map(character=>character.codePointAt(0)).concat(255))digest=(digest*131+scalar)%1000000007;return digest;}
export function assertPathRelativeSource(source,workload) {
 assert(pathRelativeIds.includes(workload.id));
 assert(containsCodeSnippet(source,'import { relative } from "std:path";')||containsCodeSnippet(source,'const relative = require("node:path").relative;'));
 for(const snippet of ['const values: string[] = [];','const from = "reports/é😊";','let to = "reports/é😊/2026";','if ((i + seed) % 2 === 0)','to = "reports/archive.txt";','let i = 0;','while (i < n)','i = i + 1;','return values;','values.push(relative(from, to));'])assert(containsCodeSnippet(source,snippet),snippet);
 assert.equal(findCodeSnippet(source,'values.push(relative(from, to));').length,1);
}
