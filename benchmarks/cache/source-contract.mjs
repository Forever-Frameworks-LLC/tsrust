import {createHash} from 'node:crypto';
import {containsCodeSnippet} from '../runtime/source-contract-tokens.mjs';

export function assertMatchingSource(source, manifest, kind) {
  const native = kind === 'native';
  if (!native && kind !== 'typescript') throw Error('Unknown matching engine');
  const fingerprint = native ? manifest.sourceSha256 : manifest.counterpartSha256;
  if (createHash('sha256').update(source).digest('hex') !== fingerprint) throw Error('Matching source fingerprint changed');
  if (manifest.iterationsPerBatch !== 256) throw Error('Matching operation count changed');
  const required=['for(let i=0;i<256;i+=1)','unwrap(cache.set(i,i*3));','const found=cache.get(i);if(found!==undefined){checksum+=found;}','checksum+=cache.size();cache.clear();return checksum;'];
  if(native&&manifest.shared)required.push('new SharedLruCache<int,int>(16)');
  if(!native)required.push('this.entries.delete(oldest[0]);','this.entries.delete(key);this.entries.set(key,value);');
  for (const original of required) {const text=native&&manifest.shared?original.replace('cache.get(', 'cache.getCloned('):original;if (!containsCodeSnippet(source, text)) throw Error('Matching scenario changed: ' + text);}
}
