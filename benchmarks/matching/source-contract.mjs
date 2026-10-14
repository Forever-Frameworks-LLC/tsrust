import {createHash} from 'node:crypto';
import {containsCodeSnippet} from '../runtime/source-contract-tokens.mjs';

export function assertMatchingSource(source, manifest, kind) {
  const native = kind === 'native';
  if (!native && kind !== 'typescript') throw Error('Unknown matching engine');
  const fingerprint = native ? manifest.sourceSha256 : manifest.counterpartSha256;
  if (createHash('sha256').update(source).digest('hex') !== fingerprint) throw Error('Matching source fingerprint changed');
  if (manifest.iterationsPerBatch !== 2048) throw Error('Matching operation count changed');
  const required = ['for(let i=0;i<2048;i+=1)', 'checksum+=value+weight+extra;', 'return checksum;'];
  required.push(...(native ? [
    'match(result(i)){Ok(value)=>value,Err(error)=>-error,}',
    'match(i%4){0=>1,1=>3,2=>5,_=>7,}',
    'match(person(i)){User(user)=>user.value,Guest(guest)=>guest.value,}',
  ] : [
    "selected.kind==='Ok'?selected.value:-selected.error",
    'switch(i%4)', 'switch(selectedPerson.kind)',
  ]));
  for (const text of required) if (!containsCodeSnippet(source, text)) throw Error('Matching scenario changed: ' + text);
}
