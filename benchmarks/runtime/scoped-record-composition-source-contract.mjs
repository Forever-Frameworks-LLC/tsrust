import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
const shared=["const removed = items.pop();", "items.push({ run: (): int => base[1] });", "removedValue = removed.run();", "arrayValue = items[0].run();", "const boxes: Box[] = [{ handlers: handlers }];", "boxes[0].handlers.set(\"second\", { run: (): int => base[1] });", "mapFirst = first.run();", "mapSecond = second.run();", "populated = boxes[0].handlers.size;", "boxes[0].handlers.clear();", "cleared = boxes[0].handlers.size;", "base[0] = 0;", "[removedValue, arrayValue, mapFirst, mapSecond, replaced, iterated, populated, cleared, base[0] + base[1]]", "boxes[0].handlers = replacementMap;", "replaced = replacement.run();", "iterated += holder.run();", "return output;"];
export function assertScopedRecordCompositionSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.id!=='scoped_record_composition'||row.family!=='scoped-record-composition') throw Error('Scoped record composition family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256)) throw Error('Scoped record composition fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete scoped record composition scenario'||row.resultEncoding!=='array-shape-scalar-v1') throw Error('Scoped record composition denominator');
 for(const text of [...shared,kind==='native'?'run: ReadOnly<Function<int>>;':'run: () => int;']) if(!containsCodeSnippet(source,text)) throw Error('Scoped record composition scenario: '+text);
}
