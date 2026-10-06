function workload(n:number,seed:number,input:string):string[]{const consumer=carry;const output:string[]=[];let i=0;while(i<n){const factory=make(clone(input));const carried=consumer(factory);const callback=carried();output.push(callback("xé"));i=i+1;}return output;}
function carry(f:()=>(text:string)=>string):()=>(text:string)=>string{return():(text:string)=>string=>{return f();};}
function make(prefix:string):()=>(text:string)=>string{return():(text:string)=>string=>{const copied=clone(prefix);return(text:string):string=>{return text.replaceAll("x",copied);};};}
function main(){const result=workload(2,7," xé😀e\u0301 ");}
function clone<T>(value:T):T{return value;}
