function workload(n:number,seed:number):number{let trace=seed;const consumer=apply;let i=0;while(i<n){const callback=(x:number):number=>{return (x*3+1)%1000000007;};const value=consumer(callback,seed+i);trace=(trace*131+value)%1000000007;i=i+1;}return trace;}
function apply(f:(x:number)=>number,x:number):number{return f(x);}

function main(){const result=workload(2,7);}
function clone<T>(value:T):T{return value;}
