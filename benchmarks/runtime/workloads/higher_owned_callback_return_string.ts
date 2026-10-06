function workload(n:number,seed:number,input:string):string[]{const keeper=keep;const output:string[]=[];let i=0;while(i<n){const callback=make(clone(input));const returned=keeper(callback);output.push(returned("x😀"));output.push(returned("a\u0000bx"));i=i+1;}return output;}
function keep(f:(text:string)=>string):(text:string)=>string{return f;}
function make(prefix:string):(text:string)=>string{return(text:string):string=>{return text.replaceAll("x",prefix);};}
function main(){const result=workload(2,7," xé😀e\u0301 ");}
function clone<T>(value:T):T{return value;}
