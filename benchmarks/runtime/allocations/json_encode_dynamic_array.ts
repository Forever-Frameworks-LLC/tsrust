function workload(n:number,seed:number,input:unknown):string{
 const output=JSON.stringify(input);
 if(output===undefined)throw Error('undefined-value');
 return output;
}
