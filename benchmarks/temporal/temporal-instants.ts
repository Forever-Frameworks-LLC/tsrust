import {performance} from 'node:perf_hooks';
import {parseISO as parse,format,addMilliseconds,isValid} from 'date-fns';
process.env.TZ='UTC';
type Result<T>={kind:'Ok';value:T}|{kind:'Err';error:string};
function checked(value:Date):Result<Date>{return isValid(value)&&value.getUTCFullYear()>=1&&value.getUTCFullYear()<=9999?{kind:'Ok',value}:{kind:'Err',error:'Range'};}
function parseISO(text:string):Result<Date>{return /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d{1,3})?(Z|[+-]\d{2}:\d{2})$/.test(text)?checked(parse(text)):{kind:'Err',error:'Syntax'};}
function formatISO(value:Date):string{return format(value,"yyyy-MM-dd'T'HH:mm:ss.SSSxxx");}
function durationFromMilliseconds(value:number):Result<number>{return Number.isSafeInteger(value)&&Math.abs(value)<=9223372036854?{kind:'Ok',value}:{kind:'Err',error:'Range'};}
function addDuration(value:Date,amount:number):Result<Date>{return checked(addMilliseconds(value,amount));}
function toUnixMilliseconds(value:Date):Result<number>{return {kind:'Ok',value:value.getTime()};}
function unwrap<T>(value:Result<T>):T{if(value.kind==='Err')throw Error(value.error);return value.value;}
function batch():number{
 const base=unwrap(parseISO('2024-01-01T00:00:00Z'));let checksum=0;
 for(let i=0;i<256;i+=1){
 const shifted=unwrap(addDuration(base,unwrap(durationFromMilliseconds(i))));
 const restored=unwrap(parseISO(formatISO(shifted)));
 checksum+=unwrap(toUnixMilliseconds(restored))-1704067200000;
 }
 return checksum;
}
for(let warmup=0;warmup<8;warmup+=1){batch();}for(let sample=0;sample<31;sample+=1){const start=performance.now();const checksum=batch();const elapsed=performance.now()-start;console.log(checksum);console.log(elapsed);}
