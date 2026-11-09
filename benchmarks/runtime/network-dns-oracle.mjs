import assert from 'node:assert/strict';import {lookup} from 'node:dns/promises';
// Environmental resolver observations are correctness only, never throughput.
// Native and Node both resolve first, then apply the family filter.
export async function checkDnsObservations(rows){assert.equal(rows.length,4);for(const row of rows){assert(['127.0.0.1','::1'].includes(row.host));assert(['any','ipv4','ipv6'].includes(row.family));const resolved=await lookup(row.host,{all:true,verbatim:true});const expected=resolved.filter(value=>row.family==='any'||value.family===(row.family==='ipv4'?4:6)).map(value=>value.address);assert.deepEqual(row.values,expected);}}
