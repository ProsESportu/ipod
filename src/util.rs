pub fn bool_to_byte(value: bool) -> u8 {
    if value {
        0x01
    } else {
        0x00
    }
}

pub fn byte_to_bool(value: u8) -> bool {
    value == 0x01
}

pub fn string_to_bytes(value: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(value.len() + 1);
    bytes.extend_from_slice(value.as_bytes());
    bytes.push(0x00);
    bytes
}

pub(crate) fn bool_from_wire(value: u8) -> bool {
    value != 0
}
