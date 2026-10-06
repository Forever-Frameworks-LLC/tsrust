function workload(n:number,seed:number):number[][][]{const consumer=copying;const output:number[][][]=[];let i=0;while(i<n){const value:number[][]=[[seed+i,1],[],[i,2,3]];output.push(consumer(duplicate,value));i=i+1;}return output;}
function duplicate(values:number[][]):number[][]{return clone(values);}
function copying(f:(values:number[][])=>number[][],values:number[][]):number[][]{return f(values);}
function main(){const result=workload(2,7);}
function clone<T>(value:T):T{return value;}
