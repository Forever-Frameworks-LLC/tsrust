function workload(n:number,seed:number):number{const selected=step;const action=(value:number):number=>{return selected(value);};let trace=seed;let i=0;while(i<n){const value=action(seed+i);trace=(trace*131+value)%1000000007;i=i+1;}return trace;}
function step(value:number):number{return(value*3+1)%1000000007;}

function main(){const result=workload(2,7);}
