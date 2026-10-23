//! Non-cryptographic hashing.

/// FNV-1a 64-bit hash.
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// FNV-1a 32-bit hash.
pub fn fnv1a32(bytes: &[u8]) -> u32 {
    let mut hash = 0x811c_9dc5u32;
    for byte in bytes {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

const XXH_PRIME1: u64 = 0x9e37_79b1_85eb_ca87;
const XXH_PRIME2: u64 = 0xc2b2_ae3d_27d4_eb4f;
const XXH_PRIME3: u64 = 0x1656_67b1_9e37_79f9;
const XXH_PRIME4: u64 = 0x85eb_ca77_c2b2_ae63;
const XXH_PRIME5: u64 = 0x27d4_eb2f_1656_67c5;

#[inline]
fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    let mut value = [0u8; 8];
    value.copy_from_slice(&bytes[offset..offset + 8]);
    u64::from_le_bytes(value)
}

#[inline]
fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    let mut value = [0u8; 4];
    value.copy_from_slice(&bytes[offset..offset + 4]);
    u32::from_le_bytes(value)
}

#[inline]
fn round(acc: u64, input: u64) -> u64 {
    acc.wrapping_add(input.wrapping_mul(XXH_PRIME2))
        .rotate_left(31)
        .wrapping_mul(XXH_PRIME1)
}

/// xxHash64, a fast hash suitable for hash tables and cache keys.
pub fn xxh64(bytes: &[u8], seed: u64) -> u64 {
    let mut offset = 0;
    let mut hash;
    if bytes.len() >= 32 {
        let mut v1 = seed.wrapping_add(XXH_PRIME1).wrapping_add(XXH_PRIME2);
        let mut v2 = seed.wrapping_add(XXH_PRIME2);
        let mut v3 = seed;
        let mut v4 = seed.wrapping_sub(XXH_PRIME1);
        while offset + 32 <= bytes.len() {
            v1 = round(v1, read_u64(bytes, offset));
            v2 = round(v2, read_u64(bytes, offset + 8));
            v3 = round(v3, read_u64(bytes, offset + 16));
            v4 = round(v4, read_u64(bytes, offset + 24));
            offset += 32;
        }
        hash = v1
            .rotate_left(1)
            .wrapping_add(v2.rotate_left(7))
            .wrapping_add(v3.rotate_left(12))
            .wrapping_add(v4.rotate_left(18));
        hash = merge_round(hash, v1);
        hash = merge_round(hash, v2);
        hash = merge_round(hash, v3);
        hash = merge_round(hash, v4);
    } else {
        hash = seed.wrapping_add(XXH_PRIME5);
    }
    hash = hash.wrapping_add(bytes.len() as u64);
    while offset + 8 <= bytes.len() {
        let k1 = round(0, read_u64(bytes, offset));
        hash ^= k1;
        hash = hash
            .rotate_left(27)
            .wrapping_mul(XXH_PRIME1)
            .wrapping_add(XXH_PRIME4);
        offset += 8;
    }
    if offset + 4 <= bytes.len() {
        hash ^= (read_u32(bytes, offset) as u64).wrapping_mul(XXH_PRIME1);
        hash = hash
            .rotate_left(23)
            .wrapping_mul(XXH_PRIME2)
            .wrapping_add(XXH_PRIME3);
        offset += 4;
    }
    while offset < bytes.len() {
        hash ^= (bytes[offset] as u64).wrapping_mul(XXH_PRIME5);
        hash = hash.rotate_left(11).wrapping_mul(XXH_PRIME1);
        offset += 1;
    }
    avalanche(hash)
}

#[inline]
fn merge_round(mut acc: u64, value: u64) -> u64 {
    let value = round(0, value);
    acc ^= value;
    acc.wrapping_mul(XXH_PRIME1).wrapping_add(XXH_PRIME4)
}

#[inline]
fn avalanche(mut hash: u64) -> u64 {
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(XXH_PRIME2);
    hash ^= hash >> 29;
    hash = hash.wrapping_mul(XXH_PRIME3);
    hash ^= hash >> 32;
    hash
}

/// A conveniently incremental FNV-1a hasher.
#[derive(Clone, Debug)]
pub struct Hasher {
    hash: u64,
}

impl Hasher {
    pub const fn new() -> Self {
        Self {
            hash: 0xcbf2_9ce4_8422_2325,
        }
    }

    pub fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.hash ^= *byte as u64;
            self.hash = self.hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    pub fn write_u64(&mut self, value: u64) {
        self.write(&value.to_le_bytes());
    }

    pub fn finish(&self) -> u64 {
        self.hash
    }
}

impl Default for Hasher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_fnv() {
        assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64(b"hello"), 0xa430_d846_80aa_bd0b);
    }

    #[test]
    fn incremental_matches() {
        let mut hasher = Hasher::new();
        hasher.write(b"hello");
        assert_eq!(hasher.finish(), fnv1a64(b"hello"));
    }

    #[test]
    fn xxhash_deterministic() {
        assert_eq!(xxh64(b"hello", 0), xxh64(b"hello", 0));
        assert_ne!(xxh64(b"hello", 1), xxh64(b"hello", 2));
    }
}
