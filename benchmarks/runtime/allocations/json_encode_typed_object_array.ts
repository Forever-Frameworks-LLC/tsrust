type Item={active:boolean;items:number[][];name:string};
function workload(n:number,seed:number,input:Item):string{
 return JSON.stringify(input);
}
