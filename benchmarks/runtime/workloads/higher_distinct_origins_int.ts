function workload(n:number,seed:number):number{const consumer=apply;const left=plus;const right=times;let trace=seed;let i=0;while(i<n){const a=consumer(left,seed+i);const b=consumer(right,seed+i);const value=a*7+b;trace=(trace*131+value)%1000000007;i=i+1;}return trace;}
function apply(f:(x:number)=>number,x:number):number{return f(x);}
function plus(x:number):number{return x+3;}
function times(x:number):number{return x*2;}
function main(){const result=workload(2,7);}
function clone<T>(value:T):T{return value;}
