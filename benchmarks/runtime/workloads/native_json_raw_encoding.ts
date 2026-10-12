function workload(n:number,seed:number,input:string):number{
 let checksum=seed;let i=0;
 while(i<n){
  JSON.parse(input);
  const encoded='{"payload":'+input+'}';
  if(encoded!=="{\"payload\": { \"a\": [1,true,null], \"b\": \"hello\" } }"){throw 'raw output mismatch';}
  checksum+=encoded.length;
  i++;
 }
 return checksum;
}
