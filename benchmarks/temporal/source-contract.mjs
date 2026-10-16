import {createHash} from 'node:crypto';
import {containsCodeSnippet} from '../runtime/source-contract-tokens.mjs';

export function assertMatchingSource(source, manifest, kind) {
  const native = kind === 'native';
  if (!native && kind !== 'typescript') throw Error('Unknown matching engine');
  const fingerprint = native ? manifest.sourceSha256 : manifest.counterpartSha256;
  if (createHash('sha256').update(source).digest('hex') !== fingerprint) throw Error('Matching source fingerprint changed');
  if (manifest.iterationsPerBatch !== 256) throw Error('Matching operation count changed');
  const required=["const base=unwrap(parseISO('2024-01-01T00:00:00Z'));",'for(let i=0;i<256;i+=1)','const shifted=unwrap(addDuration(base,unwrap(durationFromMilliseconds(i))));','const restored=unwrap(parseISO(formatISO(shifted)));','checksum+=unwrap(toUnixMilliseconds(restored))-1704067200000;','return checksum;'];
  if(!native)required.push("process.env.TZ='UTC';",'value.getUTCFullYear()>=1&&value.getUTCFullYear()<=9999');
  for (const text of required) if (!containsCodeSnippet(source, text)) throw Error('Matching scenario changed: ' + text);
}
