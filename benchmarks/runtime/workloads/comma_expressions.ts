function observe(trace:number[],value:number):number{trace.push(value);return value;}
function selected(trace:number[],value:number):number{return observe(trace,value),value+1;}
function workload(n: number, seed: number, input: string): string[][] {
 const output:string[][]=[];
 for(let i=0,j=seed;i<n;i++,j++){
  const trace:number[]=[];const value=(observe(trace,j),observe(trace,j+2),j*3);
  const result=selected(trace,value);const text=(observe(trace,j+4),String(input));
  output.push([String(value),String(result),trace.join(","),text,String(i),String(j)]);
 }
 return output;
}
function main(){const result=workload(3,0,"abc");}
