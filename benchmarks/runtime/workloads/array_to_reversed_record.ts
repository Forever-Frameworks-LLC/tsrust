type CopyingBenchmarkRow={code:number;text:string;items:number[];};
function workload(n:number,seed:number,input:string):CopyingBenchmarkRow[]{const output:CopyingBenchmarkRow[]=[];let i=0;while(i<n){const values:CopyingBenchmarkRow[]=[item(seed+i+0,input),item(seed+i+1,input),item(seed+i+2,input),item(seed+i+3,input),item(seed+i+4,input),item(seed+i+5,input)];const result=values.toReversed();for(const value of result){output.push(clone(value));}i=i+1;}return output;}
function item(base:number,input:string):CopyingBenchmarkRow{return {code:base%1009,text:clone(input),items:[base%17,0,(base+1)%17]};}
function main(){const result=workload(2,7," xé😀e\u0301 ");}
function clone<T>(value:T):T{return value;}
