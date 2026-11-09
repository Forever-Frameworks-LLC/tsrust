import {createHash} from 'node:crypto';
import {containsCodeSnippet,findCodeSnippet} from './source-contract-tokens.mjs';
import {nativeFileIDs} from './native-files-oracle.mjs';
export function assertNativeFilesSource(source,row,kind) {
  if(!nativeFileIDs.includes(row.id)||row.family!=='native-files'||!['native','typescript'].includes(kind))throw Error('Native file family');
  if(createHash('sha256').update(source).digest('hex')!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('Native file fingerprint');
  if(assertUtilitySource(source,row,kind))return;
  if(row.id==='native_file_unbounded_full_write_read') {
    if(row.iterations!==3||row.operationsPerIteration!==1||row.operationUnit!=='One unbounded full-file write/read and byte sum'||row.resultEncoding!=='array-shape-scalar-v1'||row.inputText!=='native-file-fixture-v1')throw Error('Unbounded file denominator');
    const required=['const destination=input+".out";','let count=65536;','if(i%3===1){count=1048576;}','if(i%3===2){count=67108865;}','const fill=(i+seed)%256;','output.push(sample(input,count,fill));','while(i<n)'];
    if(kind==='native')required.push('alloc(count,fill)','writeFile(destination,owner)','readFile(destination)','while(index<length(bytes))','sum+=unwrap(at(bytes,index));','return [length(bytes),sum];');
    else required.push('new Uint8Array(count).fill(fill)','fileSystem.writeFileSync(destination,owner)','fileSystem.readFileSync(destination)','while(index<bytes.length)','sum+=bytes[index];','return [bytes.length,sum];');
    for(const snippet of required)if(!containsCodeSnippet(source,snippet))throw Error(`Unbounded file source contract: ${snippet}`);
    return;
  }
  if(row.id==='native_file_buffered_write_read') {
    if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One buffered write and readback with 16-byte buffers and 3-byte chunks'||row.resultEncoding!=='array-shape-scalar-v1'||row.inputText!=='native-file-fixture-v1')throw Error('Buffered file denominator');
    const required=['const count=(i+seed)%64+1;','while(i<n)','output.push(sample(input,count));','const destination=input+".out";','let end=offset+3;','if(end>count){end=count;}','total+=size;','if(size===0){break;}','return [total,sum];'];
    if(kind==='native')required.push('create(path)','open(path)','bufferWriter(file,16)','bufferReader(file,16)','view(owner,offset,end)','writeAll(output,bytes)','read(file,3)','close(output)','close(file)','sum+=value;','fromUtf8("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")');
    else required.push('const buffer=new Uint8Array(16);let used=0;','const buffer=new Uint8Array(16);let start=0;let end=0;','fileSystem.openSync(destination,"w")','fileSystem.openSync(destination,"r")','owner.subarray(offset,end)','if(bytes.length>buffer.length-used)','flush(output,buffer,used);','buffer.set(bytes,used);','fileSystem.writeSync(file,buffer,written,used-written,null)','fileSystem.readSync(file,buffer,0,buffer.length,null)','Math.min(3,end-start)','buffer.slice(start,start+size)','sum+=bytes[index];','fileSystem.closeSync(output)','fileSystem.closeSync(file)','new TextEncoder().encode("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")');
    for(const snippet of required)if(!containsCodeSnippet(source,snippet))throw Error(`Buffered file source contract: ${snippet}`);
    return;
  }
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

function assertUtilitySource(source,row,kind) {
  const specs={
    native_file_sync_storage:[16,'One write and syncData, write and syncAll, then byte readback',['openFile(destination, { read: true, write: true, create: true, truncate: true })','writeAll(file, first)','syncData(file)','writeAll(file, second)','syncAll(file)','seek(file, -2, "current")','read(file, 2)','unwrap(at(bytes, 0))','unwrap(at(bytes, 1))','position(file)','close(file)'],['openSync(destination,"w+")','writeSync(file,first,0,1,null)','fdatasyncSync(file)','writeSync(file,second,0,1,null)','fsyncSync(file)','readSync(file,bytes,0,2,position)','position+=count','closeSync(file)']],
    native_file_seek_read:[1024,'One positioned 64-byte read, cursor query and byte sum',['open(input)','seek(file, offset, "start")','read(file, 64)','position(file)','sum += unwrap(at(bytes, index));','close(file)'],['openSync(input, "r")','Buffer.allocUnsafe(64)','const rows:number[][]=new Array(n)','readSync(file,bytes,0,64,position)','position+=count','closeSync(file)']],
    native_directory_metadata:[64,'One directory enumeration and metadata pass',['readDir(input)','stat(entry.path)','metadata.size','entry.isSymlink'],['readdirSync(input, { withFileTypes: true })','statSync(entry.path, { bigint: true })','metadata.size','entry.isSymlink']],
    native_file_copy_rename_remove:[64,'One 1 MiB file copy, metadata, rename and remove',['copyFile(source, target)','stat(target)','rename(target, moved)','removeFile(moved)'],['copyFileSync(source, target)','statSync(target, { bigint: true })','renameSync(target, moved)','unlinkSync(moved)']],
    native_file_options_text:[128,'One UTF8 write, append, open, truncate, rewrite and read',['writeTextFile(destination, initial)','appendFile(destination, newline)','openFile(destination, { read: true, write: true })','fileStat(file)','truncate(file, 0)','writeAll(file, content)','close(file)','readTextFile(destination)'],['writeFileSync(destination, initial','appendFileSync(destination, newline)','openSync(destination, "r+")','fstatSync(file, { bigint: true })','ftruncateSync(file, 0)','closeSync(file)','fatal: true','ignoreBOM: true']]
  };
  if(row.id==='native_file_seek_read'&&kind==='typescript'){
    const buffers=findCodeSnippet(source,'Buffer.allocUnsafe(64)'),loops=findCodeSnippet(source,'while(i<n)');
    if(buffers.length!==1||loops.length!==1||buffers[0].start>=loops[0].start)throw Error('Seek Node control must reuse one buffer outside its loop');
  }
  const spec=specs[row.id];if(!spec)return false;
  if(row.iterations!==spec[0]||row.operationsPerIteration!==1||row.operationUnit!==spec[1]||row.result!=='int[][]'||row.resultEncoding!=='array-shape-scalar-v1'||JSON.stringify(row.parameters)!=='["int","int","string"]')throw Error('Native utility manifest contract');
  for(const snippet of spec[kind==='native'?2:3])if(!containsCodeSnippet(source,snippet))throw Error(`Native utility source contract: ${snippet}`);
  return true;
}
