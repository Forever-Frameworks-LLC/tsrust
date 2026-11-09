import assert from 'node:assert/strict';
// Independent integer recurrence and fixed batch result, not engine equality.
let cpuTotal=0;
for(let seed=0;seed<64;seed++){
    let value=seed;
    for(let step=0;step<4096;step++)value=(Math.imul(value,1664525)+1013904223)>>>0;
    cpuTotal+=value;
}
export const checksums=Object.freeze({ready_floor:2016,all_ready:2016,joinset_floor:2016,all_yield:2016,try_direct:2016,try_exclusive:2016,async_direct:2016,async_existing:2016,cap_construct:1,cap_closed_poll:1,cpu_join_floor:cpuTotal,cpu_async:cpuTotal});
export function validateContracts(value,candidate){
    assert.equal(value.kind,'contracts');assert.equal(value.candidate,candidate);
    assert.equal(value.retains_pool_after_owner_drop,candidate==='strong');
    assert.equal(value.started_result,11);assert.equal(value.closed_after_owner_drop,true);assert.equal(value.awaited_cleanup_drops,2);
    assert.ok(Number.isSafeInteger(value.future_bytes)&&value.future_bytes>0);
}
export function validateRecords(records,candidate,instrumented){
    const names=candidate==='strong'?Object.keys(checksums):['cap_construct','cap_closed_poll','cpu_async'];
    assert.deepEqual(records.map(value=>value.kernel),names);
    for(const value of records){
        assert.equal(value.candidate,candidate);
        if(instrumented){
            assert.equal(value.kind,'allocations');assert.equal(value.checksum,checksums[value.kernel]);
            for(const name of ['allocations','reallocations','requested','peak'])assert.ok(Number.isSafeInteger(value[name])&&value[name]>=0);
            assert.ok(Number.isSafeInteger(value.retained));
        }else{
            assert.equal(value.kind,'timing');assert.ok(Number.isSafeInteger(value.rounds)&&value.rounds>0);
            assert.equal(value.durations_ns.length,31);assert.equal(value.checksums.length,31);
            assert.ok(value.durations_ns.every(time=>Number.isSafeInteger(time)&&time>0));
            assert.ok(value.checksums.every(checksum=>checksum===checksums[value.kernel]*value.rounds));
        }
    }
}
