import {observeSliceTree} from './slice-observation.mjs';
export const stringFromCodesIds = ['string_from_codes_text'];
// Independent of String.fromCodePoint and String.fromCharCode: each code is
// written as UTF-8 bytes by hand and the bytes are decoded as text.
export function textFromCodes(codes) {
 const bytes=[];
 for(const code of codes) {
  if(code<0x80) bytes.push(code);
  else if(code<0x800) bytes.push(0xC0|code>>6,0x80|code&0x3F);
  else if(code<0x10000) bytes.push(0xE0|code>>12,0x80|code>>6&0x3F,0x80|code&0x3F);
  else bytes.push(0xF0|code>>18,0x80|code>>12&0x3F,0x80|code>>6&0x3F,0x80|code&0x3F);
 }
 return new TextDecoder('utf-8',{fatal:true}).decode(new Uint8Array(bytes));
}
export function expectedStringFromCodesValues(id,n,seed) {
 if (!stringFromCodesIds.includes(id)) throw Error('Unknown string from codes workload');
 if (!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000) throw Error('String from codes case bounds');
 const values=[];
 for(let i=0;i<n;i++) {
  const base=(i*13+seed)%97,codes=[65+base%26,233+base,19968+base,128512+base%64];
  values.push([textFromCodes(codes),textFromCodes([72+base%20,105,8364]),textFromCodes([codes[3],97+base%26])]);
 }
 return values;
}
export function expectedStringFromCodesObservation(id,n,seed) { return observeSliceTree(expectedStringFromCodesValues(id,n,seed)); }
