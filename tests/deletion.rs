//! Regression tests for deletion probing and the maximum load factor.

use core::hash::{BuildHasher, Hasher};
use hashbrown::HashMap;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Default)]
struct CollisionBuildHasher;

struct CollisionHasher;

impl Hasher for CollisionHasher {
    fn finish(&self) -> u64 {
        0
    }

    fn write(&mut self, _bytes: &[u8]) {}
}

impl BuildHasher for CollisionBuildHasher {
    type Hasher = CollisionHasher;

    fn build_hasher(&self) -> CollisionHasher {
        CollisionHasher
    }
}

#[test]
fn mixed_delete_insert_matches_reference() {
    let mut map: HashMap<u64, u64, CollisionBuildHasher> =
        HashMap::with_hasher(CollisionBuildHasher);
    let mut reference = BTreeMap::new();
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;

    // Force every key through the same probe chain so tombstones and probe
    // termination are exercised on every lookup.
    for round in 0..12 {
        for _ in 0..400 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let key = (state >> 33) % 600;

            if (state >> 17) % 4 == 0 {
                assert_eq!(map.remove(&key), reference.remove(&key));
            } else {
                assert_eq!(
                    map.insert(key, state),
                    reference.insert(key, state),
                    "round {round}, key {key}"
                );
            }
        }

        for key in 0..600 {
            assert_eq!(
                map.get(&key),
                reference.get(&key),
                "round {round}, key {key}"
            );
        }
    }

    assert_eq!(map.len(), reference.len());
}

#[test]
fn grows_at_three_quarters_load() {
    let mut map = HashMap::with_capacity(12);
    assert_eq!(map.capacity(), 12);

    for key in 0..12 {
        assert_eq!(map.insert(key, key), None);
    }
    assert_eq!(map.capacity(), 12);

    assert_eq!(map.insert(12, 12), None);
    assert_eq!(map.capacity(), 24);
}
