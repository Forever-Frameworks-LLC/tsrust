// Independent arithmetic and tag/count oracle. Never invokes tested callbacks or nullable operators.
export const ids = ['nullable_callable_declared_guard','nullable_callable_declared_optional','nullable_callable_read_forward','nullable_callable_once_forward','nullable_callable_owned_record','nullable_callable_owned_once_return','nullable_callable_generic_coalesce','nullable_callable_nullish_assign'];
export function validateCase(id,n,seed,input) {
 if(!ids.includes(id)||!Number.isInteger(n)||n<0||n>2048||!Number.isInteger(seed)||seed<0||seed>1000000||typeof input!=='string'||input.length>256||[...input].some(c=>c.codePointAt(0)>127)) throw new Error('Outside shared bounded ASCII source domain');
}
export function expectedNullableCallableValues(id,n,seed,input) {
 validateCase(id,n,seed,input);const result=[];
 for(let i=0;i<n;i++) {
  const tag=(i+seed)%3,value=(i*13+seed)%97,present=tag===0;let values=[],calls=0,args=0,fallbacks=0,factories=0;
  if(id==='nullable_callable_owned_record'||id==='nullable_callable_owned_once_return')factories=1;
  if(id==='nullable_callable_generic_coalesce'||id==='nullable_callable_nullish_assign') {
   const extra=id==='nullable_callable_nullish_assign'?3:(present?3:103);fallbacks=present?0:1;values=[(value*17+extra)%1009];calls=1;args=1;
   if(id==='nullable_callable_nullish_assign'){values.push((((value+1)%97)*17+extra)%1009);calls=2;args=2;}
  } else if(present) {
   calls=1;args=1;
   if(id==='nullable_callable_read_forward')values=[(value*17+seed%11+3)%1009];
   else if(id==='nullable_callable_once_forward')values=[(value+(i+seed)%11+5)%1009];
   else if(id==='nullable_callable_owned_record'||id==='nullable_callable_owned_once_return') {
    values=[(value+input.length+value%7)%1009];
    if(id==='nullable_callable_owned_record'){values.push(((value+1)%97+input.length+value%7)%1009);calls=2;args=2;}
   } else values=[(value*17+3)%1009];
  }
  result.push(values,[calls,args,fallbacks,factories]);
 }
 return result;
}
