declare const Buffer:{byteLength(value:string,encoding:string):number};
declare function require(name:string):{env:Record<string,string|undefined>};
const environment=require('node:process').env;

function workload(n:number,seed:number):number{const names=['TSRUST_B03_TEXT','TSRUST_B03_EMPTY','TSRUST_B03_MISSING'];let checksum=seed;let i=0;while(i<n){for(const name of names){const value=environment[name];checksum+=value===undefined?1:Buffer.byteLength(value,'utf8');}i++;}return checksum;}
