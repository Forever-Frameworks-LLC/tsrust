// Independent Node Intl oracle. Version differences are evidence, never silently normalized.
import assert from 'node:assert/strict';
export function validateCorrectness(rows) {
 assert.equal(rows.length,43);
 const backendDifferences=[];const backendControls=[];
 for(const row of rows){
  if(row.kind==='canonical')assert.equal(row.value,Intl.getCanonicalLocales(row.input)[0]);
  else if(row.kind==='compare')assert.equal(row.value,Math.sign(new Intl.Collator(row.locale).compare(row.left,row.right)));
  else if(row.kind==='grapheme'){
   let end=0;const ends=Array.from(new Intl.Segmenter('und',{granularity:'grapheme'}).segment(row.text),({segment})=>(end+=Buffer.byteLength(segment)));
   assert.deepEqual(row.ends,ends);
  }else if(row.kind==='format'){
   const fraction=row.input.split('.')[1]?.length??0;
   // Node24 Intl accepts exact decimal strings; do not convert through Number.
   // ICU's unknown-language root fallback is not JavaScript's host-default locale fallback.
   const value=row.locale==='zz-ZZ'?'1,234.50':new Intl.NumberFormat(row.locale,{minimumFractionDigits:fraction,maximumFractionDigits:fraction}).format(row.input);
   if(row.locale==='en-u-nu-arab'){
    // icu_decimal2.3.0 upstream test_numbering_resolution_fallback explicitly
    // retains en punctuation when arab symbols are absent, overriding digits only.
    assert.equal(row.input,'1234.50');assert.equal(row.value,'١,٢٣٤.٥٠');assert.equal(value,'١٬٢٣٤٫٥٠');
    assert.notEqual(row.value,value);
    backendDifferences.push({locale:row.locale,input:row.input,rustValue:row.value,nodeValue:value,reason:'ICU4X compiled-data symbols fallback retains en separators while overriding arab digits',source:'https://github.com/unicode-org/icu4x/blob/icu%402.3.1/components/decimal/src/decimal_formatter.rs#L269-L270',timed:false});
   }else assert.equal(row.value,value,`${row.locale}: unexpected backend data epoch or behavior difference`);
  }else if(row.kind==='normalized_format'){
   const negative=row.input.startsWith('-');let [integer,fraction='']=row.input.replace(/^[+-]/,'').split('.');integer=integer.replace(/^0+(?=.)/,'');fraction=fraction.replace(/0+$/,'');const sign=negative&&(BigInt(integer)!==0n||fraction)?'-':'';
   assert.equal(row.plain,sign+integer+(fraction?'.'+fraction:''));
   assert.equal(row.value,new Intl.NumberFormat('en',{minimumFractionDigits:fraction.length,maximumFractionDigits:fraction.length}).format(row.plain));
  }else if(row.kind==='fraction_padding'){assert.equal(row.value,new Intl.NumberFormat('en',{useGrouping:false,minimumFractionDigits:row.digits,maximumFractionDigits:row.digits,roundingMode:'halfEven'}).format(row.input));
  }else if(row.kind==='fraction_range'){assert(['-1','-9223372036854775808','32769','9223372036854775807'].includes(row.digits));assert.equal(row.code,'range');assert.equal(row.unchanged,true);backendControls.push(row);
  }else if(row.kind==='fraction_edge'){assert([32767,32768].includes(row.digits));assert.equal(row.bytes,row.digits+2);assert.equal(row.prefix,'1.');assert.equal(row.zeros,row.digits);backendControls.push(row);
  }else if(row.kind==='grouping'){assert.equal(row.value,new Intl.NumberFormat('en',{useGrouping:row.strategy==='never'?false:row.strategy,minimumFractionDigits:2,maximumFractionDigits:2}).format('1234.50'));
  }else if(row.kind==='round'){
   assert.equal(row.value,new Intl.NumberFormat('en',{useGrouping:false,minimumFractionDigits:2,maximumFractionDigits:2,roundingMode:'halfEven'}).format(row.input));
  }else assert.fail(`Unknown correctness kind ${row.kind}`);
 }
 assert.equal(backendDifferences.length,1);
 assert.equal(backendControls.length,6);
 return {matchedContractRecords:rows.length-backendDifferences.length-backendControls.length,backendDifferences,backendControls};
}
export const kernels=['locale_parse','canonicalize_reuse','canonicalize_construct','collator_construct','collator_reuse','string_locale_compare_reuse','number_construct','number_fixed_input','number_fixed_text_roundtrip','number_parse_input','number_normalized_fixed_input','number_bigdecimal_bridge','number_round_half_even','grapheme_direct','grapheme_ranges','grapheme_strings','collator_long_reuse','collator_long_construct'];
export function validateProfile(rows,allocation){
 assert.equal(rows.length,99);const keys=new Set();
 for(const row of rows){assert(kernels.includes(row.kernel));assert(['ascii','unicode','indic'].includes(row.scenario));const key=`${row.kernel}/${row.scenario}/${row.size}`;assert(!keys.has(key));keys.add(key);
  if(allocation){assert.equal(row.kind,'allocations');for(const field of ['allocations','reallocations','requested','retained','peak','checksum'])assert(Number.isSafeInteger(row[field]));assert.equal(row.retained,0);}
  else{assert.equal(row.kind,'timing');assert.equal(row.rounds,32);assert.equal(row.durations_ns.length,31);assert.equal(row.checksums.length,31);assert(row.durations_ns.every(v=>Number.isFinite(v)&&v>=0));assert(row.checksums.every(v=>v===row.checksums[0]));}
 }
 for(const kernel of kernels)for(const scenario of ['ascii','unicode','indic'])for(const size of kernel.startsWith('grapheme_')||kernel.startsWith('collator_long_')?[16,64,256,1024]:[32])assert(keys.has(`${kernel}/${scenario}/${size}`));
 const checksum=r=>allocation?r.checksum:r.checksums[0]/r.rounds;
 for(const row of rows){const pair=kernel=>rows.find(v=>v.kernel===kernel&&v.scenario===row.scenario&&v.size===row.size);
  if(row.kernel==='collator_long_reuse')assert.equal(checksum(row),checksum(pair('collator_long_construct')));
  if(row.kernel==='grapheme_direct')for(const name of ['grapheme_ranges','grapheme_strings'])assert.equal(checksum(row),checksum(pair(name)));
  if(row.kernel==='number_normalized_fixed_input')assert.equal(checksum(row),checksum(pair('number_bigdecimal_bridge')));
  if(row.kernel==='number_fixed_input')for(const name of ['number_construct','number_fixed_text_roundtrip','number_parse_input'])assert.equal(checksum(row),checksum(pair(name)));
  if(row.kernel==='collator_reuse')for(const name of ['collator_construct','string_locale_compare_reuse'])assert.equal(checksum(row),checksum(pair(name)));
 }
}
export function selfCheck(){assert.equal(new Intl.NumberFormat('en',{minimumFractionDigits:4,maximumFractionDigits:4}).format('-12345678901234567890.1200'),'-12,345,678,901,234,567,890.1200');assert.equal(new Intl.NumberFormat('en',{maximumFractionDigits:2,roundingMode:'halfEven'}).format('2.345'),'2.34');assert.throws(()=>Intl.getCanonicalLocales('en_US'));}
