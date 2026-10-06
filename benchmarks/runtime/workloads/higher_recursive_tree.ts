type HigherBenchmarkNode={code:number;text:string;children:HigherBenchmarkNode[];};
function workload(n:number,seed:number,input:string):HigherBenchmarkNode[]{const consumer=copying;const output:HigherBenchmarkNode[]=[];let i=0;while(i<n){const value:HigherBenchmarkNode={code:seed+i,text:clone(input),children:[{code:i,text:"a\u0000b",children:[]},{code:i+1,text:"😀",children:[{code:seed,text:"leaf",children:[]}]}]};output.push(consumer(duplicate,value));i=i+1;}return output;}
function duplicate(value:HigherBenchmarkNode):HigherBenchmarkNode{return clone(value);}
function copying(f:(value:HigherBenchmarkNode)=>HigherBenchmarkNode,value:HigherBenchmarkNode):HigherBenchmarkNode{return f(value);}
function main(){const result=workload(2,7," xé😀e\u0301 ");}
function clone<T>(value:T):T{return value;}
