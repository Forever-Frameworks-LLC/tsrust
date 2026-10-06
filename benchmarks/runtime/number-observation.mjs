// Runtime/driver observation only, outside the workload timer.
export function observeNumber(output) {
 const values=Array.isArray(output)?output:[output], view=new DataView(new ArrayBuffer(8));
 let checksum=0;
 const tokens=[];
 for(const value of values) {
  if(typeof value!=='number')throw new Error('Number result required');
  const nan=Number.isNaN(value);
  view.setFloat64(0,nan?NaN:value,false);
  const bits=nan?0x7ff8000000000000n:view.getBigUint64(0,false);
  tokens.push(nan?'nan':bits.toString(16).padStart(16,'0'));
  for(let shift=56n;shift>=0n;shift-=8n)checksum=(checksum*131+Number((bits>>shift)&255n))%1000000007;
 }
 return {checksum,exactBits:(Array.isArray(output)?'a:':'n:')+tokens.join(',')};
}

export function isNumberResult(workload) { return workload.result === 'number' || workload.result === 'number[]'; }
// NaN payload/sign is canonical; zero and infinity signs remain exact.
export function validateNumberObservation(actual, expected) {
 if (!Number.isSafeInteger(actual?.checksum) || typeof actual?.exactBits !== 'string' || !/^(n:(?:[0-9a-f]{16}|nan)|a:(?:(?:[0-9a-f]{16}|nan)(?:,(?:[0-9a-f]{16}|nan))*)?)$/.test(actual.exactBits)) throw new Error('Invalid exact Number observation');
 if (actual.checksum !== expected.checksum || actual.exactBits !== expected.exactBits) throw new Error('Exact Number result disagrees with independent oracle');
}
