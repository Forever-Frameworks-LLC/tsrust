import {createHash} from 'node:crypto';
import {containsCodeSnippet} from '../runtime/source-contract-tokens.mjs';

export function assertMatchingSource(source, manifest, kind) {
  const native = kind === 'native';
  if (!native && kind !== 'typescript') throw Error('Unknown matching engine');
  const fingerprint = native ? manifest.sourceSha256 : manifest.counterpartSha256;
  if (createHash('sha256').update(source).digest('hex') !== fingerprint) throw Error('Matching source fingerprint changed');
  if (manifest.iterationsPerBatch !== 2048) throw Error('Matching operation count changed');
  const required = ['for(let i=0;i<2048;i+=1)', 'checksum+=value+exact+range;', 'return checksum;'];
  required.push(...(native?['unwrap(toInt(unwrap(toNumber(i))))','match(toFloat32(i)){Ok(_)=>1,Err(_)=>0,}','match(checkSigned(i,8)){Ok(_)=>1,Err(_)=>0,}']:["const integer=BigInt(i);","toFloat32(integer).kind==='Ok'?1:0","checkSigned(integer).kind==='Ok'?1:0"]));
  for (const text of required) if (!containsCodeSnippet(source, text)) throw Error('Matching scenario changed: ' + text);
}
