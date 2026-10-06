function attempt(value:number,trace:number[]):number {
 try {
  if(value%3===0){throw value;}
  return value+1;
 }catch(error){if(typeof error!=="number"){throw error;}return error+2;}
 finally{trace.push(value);}
}
function replaced(value:number):number{try{return value;}finally{return value+4;}}
function workload(n: number, seed: number, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while(i<n){
  const value=(i*13+seed)%97;
  const trace:number[]=[];
  const result=attempt(value,trace);
  output.push([String(result),String(replaced(value)),trace.join(","),input+String(value)]);
  i=i+1;
 }
 return output;
}
function main(){const result=workload(3,0,"abc");}
