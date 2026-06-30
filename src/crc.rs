#[derive(Debug, Default, Clone)]
pub struct Crc8 {
    crc: u8,
}

impl Crc8 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, data: &[u8]) {
        for value in data {
            self.crc = self.crc.wrapping_add(*value);
        }
    }

    pub fn sum8(&self) -> u8 {
        0u8.wrapping_sub(self.crc)
    }

    pub fn reset(&mut self) {
        self.crc = 0x00;
    }
}

pub fn checksum(data: &[u8]) -> u8 {
    let sum = data.iter().fold(0u8, |acc, value| acc.wrapping_add(*value));
    0u8.wrapping_sub(sum)
}

#[cfg(test)]
mod tests {
    use super::checksum;

    #[test]
    fn checksum_vectors_match_go_tests() {
        let tests: &[(&[u8], u8)] = &[
            (&[], 0x00),
            (&[0x00, 0x00], 0x00),
            (&[0xfe, 0x01], 0x01),
            (&[0xff, 0xff, 0x02], 0x00),
            (&[0x04, 0x00, 0x11, 0x00, 0x01], 0xea),
            (
                &[
                    0x0d, 0x00, 0x4c, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x06, 0x3d, 0xef, 0x73,
                    0xff,
                ],
                0xff,
            ),
        ];

        for (payload, want) in tests {
            assert_eq!(checksum(payload), *want);
        }
    }
}
