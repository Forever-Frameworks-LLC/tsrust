import {performance} from 'node:perf_hooks';
type Result={kind:'Ok';value:number}|{kind:'Err';error:number};
type Person={kind:'User';value:number}|{kind:'Guest';value:number};
function result(value:number):Result{return value%3===0?{kind:'Err',error:value}:{kind:'Ok',value:value*2};}
function person(value:number):Person{return value%2===0?{kind:'User',value:value+1}:{kind:'Guest',value:value+2};}
function batch():number{
 let checksum=0;
 for(let i=0;i<2048;i+=1){
  const selected=result(i);const value=selected.kind==='Ok'?selected.value:-selected.error;
  let weight:number;switch(i%4){case 0:weight=1;break;case 1:weight=3;break;case 2:weight=5;break;default:weight=7;}
  const selectedPerson=person(i);let extra:number;switch(selectedPerson.kind){case 'User':extra=selectedPerson.value;break;case 'Guest':extra=selectedPerson.value;break;}
  checksum+=value+weight+extra;
 }
 return checksum;
}
for(let warmup=0;warmup<8;warmup+=1){batch();}
for(let sample=0;sample<31;sample+=1){const start=performance.now();const checksum=batch();const elapsed=performance.now()-start;console.log(checksum);console.log(elapsed);}
