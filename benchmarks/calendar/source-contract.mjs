import {createHash} from 'node:crypto';
import {containsCodeSnippet} from '../runtime/source-contract-tokens.mjs';

export function assertMatchingSource(source, manifest, kind) {
  const native = kind === 'native';
  if (!native && kind !== 'typescript') throw Error('Unknown matching engine');
  const fingerprint = native ? manifest.sourceSha256 : manifest.counterpartSha256;
  if (createHash('sha256').update(source).digest('hex') !== fingerprint) throw Error('Matching source fingerprint changed');
  if (manifest.iterationsPerBatch !== 256) throw Error('Matching operation count changed');
  const required=["const base=unwrap(parseDate('2024-01-31'));",'for(let i=0;i<256;i+=1)','const due=unwrap(addMonths(base,1));','const shifted=unwrap(addDays(due,i));','const restored=unwrap(parseDate(formatDate(shifted)));','checksum+=differenceInCalendarDays(restored,base);','return checksum;'];
  if(!native)required.push("process.env.TZ='UTC';",'value.getFullYear()>=1&&value.getFullYear()<=9999');
  for (const text of required) if (!containsCodeSnippet(source, text)) throw Error('Matching scenario changed: ' + text);
}
