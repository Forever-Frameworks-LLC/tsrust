function workload(n:number,seed:number):boolean[]{const output:boolean[]=[];let i=0;while(i<n){const values:boolean[]=[item(seed+i+0),item(seed+i+1),item(seed+i+2),item(seed+i+3),item(seed+i+4),item(seed+i+5)];const replacement=item(seed+i+11);if((i+seed)%8===0){values.fill(replacement);}else if((i+seed)%8===1){values.fill(replacement,start(i,seed));}else{values.fill(replacement,start(i,seed),ending(i,seed));}for(const value of values){output.push(value);}i=i+1;}return output;}
function start(i:number,seed:number):number{const c=(i+seed)%8;if(c===0){return 0;}if(c===1){return 1;}if(c===2){return -6;}if(c===3){return -9;}if(c===4){return -9;}if(c===5){return 4;}if(c===6){return -3;}return -9;}
function ending(i:number,seed:number):number{const c=(i+seed)%8;if(c===0){return 5;}if(c===1){return 6;}if(c===2){return -1;}if(c===3){return 9;}if(c===4){return 9;}if(c===5){return 1;}if(c===6){return 9;}return -1;}
function target(i:number,seed:number):number{const c=(i+seed)%8;if(c===0){return 1;}if(c===1){return 0;}if(c===2){return -2;}if(c===3){return -9;}if(c===4){return 9;}if(c===5){return 2;}if(c===6){return -1;}return 0;}
function item(base:number):boolean{return base%3!==0;}
function main(){const result=workload(2,7);}
function clone<T>(value:T):T{return value;}
