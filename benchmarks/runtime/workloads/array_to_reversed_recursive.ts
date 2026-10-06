type CopyingBenchmarkNode={code:number;text:string;children:CopyingBenchmarkNode[];};
function workload(n:number,seed:number,input:string):CopyingBenchmarkNode[]{const output:CopyingBenchmarkNode[]=[];let i=0;while(i<n){const values:CopyingBenchmarkNode[]=[item(seed+i+0,input),item(seed+i+1,input),item(seed+i+2,input),item(seed+i+3,input),item(seed+i+4,input),item(seed+i+5,input)];const result=values.toReversed();for(const value of result){output.push(clone(value));}i=i+1;}return output;}
function item(base:number,input:string):CopyingBenchmarkNode{return {code:base%1009,text:clone(input),children:[{code:base%17,text:"a\u0000b",children:[]},{code:(base+1)%17,text:"😀",children:[{code:0,text:"leafé",children:[]}]}]};}
function main(){const result=workload(2,7," xé😀e\u0301 ");}
function clone<T>(value:T):T{return value;}
