import {createHash} from 'node:crypto';
import {containsCodeSnippet} from '../runtime/source-contract-tokens.mjs';

export function assertMatchingSource(source, manifest, kind) {
  const native = kind === 'native';
  if (!native && kind !== 'typescript') throw Error('Unknown matching engine');
  const fingerprint = native ? manifest.sourceSha256 : manifest.counterpartSha256;
  if (createHash('sha256').update(source).digest('hex') !== fingerprint) throw Error('Matching source fingerprint changed');
  if (manifest.iterationsPerBatch !== 256) throw Error('Matching operation count changed');
  const required=["const id=unwrap(parseUuid('67e55044-10b1-426f-9247-bb680e5fe0c8'));",'for(let i=0;i<256;i+=1)','const bytes=toBytes(id);unwrap(set(bytes,15,i));','const changed=unwrap(fromBytes(bytes));','const restored=unwrap(parseUuid(formatUuid(changed)));','checksum+=version(restored)+unwrap(at(toBytes(restored),15));','return checksum;'];
  if(!native)required.push('text.length===36','value.length===16','return Buffer.from(value);');
  for (const text of required) if (!containsCodeSnippet(source, text)) throw Error('Matching scenario changed: ' + text);
}
