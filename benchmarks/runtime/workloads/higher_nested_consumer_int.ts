function workload(n:number,seed:number):number{const maker=factoryFactory;const factory=maker();const consumer=factory();let trace=seed;let i=0;while(i<n){const a=pass(consumer,plus,seed+i);const b=pass(consumer,times,seed+i);const value=a*7+b;trace=(trace*131+value)%1000000007;i=i+1;}return trace;}
function plus(x:number):number{return x+3;}
function times(x:number):number{return x*2;}
function apply(f:(x:number)=>number,x:number):number{return f(x);}
function pass(c:(f:(x:number)=>number,x:number)=>number,f:(x:number)=>number,x:number):number{return c(f,x);}
function factory():(f:(x:number)=>number,x:number)=>number{return apply;}
function factoryFactory():()=>(f:(x:number)=>number,x:number)=>number{return factory;}
function main(){const result=workload(2,7);}
function clone<T>(value:T):T{return value;}
