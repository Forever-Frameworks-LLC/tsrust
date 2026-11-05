import {createHash} from 'node:crypto';
import fs from 'node:fs';

// Registration fragments retain their operation contracts; current byte identities
// belong to the live manifest refreshed by the token-preserving formatter.
export function withCurrentSourceFingerprints(value) {
    const current = new Map(JSON.parse(fs.readFileSync(new URL('./manifest.json',import.meta.url),'utf8')).workloads.map(row=>[row.id,row]));
    const select = row => {
        const live=current.get(row.id);
        if(!live)throw Error('Unknown workload: '+row.id);
        const updated={...row};
        for(const [key,pathKey,extension]of [['sourceSha256','source','tsr'],['counterpartSha256','counterpart','ts']]) {
            if(!row[key])continue;
            const fallback='workloads/'+row.id+'.'+extension;
            if((row[pathKey]??fallback)!==(live[pathKey]??fallback)||!live[key])throw Error('Workload source identity requires review: '+row.id);
            updated[key]=live[key];
        }
        return updated;
    };
    return Array.isArray(value)?value.map(select):select(value);
}

// Formatting changes exact source identities. Registration guards check those
// identities separately; this contract pins every other workload field and order.
export function manifestMetadataHash(rows) {
    const metadata = rows.map(({sourceSha256, counterpartSha256, ...row}) => row);
    return createHash('sha256').update(JSON.stringify(metadata)).digest('hex');
}
