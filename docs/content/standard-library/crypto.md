<!-- tsrust-docs
{"id":"reference-standard-library-crypto","group":"standard-library","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"std:crypto","name":"randomBytes"},{"receiver":"std:crypto","name":"sha256"},{"receiver":"std:crypto","name":"timingSafeEqual"},{"receiver":"std:crypto","name":"CryptoError"},{"receiver":"std:crypto","name":"sha384"},{"receiver":"std:crypto","name":"sha512"},{"receiver":"std:crypto","name":"createHasher"},{"receiver":"std:crypto","name":"Hasher"},{"receiver":"std:crypto","name":"hmacSha256"},{"receiver":"std:crypto","name":"verifyHmacSha256"},{"receiver":"std:crypto","name":"hkdfSha256"},{"receiver":"std:crypto","name":"randomInt"},{"receiver":"std:crypto","name":"hashPassword"},{"receiver":"std:crypto","name":"verifyPassword"},{"receiver":"std:crypto","name":"verifyPasswordBounded"},{"receiver":"std:crypto","name":"PasswordHashOptions"},{"receiver":"std:crypto","name":"PasswordVerifyOptions"},{"receiver":"Hasher","name":"update"},{"receiver":"Hasher","name":"finalize"}]}
-->
# Crypto

`std:crypto` hashes exact readable bytes (`Bytes`, `ByteView`, `ByteViewMut`).
`sha256`, `sha384`, and `sha512` return independent32/48/64-byte digests.
`createHasher('sha256' | 'sha384' | 'sha512')` returns one owned `Hasher`;
`update(bytes)` borrows chunks synchronously; `finalize()` consumes the state.
Empty input/chunks are valid. There is no state clone or post-finalization reuse.

`hmacSha256(key,input)` returns32bytes. `verifyHmacSha256(key,input,tag)` verifies
complete tags with constant-time equal-length comparison; mismatch and truncated
or wrong-length tags returnfalse. Key/message/tag positions all accept readable
views without hidden payload copies. `hkdfSha256(input,salt,info,length)` returns
`Result<Bytes,CryptoError>` for an explicit0..8160-byte output; its upper bound is
algorithmic. It generates no salt. Inputs are never normalized or mutated.

`randomBytes` accepts nonnegative target-representable lengths, includingzero,
without an artificial byte cap. `randomInt(min,max)` returns a fallible unbiased
integer in the half-open signed interval. Invalid bounds fail before entropy;
valid calls use fresh fallible OS-seeded state, with no predictable fallback or
hidden cache. Allocation failures follow ordinary Rust allocation behavior.

Password operations take an explicit caller-owned `ThreadPool` and return lazy
`Task<Result<T,CryptoError | SpawnError | ThreadError<undefined>>>`:

- `hashPassword(pool,password,options?)` generates Argon2id v19 PHC text. Defaults
  are19456KiB,2passes,1lane,32output bytes, fresh16-byte OS salt.
- `PasswordHashOptions` optionally overrides `memoryKiB`, `iterations`,
  `parallelism`, `outputBytes`. Backend cost/PHC output bounds apply.
- `verifyPassword(pool,password,encoded)` follows backend encoded parameters,
  without an ordinary hidden cost limit. Valid mismatch returnsfalse.
- `verifyPasswordBounded(pool,password,encoded,options)` requires explicit
  positive `maxMemoryKiB`, `maxIterations`, `maxParallelism`, comparing parsed
  costs before password copies, submission or expensive workspace allocation.

Unpolled tasks do no password copy/work. First poll validates, creates one
zeroizing worker password copy, and submits to the existing bounded pool.
Verification also copies public PHC text. Pool ownership is not captured: a weak
submission capability is extracted immediately. Input loans remain through Task
settlement. Drop/cancel preserves actual worker settlement and drainage; it does
not interrupt in-progress Argon2 code or release owners early. Zeroization does
not promise caller-buffer or full Argon2 workspace erasure. No secret values are
logged. `CryptoError` codes are Range, Random, Options, Password, Policy;
SpawnError and ThreadError remain distinct typed categories.

See `examples/crypto/primitives.tsr` and `examples/crypto/passwords.tsr` for exact
bytes and explicit pool cleanup. Fast digest/MAC operations are not password
hashing APIs. No encryption, key-exchange or general WebCrypto facade is exposed.

`timingSafeEqual(left,right)` accepts readable byte owners/views in both positions
without input copies. It compares equal-length content with the maintained
constant-time primitive and returns false for unequal lengths; length is public.
No input is mutated or retained.

```tsrust
import {createHasher} from 'std:crypto';
import {fromUtf8,length} from 'std:bytes';
function main(){const state=createHasher('sha256');state.update(unwrap(fromUtf8('abc')));console.log(length(state.finalize()));}
```

Expected output: `32`.
