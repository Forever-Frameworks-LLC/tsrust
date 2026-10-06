type Value = {value: number; active: boolean};
function workload(n: number, seed: number): Value { return {value: seed + n, active: n > seed}; }
function probeChecksum(value: unknown): number { const record = value as Value; return record.value + Number(record.active); }
