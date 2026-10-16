import {createHash} from 'node:crypto';
import {containsCodeSnippet} from '../runtime/source-contract-tokens.mjs';

export function assertMatchingSource(source, manifest, kind) {
  const native = kind === 'native';
  if (!native && kind !== 'typescript') throw Error('Unknown matching engine');
  const fingerprint = native ? manifest.sourceSha256 : manifest.counterpartSha256;
  if (createHash('sha256').update(source).digest('hex') !== fingerprint) throw Error('Matching source fingerprint changed');
  if (manifest.iterationsPerBatch !== 256) throw Error('Matching operation count changed');
  const required=['for(let i=0;i<256;i+=1)',"const price=unwrap(parseDecimal('19.99'));","const fee=unwrap(parseDecimal('0.01'));",'const adjusted=unwrap(add(total,fee));','const cents=unwrap(multiply(adjusted,hundred));','return checksum;'];
  required.push(...(native?['checksum+=unwrap(toInt(cents));']:['Decimal.set({precision:100});','checksum+=Number(unwrap(toInt(cents)));','value.sd()<=38&&value.dp()<=28&&value.e<38']));
  for (const text of required) if (!containsCodeSnippet(source, text)) throw Error('Matching scenario changed: ' + text);
}
