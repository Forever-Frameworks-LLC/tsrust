declare function require(name:string):{availableParallelism():number;hostname():string};
const os=require('node:os');

function workload(n:number,seed:number):number{let checksum=seed;let i=0;while(i<n){const value=os.availableParallelism();if(value<1){throw 'parallelism';}checksum+=1;i++;}return checksum;}
