function make(base:number):(value:number)=>number{return (value:number):number=>base+value;}
function chain(a:number):(b:number)=>(c:number)=>number{return (b:number):(c:number)=>number => (c:number):number=>a+b+c;}
function factory(trace:number[],base:number):(value:number)=>number{trace.push(1);return (value:number):number=>value+base;}
function mark(trace:number[],value:number):number{trace.push(2);return value;}
function workload(n: number, seed: number, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while (i < n) {
  const base=(i*13+seed)%97;
  const direct=make(base)(3);
  const nested=chain(base)(2)(3);
  const inline=((value:number)=>value*2)(base);
  const named=(function(value:number):number{return value+4;})(base);
  const args:[number,number]=[base,2];const spread=((a:number,b:number)=>a+b)(...args);
  const omitted=((value:number=7):number=>value)();
  const trace:number[]=[];const ordered=factory(trace,base)(mark(trace,5));
  output.push([String(direct),String(nested),String(inline),String(named),String(spread),String(omitted),String(ordered),trace.join(","),input+String(base)]);
  i=i+1;
 }return output;
}
function main(){const result=workload(3,0,"abc");}
