import {createHash} from 'node:crypto';

// Source guards check exact current hashes. This history proof freezes workload
// protocols without treating verified formatting as a new measurement protocol.
export function manifestProtocolFingerprint(rows) {
    const protocols=rows.map(({sourceSha256, counterpartSha256, ...protocol})=>protocol);
    return createHash('sha256').update(JSON.stringify(protocols)).digest('hex');
}
