function workload(n:number,seed:number):number{const base:number[]=[0.25,-0.5,1.5,2.25,-1.0];let total:number=0.0;let i=0;while(i<n){const mode=(seed+i)%4;let count=5;if(mode===0){count=0;}else if(mode===1){count=1;}else if(mode===2){count=4;}const values=base.slice(0,count);total=total+values.reduce(add,0.0);i=i+1;}return total;}
function add(a:number,v:number):number{return a+v;}

function main(){const result=workload(2,7);}
