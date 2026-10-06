function workload(n:number,seed:number):number{const consumer=useOnce;let trace=seed;let i=0;while(i<n){const callback=(x:number):number=>{return x+seed%13;};const factory=consumer((f:(x:number)=>number):()=>number=>{return():number=>{return f(i);};},callback);const value=factory();trace=(trace*131+value)%1000000007;i=i+1;}return trace;}
function useOnce(c:(f:(x:number)=>number)=>()=>number,f:(x:number)=>number):()=>number{return c(f);}
function main(){const result=workload(2,7);}
function clone<T>(value:T):T{return value;}
