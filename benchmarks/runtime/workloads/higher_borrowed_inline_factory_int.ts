function workload(n:number,seed:number):number{const keeper=(f:(x:number)=>number):(x:number)=>number=>{return f;};let trace=seed;let i=0;while(i<n){const left=plus(seed%11);const right=times(seed%7+1);const first=relay(keeper,left);const second=relay(keeper,right);const a=first(i);const b=second(i);const c=first(i+1);const value=a*7+b+c;trace=(trace*131+value)%1000000007;i=i+1;}return trace;}
function relay(c:(f:(x:number)=>number)=>(x:number)=>number,f:(x:number)=>number):(x:number)=>number{return c(f);}
function plus(offset:number):(x:number)=>number{return(x:number):number=>{return x+offset;};}
function times(scale:number):(x:number)=>number{return(x:number):number=>{return x*scale;};}
function main(){const result=workload(2,7);}
function clone<T>(value:T):T{return value;}
