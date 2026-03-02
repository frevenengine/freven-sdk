/// Canonical button bits for `HumanoidInputV1::buttons`.
pub mod button_bits {
    pub const JUMP: u16 = 1;
    pub const SPRINT: u16 = 2;
    pub const CROUCH: u16 = 4;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HumanoidInputV1 {
    pub move_x: i8,
    pub move_z: i8,
    pub buttons: u16,
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum HumanoidInputError {
    #[error("invalid humanoid input payload length: expected 4 bytes, got {got}")]
    InvalidLength { got: usize },
}

#[must_use]
pub fn encode_humanoid_input_v1(input: HumanoidInputV1) -> [u8; 4] {
    let buttons = input.buttons.to_le_bytes();
    [
        input.move_x as u8,
        input.move_z as u8,
        buttons[0],
        buttons[1],
    ]
}

pub fn decode_humanoid_input_v1(payload: &[u8]) -> Result<HumanoidInputV1, HumanoidInputError> {
    if payload.len() != 4 {
        return Err(HumanoidInputError::InvalidLength { got: payload.len() });
    }
    Ok(HumanoidInputV1 {
        move_x: payload[0] as i8,
        move_z: payload[1] as i8,
        buttons: u16::from_le_bytes([payload[2], payload[3]]),
    })
}

#[inline]
#[must_use]
pub fn quantize_deg_x100_i16(deg: f32) -> i16 {
    let q = (deg * 100.0).round();
    q.clamp(i16::MIN as f32, i16::MAX as f32) as i16
}

#[inline]
#[must_use]
pub fn dequantize_deg_x100_i16(value: i16) -> f32 {
    value as f32 / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn humanoid_input_roundtrip() {
        let src = HumanoidInputV1 {
            move_x: -127,
            move_z: 64,
            buttons: button_bits::JUMP | button_bits::SPRINT,
        };
        let encoded = encode_humanoid_input_v1(src);
        let decoded = decode_humanoid_input_v1(&encoded).expect("decode");
        assert_eq!(decoded, src);
    }
}
