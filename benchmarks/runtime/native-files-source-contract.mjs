import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {nativeFileIDs} from './native-files-oracle.mjs';
export function assertNativeFilesSource(source,row,kind) {
  if(!nativeFileIDs.includes(row.id)||row.family!=='native-files'||!['native','typescript'].includes(kind))throw Error('Native file family');
  if(createHash('sha256').update(source).digest('hex')!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('Native file fingerprint');
  if(row.id==='native_file_full_write_read') {
    if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One bounded full-file write and readback with byte observation'||row.resultEncoding!=='array-shape-scalar-v1'||row.inputText!=='native-file-fixture-v1')throw Error('Full file denominator');
    const required=['const count=(i+seed)%64+1;','while(i<n)','output.push(sample(input,count));','const destination=input+".out";'];
    if(kind==='native')required.push('writeFile(destination,bytes)','readFile(destination,count+1)','view(owner,0,count)','fromUtf8("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")','sum+=value;','return [length(bytes),sum];');
    else required.push('fileSystem.writeFileSync(destination,bytes)','fileSystem.openSync(destination,"r")','new TextEncoder().encode("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")','owner.subarray(0,count)','fileSystem.readSync(file,readback,size,limit-size+1,null)','if(size>limit)','if(got===0)','sum+=readback[index];','return [size,sum];','fileSystem.closeSync(file);');
    for(const snippet of required)if(!containsCodeSnippet(source,snippet))throw Error(`Full file source contract: ${snippet}`);
    return;
  }
  const writing=['native_file_write','native_file_write_all'].includes(row.id);
  const unit=writing?'One create/truncate, bounded byte-view write, close, readback observation and release':'One open, bounded cursor read, byte observation and file release';
  if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!==unit||row.resultEncoding!=='array-shape-scalar-v1'||row.inputText!=='native-file-fixture-v1')throw Error('Native file denominator');
  const required=['const count = (i + seed) % 64 + 1;','while (i < n)','output.push(sample(input, count));'];
  if(kind==='native') {
    required.push(writing?'match (open(destination))':'match (open(input))','match (read(file, count))','match (at(bytes, index))','sum += value;','return [length(bytes), sum];',writing?'const observed = observe(file, count + 1);':'const observed = observe(file, count);','return observed;');
    if(writing || row.id==='native_file_read_close')required.push('match (close(file))');
    else if(containsCodeSnippet(source,'close(file)'))throw Error('Scope-drop workload closes explicitly');
  } else required.push(writing?'fileSystem.openSync(destination, "r")':'fileSystem.openSync(input, "r")','fileSystem.readSync(file, bytes, 0, count, null)','finally { fileSystem.closeSync(file); }','sum += bytes[index];','return [size, sum];');
  if(writing) {
    required.push('const destination = input + ".out";');
    if(kind==='native') required.push('create(destination)','fromUtf8("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")','view(owner, 0, count)','close(output)',row.id==='native_file_write'?'write(output, bytes)':'writeAll(output, bytes)');
    else required.push('return observe(file, count + 1);','fileSystem.openSync(destination, "w")','new TextEncoder().encode("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")','owner.subarray(0, count)','fileSystem.closeSync(output)',row.id==='native_file_write'?'fileSystem.writeSync(output, bytes, 0, count, null)':'fileSystem.writeSync(output, bytes, written, count - written, null)');
    if(row.id==='native_file_write') required.push(kind==='native'?'if (written !== count)':'if (written !== count)');
    else if(kind==='typescript') required.push('while (written < count)','if (size === 0)','written += size;');
  }
  for(const snippet of required)if(!containsCodeSnippet(source,snippet))throw Error(`Native file source contract: ${snippet}`);
}
