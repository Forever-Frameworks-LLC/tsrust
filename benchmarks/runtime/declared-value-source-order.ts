type Probe={state:number;};
function main(){let probe:Probe={state:0};console.log(receiver().reduce(factory(probe),initial(probe)));console.log(receiver().reduceRight(factory(probe),initial(probe)));console.log(empty().reduce(factory(probe),initial(probe)));console.log([5].reduce(fold));const selected=step;const bound=(value:number):number=>{return selected(value);};const global=(value:number):number=>{const target=step;return target(value);};console.log(bound(2));console.log(bound(3));console.log(global(2));const original=step;const moved=original;console.log(moved(4));const factoryValue=make;const action=factoryValue("é\u0000");console.log(action("xx"));console.log(action("x😀"));}
function receiver():number[]{console.log("receiver");return [1,2];}
function empty():number[]{console.log("empty");return [];}
function factory(probe:Probe):(a:number,value:number,index:number)=>number{probe.state=probe.state+1;console.log("factory");return fold;}
function initial(probe:Probe):number{console.log("initial");return probe.state;}
function fold(a:number,value:number,index:number):number{console.log(index);return a*10+value;}
function step(value:number):number{return value*3+1;}
function make(prefix:string):(text:string)=>string{return(text:string):string=>{return text.replaceAll("x",prefix);};}
