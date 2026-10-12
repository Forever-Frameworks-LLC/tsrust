import assert from 'node:assert/strict';
export function validateEncodingTiming(row,expected,iterations=100){
 assert.equal(row.checksum,expected);
 assert.equal(row.iterations,iterations);
 assert.equal(row.warmups,8);
 assert.ok(Number.isFinite(row.coldMs)&&row.coldMs>=0);
 assert.equal(row.durations.length,31);
 assert.ok(row.durations.every(value=>Number.isFinite(value)&&value>0));
}

const defaultKinds=['encode_typed','encode_dynamic','encode_typed_strings','encode_dynamic_strings','encode_dynamic_deep','encode_dynamic_object','encode_typed_object'];
export function encodingSelection(args){
 let rss=false,selected;
 for(const arg of args){
  if(arg==='--rss'&&!rss){rss=true;continue;}
  if(arg.startsWith('--only=')&&selected===undefined){selected=arg.slice(7).split(',');continue;}
  throw Error('Usage: json-encoding-timing.mjs [--rss] [--only=kind,kind]');
 }
 const kinds=selected??defaultKinds;
 const supported=[...defaultKinds,'encode_raw','encode_raw_strings','encode_raw_deep'];
 if(new Set(kinds).size!==kinds.length||kinds.some(kind=>!supported.includes(kind)))throw Error('Unknown or duplicate encoding kind');
 return {rss,kinds:[...kinds]};
}
