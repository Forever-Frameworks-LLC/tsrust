// Independent exact integer observer; no assertion API or workload pool is reused.
export function expectedTestingAssertionChecksum(n, seed) {
 if(!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(seed)||seed<0)throw Error('Nonnegative safe integer inputs required');
 let result=0n;
 const offset=BigInt(seed);
 for(let position=0n;position<BigInt(n);position++){
  const index=(13n*position+offset)%8n;
  result=(result*131n+index**2n)%1000000007n;
 }
 return Number(result);
}

// Independent category observer, without matcher calls or numeric source values.
export function expectedTestingExpectationChecksum(n, seed) {
 if(!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(seed)||seed<0)throw Error('Nonnegative safe integer inputs required');
 const categories=[1n,2n,8n,8n,8n,4n];let result=0n;
 for(let i=0n;i<BigInt(n);i++)result=(result*131n+categories[Number((13n*i+BigInt(seed))%6n)])%1000000007n;
 return Number(result);
}
