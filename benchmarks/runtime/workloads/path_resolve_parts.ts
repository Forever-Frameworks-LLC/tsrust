declare function require(name:string): {parse(value:string):{root:string;dir:string;base:string;ext:string;name:string};format(parts:{root?:string;dir?:string;base?:string;ext?:string;name?:string}):string;resolve(...paths:string[]):string;sep:string};
const path=require('node:path');

function workload(n:number,seed:number,input:string):number{const root=path.sep==='\\'?'C:\\base':'/base';let checksum=seed;let i=0;while(i<n){const value=path.resolve(root,input);if(!value.endsWith('file.txt')){throw 'path resolve';}checksum+=value.length;i++;}return checksum;}
