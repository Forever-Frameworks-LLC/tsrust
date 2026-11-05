import assert from 'node:assert/strict';
import fs from 'node:fs';
// Independent semantic result, not an engine-to-engine equality assertion.
export const expected={all_order:[3,5],all_void_len:2,all_empty_len:0,sibling_drops:1,timer_policies:['burst','delay','skip'],token_direction:true,send_owner_retained:true,pool_unpolled_slots:0,pool_cleanup_slots:0,pool_drops:4,pool_retained_start:true,pool_owner_drop_closes:true};
if(process.argv[2]) assert.deepEqual(JSON.parse(fs.readFileSync(process.argv[2],'utf8')),expected);
else console.log(JSON.stringify(expected));
