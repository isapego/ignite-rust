use super::{InStream, Readable};

/// Deserialize i32 from byte array
pub fn deserialize_i32(data: &[u8; 4]) -> i32 {
    (data[0] as i32 & 0xFFi32)
        | ((data[1] as i32 & 0xFFi32) << 8)
        | ((data[2] as i32 & 0xFFi32) << 16)
        | ((data[3] as i32 & 0xFFi32) << 24)
}

/// Deserialize to a value of a certain type
pub fn deserialize_readable<T: Readable<Item = T>>(data: &[u8]) -> T {
    let stream = InStream::new(data);

    T::read(&stream)
}

/// Calculate the value fast which is the power of two and is greater or equals to the provided
/// value. See https://graphics.stanford.edu/~seander/bithacks.html#RoundUpPowerOf2 for details.
pub fn round_to_pow2_u32(val: u32) -> u32 {
    assert_ne!(val, 0, "Value can not be zero");
    assert!(val <= (1 << 31), "Value can not be bigger than 2^31");

    let mut v = val;

    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v + 1
}

#[test]
fn test_deserialize_i32_1() {
    let data = [1u8, 0, 0, 0];

    let res = deserialize_i32(&data);

    assert_eq!(res, 1);
}

#[test]
fn test_deserialize_i32_x11378caa() {
    let data = [0xAAu8, 0x8C, 0x37, 0x11];

    let res = deserialize_i32(&data);

    assert_eq!(res, 0x11378CAA);
}

#[test]
fn test_round_to_pow2_exact() {
    assert_eq!(1, round_to_pow2_u32(1));
    assert_eq!(2, round_to_pow2_u32(2));
    assert_eq!(4, round_to_pow2_u32(4));
    assert_eq!(8, round_to_pow2_u32(8));
    assert_eq!(16, round_to_pow2_u32(16));
    assert_eq!(32, round_to_pow2_u32(32));
    assert_eq!(64, round_to_pow2_u32(64));
    assert_eq!(256, round_to_pow2_u32(256));
    assert_eq!(1024, round_to_pow2_u32(1024));

    for i in 0..32 {
        assert_eq!(1 << i, round_to_pow2_u32(1 << i));
    }
}

#[test]
fn test_round_to_pow2_random() {
    assert_eq!(4, round_to_pow2_u32(3));
    assert_eq!(8, round_to_pow2_u32(5));
    assert_eq!(16, round_to_pow2_u32(12));
    assert_eq!(32, round_to_pow2_u32(30));
    assert_eq!(64, round_to_pow2_u32(53));
    assert_eq!(256, round_to_pow2_u32(211));
    assert_eq!(1024, round_to_pow2_u32(618));

    assert_eq!(1 << 31, round_to_pow2_u32(::std::i32::MAX as u32));

    for i in 2..32 {
        assert_eq!(1 << i, round_to_pow2_u32((1 << i) - 1));
    }
}
