function attempt(value:number,trace:number[]):number {
 try {
  if(value%3===0){throw value;}
  return value+1;
 }catch(error){if(typeof error!=="number"){throw error;}return error+2;}
 finally{trace.push(value);}
}
function replaced(value:number):number{try{return value;}finally{return value+4;}}
function retained(value: string, early: boolean, trace: number[]): string {
  try {
    if (early) { return value; }
  } finally { trace.push(7); }
  return value;
}

type RetainedOwner = {text: string; stamp: number;};
function retainedField(value: RetainedOwner, early: boolean, trace: number[]): string {
  try { if (early) { return value.text; } } finally { trace.push(value.stamp); }
  return value.text;
}

function workload(n: number, seed: number, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while(i<n){
  const value=(i*13+seed)%97;
  const trace:number[]=[];
  const result=attempt(value,trace);
  const owned = retained(input + String(value), value % 2 === 0, trace);
    const field = retainedField({text: owned, stamp: 11}, value % 2 === 0, trace);
  output.push([String(result),String(replaced(value)),trace.join(","),field]);
  i=i+1;
 }
 return output;
}
function main(){const result=workload(3,0,"abc");}
