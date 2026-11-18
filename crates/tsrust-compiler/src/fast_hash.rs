//! Fast non-cryptographic hashing for compiler-internal tables.
//!
//! Analysis tables key on small interned identities such as `syntax::Type`. The
//! standard SipHash `RandomState` cost about a fifth of whole-program check time on
//! the GPUI starter. These tables never hold attacker-chosen keys from a network,
//! so the multiply-rotate hash used by rustc's own `FxHasher` is appropriate.
use std::hash::{BuildHasherDefault, Hasher};

#[derive(Default, Clone, Copy)]
pub(crate) struct FastHasher(u64);

const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

impl FastHasher {
    #[inline]
    fn add(&mut self, word: u64) {
        self.0 = (self.0.rotate_left(5) ^ word).wrapping_mul(SEED);
    }
}

impl Hasher for FastHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut chunks = bytes.chunks_exact(8);
        for chunk in &mut chunks {
            self.add(u64::from_le_bytes(chunk.try_into().unwrap()));
        }
        let rest = chunks.remainder();
        if !rest.is_empty() {
            let mut word = [0u8; 8];
            word[..rest.len()].copy_from_slice(rest);
            self.add(u64::from_le_bytes(word));
        }
    }
    #[inline] fn write_u8(&mut self, value: u8) { self.add(value as u64); }
    #[inline] fn write_u16(&mut self, value: u16) { self.add(value as u64); }
    #[inline] fn write_u32(&mut self, value: u32) { self.add(value as u64); }
    #[inline] fn write_u64(&mut self, value: u64) { self.add(value); }
    #[inline] fn write_usize(&mut self, value: usize) { self.add(value as u64); }
    #[inline] fn finish(&self) -> u64 { self.0 }
}

pub(crate) type FastBuild = BuildHasherDefault<FastHasher>;
pub(crate) type FastSet<T> = std::collections::HashSet<T, FastBuild>;
pub(crate) type FastMap<K, V> = std::collections::HashMap<K, V, FastBuild>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn equal_values_hash_equally_and_small_identities_spread() {
        let hash = |value: &dyn Fn(&mut FastHasher)| { let mut hasher = FastHasher::default(); value(&mut hasher); hasher.finish() };
        assert_eq!(hash(&|h| h.write(b"record")), hash(&|h| h.write(b"record")));
        let mut set = FastSet::default();
        for id in 0..10_000usize { set.insert(id); }
        assert_eq!(set.len(), 10_000);
        assert!((0..10_000usize).all(|id| set.contains(&id)));
        // hashbrown indexes buckets with the low bits; an odd multiplier keeps them distinct.
        let buckets = (0..1024u64).map(|id| hash(&|h| h.write_u64(id)) & 1023).collect::<std::collections::HashSet<_>>();
        assert_eq!(buckets.len(), 1024);
    }
}
