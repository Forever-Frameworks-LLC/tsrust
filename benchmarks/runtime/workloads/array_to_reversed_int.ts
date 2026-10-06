function workload(n:number,seed:number):number[]{const output:number[]=[];let i=0;while(i<n){const values:number[]=[item(seed+i+0),item(seed+i+1),item(seed+i+2),item(seed+i+3),item(seed+i+4),item(seed+i+5)];const result=values.toReversed();for(const value of result){output.push(value);}i=i+1;}return output;}
function item(base:number):number{return base%1009;}
function main(){const result=workload(2,7);}
