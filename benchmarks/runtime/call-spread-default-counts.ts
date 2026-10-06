type int=number;
type Trace={calls:int;defaults:int;};
function fallback(trace:Trace):int{trace.defaults=trace.defaults+1;return 10;}
function values(trace:Trace,count:int):int[]{trace.calls=trace.calls+1;const result:int[]=[];let at=0;while(at<count){result.push(at+3);at=at+1;}return result;}
function apply(trace:Trace,prefix:int=fallback(trace),...tail:int[]):int{return prefix+tail.length;}
function main(){const defaults:Trace={calls:0,defaults:0};const argumentsTrace:Trace={calls:0,defaults:0};console.log(apply(defaults,...values(argumentsTrace,0)),argumentsTrace.calls,defaults.defaults);console.log(apply(defaults,...values(argumentsTrace,3)),argumentsTrace.calls,defaults.defaults);console.log(apply(defaults,undefined,...values(argumentsTrace,2)),argumentsTrace.calls,defaults.defaults);}
main();
