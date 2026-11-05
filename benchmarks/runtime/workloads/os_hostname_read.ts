declare const Buffer:{byteLength(value:string,encoding:string):number};
declare function require(name:string):{availableParallelism():number;hostname():string};
const os=require('node:os');

function workload(n:number,seed:number):number{let checksum=seed;let i=0;while(i<n){const value=os.hostname();checksum+=Buffer.byteLength(value,'utf8');i++;}return checksum;}
