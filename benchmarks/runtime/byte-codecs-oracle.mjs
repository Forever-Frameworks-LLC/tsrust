import {observeSliceTree} from './slice-observation.mjs';
import {codecOperations, codecSizes} from './byte-codecs-workloads.mjs';
export function expectedByteCodecValues(id,n,seed,input) {
  const size=codecSizes[input];
  const operation=codecOperations.find(operation=>id===`bytes_${operation}_${input}`);
  if(!operation||!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(seed)||seed<0)throw Error('Byte codec case');
  let values;
  if(operation==='hex_encode')values=[size*2,1,1];
  else if(operation==='base64_encode')values=[Math.ceil(size/3)*4,1,1];
  else if(operation==='base64url_encode')values=[Math.ceil(size*8/6),1,1];
  else if(['hex_decode','base64_decode','base64url_decode'].includes(operation))values=[size,195,98];
  else if(operation==='utf16le_encode')values=[size/8*10,233,0];
  else if(operation==='utf16be_encode')values=[size/8*10,0,98];
  else values=[size+(operation==='utf8_lossy_invalid'?4:0),1,1];
  return Array.from({length:n},()=>values).flat();
}
export function expectedByteCodecObservation(id,n,seed,input) {return observeSliceTree(expectedByteCodecValues(id,n,seed,input));}
