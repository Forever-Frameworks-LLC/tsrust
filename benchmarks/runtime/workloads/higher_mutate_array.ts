function workload(n:number,seed:number):number[][]{const consumer=drive;const output:number[][]=[];let i=0;while(i<n){const values=[seed+i,2,3];const result=consumer(change,values);output.push(result);output.push(clone(values));i=i+1;}return output;}
function change(values:number[]):number[]{values[0]=values[0]+1;return clone(values);}
function drive(f:(values:number[])=>number[],values:number[]):number[]{return f(values);}
function main(){const result=workload(2,7);}
function clone<T>(value:T):T{return value;}
