import {performance} from 'node:perf_hooks';
import Decimal from 'decimal.js';
Decimal.set({precision:100});
type Result<T>={kind:'Ok';value:T}|{kind:'Err';error:string};
function checked(value:Decimal):Result<Decimal>{return value.isFinite()&&value.sd()<=38&&value.dp()<=28&&value.e<38?{kind:'Ok',value}:{kind:'Err',error:'Range'};}
function parseDecimal(text:string):Result<Decimal>{if(text.length>128||!/^[-+]?\d+(\.\d+)?$/.test(text))return {kind:'Err',error:'Syntax'};return checked(new Decimal(text));}
function multiply(left:Decimal,right:Decimal):Result<Decimal>{return checked(left.mul(right));}
function add(left:Decimal,right:Decimal):Result<Decimal>{return checked(left.add(right));}
function toInt(value:Decimal):Result<bigint>{return value.isInteger()&&value.gte('-9223372036854775808')&&value.lte('9223372036854775807')?{kind:'Ok',value:BigInt(value.toFixed(0))}:{kind:'Err',error:'Range'};}
function unwrap<T>(value:Result<T>):T{if(value.kind==='Err')throw Error(value.error);return value.value;}
function batch():number{
 let checksum=0;
 const price=unwrap(parseDecimal('19.99'));
 const fee=unwrap(parseDecimal('0.01'));
 const hundred=new Decimal(100);
 for(let i=0;i<256;i+=1){
 const total=unwrap(multiply(price,new Decimal(i)));
 const adjusted=unwrap(add(total,fee));
 const cents=unwrap(multiply(adjusted,hundred));
 checksum+=Number(unwrap(toInt(cents)));
 }
 return checksum;
}
for(let warmup=0;warmup<8;warmup+=1){batch();}
for(let sample=0;sample<31;sample+=1){const start=performance.now();const checksum=batch();const elapsed=performance.now()-start;console.log(checksum);console.log(elapsed);}
