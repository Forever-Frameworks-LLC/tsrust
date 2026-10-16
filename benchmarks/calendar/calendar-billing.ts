import {performance} from 'node:perf_hooks';
import {parseISO,formatISO,addMonths as shiftMonths,addDays as shiftDays,differenceInCalendarDays,isValid} from 'date-fns';
process.env.TZ='UTC';
type Result<T>={kind:'Ok';value:T}|{kind:'Err';error:string};
function checked(value:Date):Result<Date>{return isValid(value)&&value.getFullYear()>=1&&value.getFullYear()<=9999?{kind:'Ok',value}:{kind:'Err',error:'Range'};}
function parseDate(text:string):Result<Date>{return /^\d{4}-\d{2}-\d{2}$/.test(text)?checked(parseISO(text)):{kind:'Err',error:'Syntax'};}
function formatDate(value:Date):string{return formatISO(value,{representation:'date'});}
function addMonths(value:Date,amount:number):Result<Date>{return checked(shiftMonths(value,amount));}
function addDays(value:Date,amount:number):Result<Date>{return checked(shiftDays(value,amount));}
function unwrap<T>(value:Result<T>):T{if(value.kind==='Err')throw Error(value.error);return value.value;}
function batch():number{
 const base=unwrap(parseDate('2024-01-31'));let checksum=0;
 for(let i=0;i<256;i+=1){
 const due=unwrap(addMonths(base,1));
 const shifted=unwrap(addDays(due,i));
 const restored=unwrap(parseDate(formatDate(shifted)));
 checksum+=differenceInCalendarDays(restored,base);
 }
 return checksum;
}
for(let warmup=0;warmup<8;warmup+=1){batch();}for(let sample=0;sample<31;sample+=1){const start=performance.now();const checksum=batch();const elapsed=performance.now()-start;console.log(checksum);console.log(elapsed);}
