// Appended driver observation, outside timed workload execution.
function observeNumber(output: number | number[]): {checksum: number; exactBits: string} {
    const values = Array.isArray(output) ? output : [output];
    const view = new DataView(new ArrayBuffer(8));
    let checksum = 0; const tokens: string[] = [];
    for (const value of values) {
        const nan = Number.isNaN(value);
        view.setFloat64(0, nan ? NaN : value, false);
        const bits = nan ? 0x7ff8000000000000n : view.getBigUint64(0, false);
        tokens.push(nan ? 'nan' : bits.toString(16).padStart(16, '0'));
        for (let shift = 56n; shift >= 0n; shift -= 8n) checksum = (checksum * 131 + Number((bits >> shift) & 255n)) % 1000000007;
    }
    return {checksum, exactBits: (Array.isArray(output) ? 'a:' : 'n:') + tokens.join(',')};
}
