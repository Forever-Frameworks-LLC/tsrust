import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
export const localeSpecs={
  "locale_canonicalize": {
    "input": "iw-IL",
    "weight": 5,
    "native": "const value=unwrap(parseLocale(input));checksum+=formatLocale(value).length;",
    "typescript": "const value=Intl.getCanonicalLocales(input)[0];checksum+=value.length;"
  },
  "locale_format": {
    "input": "iw-IL",
    "weight": 5,
    "native": "const text=formatLocale(value);checksum+=text.length;",
    "typescript": "const text=value;checksum+=text.length;"
  },
  "locale_collator_construct": {
    "input": "é2",
    "weight": 3,
    "native": "const value=unwrap(createCollator(locale));checksum+=value.compare(input,'e10')+2;",
    "typescript": "const value=new Intl.Collator(locale);checksum+=Math.sign(value.compare(input,'e10'))+2;"
  },
  "locale_compare": {
    "input": "é2",
    "weight": 3,
    "native": "checksum+=value.compare(input,'e10')+2;",
    "typescript": "checksum+=Math.sign(value.compare(input,'e10'))+2;"
  },
  "string_locale_compare": {
    "input": "é2",
    "weight": 3,
    "native": "checksum+=input.localeCompare('e10',value)+2;",
    "typescript": "checksum+=Math.sign(value.compare(input,'e10'))+2;"
  },
  "locale_segmenter_construct": {
    "input": "é🦀👨‍👩‍👧‍👦",
    "weight": 55,
    "native": "const value=createSegmenter();const ranges=value.segment(input);for(const range of ranges){checksum+=range.start+range.end+1;}",
    "typescript": "const value=new Intl.Segmenter('und',{granularity:'grapheme'});const ranges=segmentRanges(value,input);for(const range of ranges){checksum+=range.start+range.end+1;}"
  },
  "locale_segment": {
    "input": "é🦀👨‍👩‍👧‍👦",
    "weight": 55,
    "native": "const ranges=value.segment(input);for(const range of ranges){checksum+=range.start+range.end+1;}",
    "typescript": "const ranges=segmentRanges(value,input);for(const range of ranges){checksum+=range.start+range.end+1;}"
  },
  "locale_number_construct": {
    "input": "1234",
    "weight": 5,
    "native": "const value=unwrap(createNumberFormat(locale));checksum+=value.format(1234).length;",
    "typescript": "const value=new Intl.NumberFormat(locale);checksum+=value.format(1234).length;"
  },
  "locale_number_int": {
    "input": "1234",
    "weight": 5,
    "native": "checksum+=value.format(1234).length;",
    "typescript": "checksum+=value.format(1234).length;"
  },
  "locale_number_decimal": {
    "input": "12345678901234567890.12",
    "weight": 29,
    "native": "checksum+=value.format(decimal).length;",
    "typescript": "checksum+=value.format(input as unknown as number).length;"
  },
  "locale_number_fixed": {
    "input": "2.345",
    "weight": 4,
    "native": "checksum+=value.format(decimal).length;",
    "typescript": "checksum+=value.format(input as unknown as number).length;"
  }
};
export function localeChecksum(id,n,seed,input){const spec=localeSpecs[id];assert(spec);assert.equal(input,spec.input);assert(Number.isSafeInteger(n)&&n>=0&&Number.isSafeInteger(seed));const result=seed+n*spec.weight;assert(Number.isSafeInteger(result));return result;}
export function assertLocaleSource(source,row,kind){const spec=localeSpecs[row.id];assert(spec);assert.equal(row.family,'explicit-locale');assert.equal(row.operationsPerIteration,1);assert.equal(row.inputText,spec.input);assert.deepEqual(row.parameters,['int','int','string']);assert.equal(row.result,'int');assert.equal(createHash('sha256').update(source).digest('hex'),row[kind==='native'?'sourceSha256':'counterpartSha256']);for(const snippet of ['let checksum=seed;','while(i<n)','i++;','return checksum;',spec[kind]])assert(containsCodeSnippet(source,snippet),row.id+':'+snippet);}
export function selfCheck(){assert.equal(Object.keys(localeSpecs).length,11);assert.equal(Intl.getCanonicalLocales('iw-IL')[0].length,5);assert.equal(Math.sign(new Intl.Collator('en').compare('é2','e10'))+2,3);let offset=0;const ranges=Array.from(new Intl.Segmenter('und',{granularity:'grapheme'}).segment(localeSpecs.locale_segment.input),({segment})=>{const start=offset;offset+=Buffer.byteLength(segment);return{start,end:offset};});assert.equal(ranges.reduce((n,r)=>n+r.start+r.end+1,0),55);assert.equal(new Intl.NumberFormat('en',{maximumFractionDigits:28}).format(localeSpecs.locale_number_decimal.input).length,29);assert.equal(new Intl.NumberFormat('en',{useGrouping:false,minimumFractionDigits:2,maximumFractionDigits:2,roundingMode:'halfEven'}).format('2.345'),'2.34');}
