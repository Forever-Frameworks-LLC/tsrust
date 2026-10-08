import {createHash} from 'node:crypto';

// Formatting changes exact source identities. Registration guards check those
// identities separately; this contract pins every other workload field and order.
export function manifestMetadataHash(rows) {
    const metadata = rows.map(({sourceSha256, counterpartSha256, ...row}) => row);
    return createHash('sha256').update(JSON.stringify(metadata)).digest('hex');
}
