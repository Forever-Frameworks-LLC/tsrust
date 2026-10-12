import assert from 'node:assert/strict';
export function validateEncodingTiming(row,expected){
 assert.equal(row.checksum,expected);
 assert.equal(row.iterations,100);
 assert.equal(row.warmups,8);
 assert.ok(Number.isFinite(row.coldMs)&&row.coldMs>=0);
 assert.equal(row.durations.length,31);
 assert.ok(row.durations.every(value=>Number.isFinite(value)&&value>0));
}
