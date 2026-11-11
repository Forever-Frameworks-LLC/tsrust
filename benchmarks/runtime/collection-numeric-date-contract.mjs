import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
export const completionSpecs={
  "collection_endpoints": {
    "native": "values.first()!+values.last()!+map.firstKey()!+map.lastKey()!",
    "typescript": "keys[keys.length-1]",
    "operations": 4
  },
  "ordered_set_snapshots": {
    "native": "set.forEach((value,key,source)=>",
    "typescript": "set.clear();",
    "operations": 4
  },
  "deque_values_snapshot": {
    "native": "const rows=deque.values();",
    "typescript": "deque.map(row=>row.slice())",
    "operations": 1
  },
  "heap_sorted_snapshot": {
    "native": "heap.toSortedArray()",
    "typescript": "heap.slice().sort((a,b)=>a-b)",
    "operations": 1
  },
  "numeric_strict_parse": {
    "native": "exact('7fffffffffffffff',16)",
    "typescript": "BigInt('0x7fffffffffffffff')",
    "operations": 3
  },
  "numeric_checked_arithmetic": {
    "native": "checkedRemainder(a,3)",
    "typescript": "a%three",
    "operations": 5
  },
  "date_completion_access": {
    "native": "getNanoseconds(time)",
    "typescript": "time.nanoseconds",
    "operations": 7
  }
};
export function completionChecksum(id,n,seed){
 assert(Object.hasOwn(completionSpecs,id));assert(Number.isSafeInteger(n)&&n>=0&&Number.isSafeInteger(seed));let sum=BigInt(seed);
 for(let i=0;i<n;i++){
  const s=BigInt(seed),j=BigInt(i);
  if(id==='collection_endpoints')sum+=4n*s+8n;
  else if(id==='ordered_set_snapshots')sum+=30n*s+77n;
  else if(id==='deque_values_snapshot')sum+=3n*(s+j)+15n;
  else if(id==='heap_sorted_snapshot')sum+=36n*(s+j)+213n;
  else if(id==='numeric_strict_parse')sum+=121n;
  else if(id==='numeric_checked_arithmetic'){const a=s+j+10n;sum+=5n*a+a/3n+a%3n;}
  else if(id==='date_completion_access')sum+=123456933n;
 }
 const result=Number(sum);assert(Number.isSafeInteger(result));return result;
}
export function assertCompletionSource(source,row,kind){
 const spec=completionSpecs[row.id];assert(spec&&['native','typescript'].includes(kind));assert.equal(row.family,'collection-numeric-date');assert.equal(row.iterations,2048);assert.equal(row.operationsPerIteration,spec.operations);assert.deepEqual(row.parameters,['int','int']);assert.equal(row.result,'int');
 assert.equal(createHash('sha256').update(source).digest('hex'),row[kind==='native'?'sourceSha256':'counterpartSha256']);
 for(const text of ['let checksum=seed;','while(i<n)','i++;','return checksum;',spec[kind]])assert(containsCodeSnippet(source,text),row.id+': '+text);
 if(row.id==='date_completion_access'&&kind==='typescript'){for(const text of ["const text='23:59:58.123456789'",'matched===null',...['hours','minutes','seconds','nanoseconds'].map((field,index)=>field+':Number(matched['+(index+1)+'])'),'time.hours>23||time.minutes>59||time.seconds>59'])assert(containsCodeSnippet(source,text),'Node time parser missing '+text);assert(source.includes('/^(\\d{2}):(\\d{2}):(\\d{2})\\.(\\d{9})$/u.exec(text)'),'Node whole-input time grammar changed');}
}

// Finite authored controls independently retain their backend operations and every observation.
export const completionReferenceOperations={
 collection_endpoints:['std::collections::BTreeSet::from','std::collections::BTreeMap::from','values.first().unwrap()+values.last().unwrap()+map.first_key_value().unwrap().0+map.last_key_value().unwrap().0'],
 ordered_set_snapshots:['let values:Vec<_>=set.iter().copied().collect()','let keys:Vec<_>=set.iter().copied().collect()','set.iter().map(|v|(*v,*v)).collect()','let calls:i64=set.iter().map(|v|v+v+set.len() as i64).sum()','set.clear()','checksum+=(j as i64+1)*(values[j]+keys[j]+entries[j].0+entries[j].1)','checksum+=calls+set.len() as i64'],
 deque_values_snapshot:['deque.pop_front()','deque.push_back(vec![value1+i+2])','deque.iter().cloned().collect()','rows[0].push(9)','checksum+=rows[0][0]+rows[0][1]+2*rows[1][0]+deque.front().unwrap().len() as i64'],
 heap_sorted_snapshot:['std::collections::BinaryHeap<_>','heap.iter().copied().collect()','rows.sort_unstable()','checksum+=(j as i64+1)*v','checksum+=heap.len() as i64'],
 numeric_strict_parse:['i64::from_str_radix("7fffffffffffffff",16).unwrap()','i64::from_str_radix("-100",2).unwrap()','"1.25e2".parse::<f64>().unwrap()','assert_eq!(integer,i64::MAX)','assert_eq!(number,125.0)','checksum+=negative+125'],
 numeric_checked_arithmetic:['a.checked_add(3).unwrap()+a.checked_sub(3).unwrap()+a.checked_mul(3).unwrap()+a.checked_div(3).unwrap()+a.checked_rem(3).unwrap()'],
 date_completion_access:['chrono::NaiveDate::parse_from_str("2024-02-29","%Y-%m-%d").unwrap()','chrono::NaiveTime::parse_from_str("23:59:58.123456789","%H:%M:%S%.f").unwrap()','date.with_day(1).unwrap().with_month(1).unwrap()','date.with_day(1).unwrap().with_month(12).unwrap().with_day(31).unwrap()','assert_eq!(start.to_string(),"2024-01-01")','assert_eq!(end.to_string(),"2024-12-31")','date.weekday().num_days_from_sunday()+time.hour()+time.minute()+time.second()+time.nanosecond()'],
};
export function assertCompletionReference(source,row){
 assert(Object.hasOwn(completionSpecs,row.id));assert.equal(row.family,'collection-numeric-date');
 assert.equal(createHash('sha256').update(source).digest('hex'),row.rustReferenceSha256,'Completion reference source identity changed');
 for(const text of ['fn function0(value0:i64,value1:i64)->i64','let mut checksum=value1','for i in 0..value0',...completionReferenceOperations[row.id]])assert(containsCodeSnippet(source,text),row.id+': reference '+text);
 assert.match(source,/checksum\s*}\s*fn main\(\)\s*{\s*}/,'Completion reference must return observed checksum');
}
