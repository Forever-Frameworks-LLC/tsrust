function workload(n:number,seed:number,input:string):number {
 const owner=JSON.parse(input);
 let checksum=seed;let i=0;
 while(i<n){
  if(typeof owner!=='object'||owner===null||Array.isArray(owner)){throw 'expected object';}
  const items=owner['items'];
  if(items===undefined){throw 'missing array';}
  if(!Array.isArray(items)){throw 'expected array';}
  const child=items[0];
  if(child===undefined){throw 'missing item';}
  if(typeof child!=='number'||!Number.isSafeInteger(child)){throw 'expected integer';}
  checksum+=child;
  i++;
 }
 return checksum;
}
