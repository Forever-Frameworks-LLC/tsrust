type int = number;
type Trace = {calls:int;};
function collect(...values:int[]):int[]{return values;}
function segment(trace:Trace,value:int):int[]{trace.calls=trace.calls+1;return [value,trace.calls];}
function choose(trace:Trace):(...values:int[])=>int[]{trace.calls=trace.calls+1;return collect;}
function mutate(source:int[]):int[]{source[0]=9;return [source[0],source[1]];}
function mixed(prefix:string,...tail:string[]):string[]{tail.push(String(prefix.length));return tail;}
function emitInts(values:int[]){let at=0;let line="";while(at < values.length){line=line.concat(at===0?"":",",String(values[at]));at=at+1;}console.log(line);}
function emitText(values:string[]){let at=0;let line="";while(at < values.length){line=line.concat(at===0?"":",",values[at]);at=at+1;}console.log(line);}
function optional(action:((...values:int[])=>int[])|undefined,trace:Trace):int[]|undefined{return action?.(...segment(trace,99));}
function main(){
 const callee:Trace={calls:0};const argumentsTrace:Trace={calls:0};
 emitInts(choose(callee)(...segment(argumentsTrace,4),...segment(argumentsTrace,callee.calls)));console.log(callee.calls,argumentsTrace.calls);
 const source:int[]=[1,2];emitInts(collect(...source,...mutate(source)));emitInts(source);
 const values:string[]=["ab","tail","é🙂"];emitText(mixed(...(values as [string,...string[]])));
 const skipped=optional(undefined,argumentsTrace);console.log(String(skipped),argumentsTrace.calls);
}
main();
