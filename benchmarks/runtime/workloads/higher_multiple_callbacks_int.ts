function workload(n:number,seed:number):number{const consumer=pair;const arrow=(left:(x:number)=>number,right:(x:number)=>number,x:number):number=>{return left(x)*7+right(x);};let trace=seed;let i=0;while(i<n){const a=consumer(plus,times,seed+i);const b=arrow(times,plus,seed+i);const value=a+b;trace=(trace*131+value)%1000000007;i=i+1;}return trace;}
function pair(left:(x:number)=>number,right:(x:number)=>number,x:number):number{return left(x)*7+right(x);}
function plus(x:number):number{return x+3;}
function times(x:number):number{return x*2;}
function main(){const result=workload(2,7);}
function clone<T>(value:T):T{return value;}
