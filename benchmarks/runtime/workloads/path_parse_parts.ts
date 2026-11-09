declare function require(name:string): {parse(value:string):{root:string;dir:string;base:string;ext:string;name:string};format(parts:{root?:string;dir?:string;base?:string;ext?:string;name?:string}):string;resolve(...paths:string[]):string;sep:string};
const path=require('node:path');

function workload(n:number,seed:number,input:string):number{let checksum=seed;let i=0;while(i<n){const value=path.parse(input);if(value.root!==''||value.dir!=='reports'||value.base!=='file.tar.gz'||value.ext!=='.gz'||value.name!=='file.tar'){throw 'path parts';}checksum+=value.root.length+value.dir.length+value.base.length+value.ext.length+value.name.length;i++;}return checksum;}
