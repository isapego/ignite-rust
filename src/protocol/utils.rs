/// Deserialize i32 from byte slice
pub fn deserialize_i32(data: &[u8; 4]) -> i32 {
    (data[0] as i32 & 0xFFi32) |
        ((data[1] as i32 & 0xFFi32) << 8) |
        ((data[2] as i32 & 0xFFi32) << 16) |
        ((data[3] as i32 & 0xFFi32) << 24)
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