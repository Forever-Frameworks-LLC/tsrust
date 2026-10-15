import {performance} from 'node:perf_hooks';
type Result<T>={kind:'Ok';value:T}|{kind:'Err';error:string};
function toNumber(value:bigint):Result<number>{const result=Number(value);return BigInt(result)===value?{kind:'Ok',value:result}:{kind:'Err',error:'Precision'};}
function toInt(value:number):Result<bigint>{return Number.isFinite(value)&&Number.isInteger(value)&&value>=-9223372036854775808&&value<9223372036854775808?{kind:'Ok',value:BigInt(value)}:{kind:'Err',error:'Range'};}
function toFloat32(value:bigint):Result<number>{const result=Math.fround(Number(value));return BigInt(result)===value?{kind:'Ok',value:result}:{kind:'Err',error:'Precision'};}
function checkSigned(value:bigint):Result<bigint>{return value>=-128n&&value<128n?{kind:'Ok',value}:{kind:'Err',error:'Range'};}
function unwrap<T>(value:Result<T>):T{if(value.kind==='Err')throw Error(value.error);return value.value;}
function batch():number{
 let checksum=0;
 for(let i=0;i<2048;i+=1){
 const integer=BigInt(i);
 const value=Number(unwrap(toInt(unwrap(toNumber(integer)))));
 const exact=toFloat32(integer).kind==='Ok'?1:0;
 const range=checkSigned(integer).kind==='Ok'?1:0;
 checksum+=value+exact+range;
 }
 return checksum;
}
for(let warmup=0;warmup<8;warmup+=1){batch();}
for(let sample=0;sample<31;sample+=1){const start=performance.now();const checksum=batch();const elapsed=performance.now()-start;console.log(checksum);console.log(elapsed);}
