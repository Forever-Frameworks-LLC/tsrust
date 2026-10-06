type int=number;
type Probe={trace:int;calls:int;};
function workload(n: int, seed: int, input: string): string[][]{const pool=[clone(input),""];const counts=[0,1,3,0];const probe:Probe={trace:seed%65521,calls:0};const output:string[][]=[];let i=0;while(i<n){const value=pool[mark(probe,(i+seed)%2,1)].repeat(mark(probe,counts[(i+seed)%4],2));output.push([value]);i=i+1;}output.push(markers(probe));return output;}

function mark(probe:Probe,index:int,tag:int):int{probe.trace=(probe.trace*131+probe.calls+index%65521+65521+tag)%65521;probe.calls=probe.calls+1;return index;}
function markers(probe:Probe):string[]{const row:string[]=["trace"];let value=probe.trace;let factor=32768;while(factor>=1){if(value>=factor){row.push("1");value=value-factor;}else{row.push("0");}factor=factor/2;}row.push("calls");value=probe.calls;factor=32768;while(factor>=1){if(value>=factor){row.push("1");value=value-factor;}else{row.push("0");}factor=factor/2;}return row;}
function main(){console.log(workload(2,7,"").length);}
function clone(value:string):string{return value;}
