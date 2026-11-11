function workload(n:number,seed:number):number{const level='info';const debug=(make:()=>unknown)=>{if(String(level)==='debug')make();};let sum=0;for(let i=0;i<n;i++){debug(()=>{throw Error('disabled factory ran');});sum=(sum*131+i+seed)%1000000007;}return sum;}
function main(){}
