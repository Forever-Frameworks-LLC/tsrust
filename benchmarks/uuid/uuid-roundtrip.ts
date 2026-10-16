import {performance} from 'node:perf_hooks';
type Result<T>={kind:'Ok';value:T}|{kind:'Err';error:string};
function parseUuid(text:string):Result<Buffer>{return text.length===36&&/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(text)?{kind:'Ok',value:Buffer.from(text.replaceAll('-',''),'hex')}:{kind:'Err',error:'Syntax'};}
function formatUuid(value:Buffer):string{const hex=value.toString('hex');return `${hex.slice(0,8)}-${hex.slice(8,12)}-${hex.slice(12,16)}-${hex.slice(16,20)}-${hex.slice(20)}`;}
function toBytes(value:Buffer):Buffer{return Buffer.from(value);}
function fromBytes(value:Buffer):Result<Buffer>{return value.length===16?{kind:'Ok',value:Buffer.from(value)}:{kind:'Err',error:'Length'};}
function version(value:Buffer):number{return value[6]>>4;}
function set(value:Buffer,index:number,byte:number):Result<void>{if(!Number.isInteger(index)||index<0||index>=value.length||!Number.isInteger(byte)||byte<0||byte>255)return {kind:'Err',error:'Range'};value[index]=byte;return {kind:'Ok',value:undefined};}
function at(value:Buffer,index:number):Result<number>{return Number.isInteger(index)&&index>=0&&index<value.length?{kind:'Ok',value:value[index]}:{kind:'Err',error:'Range'};}
function unwrap<T>(value:Result<T>):T{if(value.kind==='Err')throw Error(value.error);return value.value;}
function batch():number{let checksum=0;for(let i=0;i<256;i+=1){
 const id=unwrap(parseUuid('67e55044-10b1-426f-9247-bb680e5fe0c8'));
 const bytes=toBytes(id);unwrap(set(bytes,15,i));
 const changed=unwrap(fromBytes(bytes));const restored=unwrap(parseUuid(formatUuid(changed)));
 checksum+=version(restored)+unwrap(at(toBytes(restored),15));
 }return checksum;}
for(let warmup=0;warmup<8;warmup+=1){batch();}for(let sample=0;sample<31;sample+=1){const start=performance.now();const checksum=batch();const elapsed=performance.now()-start;console.log(checksum);console.log(elapsed);}
