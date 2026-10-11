import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {nativeFileIDs} from './native-files-oracle.mjs';
export function assertNativeFilesSource(source,row,kind) {
  if(!nativeFileIDs.includes(row.id)||row.family!=='native-files'||!['native','typescript'].includes(kind))throw Error('Native file family');
  if(createHash('sha256').update(source).digest('hex')!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('Native file fingerprint');
  if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One open, bounded cursor read, byte observation and file release'||row.resultEncoding!=='array-shape-scalar-v1'||row.inputText!=='native-file-fixture-v1')throw Error('Native file denominator');
  const required=['const count = (i + seed) % 64 + 1;','while (i < n)','output.push(sample(input, count));'];
  if(kind==='native') {
    required.push('match (open(input))','match (read(file, count))','match (at(bytes, index))','sum += value;','return [length(bytes), sum];','const observed = observe(file, count);','return observed;');
    if(row.id==='native_file_read_close')required.push('match (close(file))');
    else if(containsCodeSnippet(source,'close(file)'))throw Error('Scope-drop workload closes explicitly');
  } else required.push('fileSystem.openSync(input, "r")','fileSystem.readSync(file, bytes, 0, count, null)','finally { fileSystem.closeSync(file); }','sum += bytes[index];','return [size, sum];');
  for(const snippet of required)if(!containsCodeSnippet(source,snippet))throw Error(`Native file source contract: ${snippet}`);
}
