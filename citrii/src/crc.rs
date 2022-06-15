pub fn crc16_ninty(data: &[u8]) -> u16 {
    let poly = 0x1021;
    let mut crc = 0u32;
    for byte in data {
        crc ^= (*byte as u32) << 8;
        for _ in 0..8 {
            crc <<= 1;
            if crc & 0x10000 != 0 {
                crc = (crc ^ poly) & 0xFFFF;
            }
        }
    }
    (crc & 0xFFFF) as u16
}
