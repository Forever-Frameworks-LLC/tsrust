<!-- tsrust-docs
{"id":"reference-standard-library-crypto","group":"standard-library","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"std:crypto","name":"randomBytes"},{"receiver":"std:crypto","name":"sha256"},{"receiver":"std:crypto","name":"timingSafeEqual"},{"receiver":"std:crypto","name":"CryptoError"}]}
-->
# Crypto

Cryptographic operations work on explicit text or byte inputs. Choose the operation and encoding required by your protocol. Keep key material and application credential policy under application ownership.

`randomBytes(length)` returns independently owned `Bytes` filled by the operating system. The count must be nonnegative and representable as a target allocation length. Zero returns empty bytes; there is no library size cap. Negative or target-unrepresentable counts return `CryptoError` code `Range`; operating system failures return `Random`, without a predictable fallback. Allocation follows ordinary Rust behavior and is not converted into a catchable crypto error. Returned memory is not automatically zeroized.

```tsrust
import { randomBytes } from 'std:crypto';
import { length } from 'std:bytes';

function main() {
  console.log(length(unwrap(randomBytes(0)))); // 0
}
```
