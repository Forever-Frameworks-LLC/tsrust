type DeclaredBenchmarkNode={code:number;text:string;children:DeclaredBenchmarkNode[];};
function workload(n:number,seed:number,input:string):DeclaredBenchmarkNode[]{const output:DeclaredBenchmarkNode[]=[];const action=take;let i=0;while(i<n){const root:DeclaredBenchmarkNode={code:seed+i,text:clone(input),children:[{code:i,text:"a\u0000b",children:[]},{code:i+1,text:"e\u0301😀",children:[{code:seed,text:"leaf",children:[]}]}]};const moved=action(root);output.push(moved);i=i+1;}return output;}
function take(root:DeclaredBenchmarkNode):DeclaredBenchmarkNode{return root;}

function main(){const result=workload(2,7," xé😀e\u0301 ");}
function clone<T>(value:T):T{return value;}
