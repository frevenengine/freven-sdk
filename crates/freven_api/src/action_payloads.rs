//! Versioned codecs for standard gameplay action payloads.
//!
//! Responsibilities:
//! - define stable payload shapes used by default break/place interactions
//! - provide byte codecs that preserve the on-wire format across protocol migrations
//! - keep gameplay payload semantics out of `freven_net_proto`
//!
//! Extension guidance:
//! - keep new payload formats explicitly versioned (e.g. `_v2`)
//! - preserve backwards-compatible decoders where possible

/// Target voxel + hit face for action payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionTarget {
    pub pos: (i32, i32, i32),
    pub face: u8,
}

/// Decoded break payload v1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreakPayloadV1 {
    pub status: u8,
    pub target: ActionTarget,
}

/// Decoded place payload v1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacePayloadV1 {
    pub target: ActionTarget,
    pub block_id: u8,
}

/// Errors from action payload codecs.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum ActionPayloadError {
    #[error("payload ended early while reading {field}")]
    UnexpectedEof { field: &'static str },
    #[error("varint for {field} exceeds u32 encoding bounds")]
    VarintTooLong { field: &'static str },
    #[error("payload contains trailing bytes")]
    TrailingBytes,
}

/// Encode break payload v1.
///
/// Wire format:
/// - status: u8
/// - pos_x: varint zigzag i32
/// - pos_y: varint zigzag i32
/// - pos_z: varint zigzag i32
/// - face: u8
#[must_use]
pub fn encode_break_payload_v1(status: u8, target: ActionTarget) -> Vec<u8> {
    let mut out = Vec::with_capacity(17);
    out.push(status);
    write_var_i32(&mut out, target.pos.0);
    write_var_i32(&mut out, target.pos.1);
    write_var_i32(&mut out, target.pos.2);
    out.push(target.face);
    out
}

/// Decode break payload v1.
pub fn decode_break_payload_v1(payload: &[u8]) -> Result<BreakPayloadV1, ActionPayloadError> {
    let mut cursor = 0usize;
    let status = read_u8(payload, &mut cursor, "status")?;
    let pos_x = read_var_i32(payload, &mut cursor, "pos_x")?;
    let pos_y = read_var_i32(payload, &mut cursor, "pos_y")?;
    let pos_z = read_var_i32(payload, &mut cursor, "pos_z")?;
    let face = read_u8(payload, &mut cursor, "face")?;
    if cursor != payload.len() {
        return Err(ActionPayloadError::TrailingBytes);
    }
    Ok(BreakPayloadV1 {
        status,
        target: ActionTarget {
            pos: (pos_x, pos_y, pos_z),
            face,
        },
    })
}

/// Encode place payload v1.
///
/// Wire format:
/// - hit_pos_x: varint zigzag i32
/// - hit_pos_y: varint zigzag i32
/// - hit_pos_z: varint zigzag i32
/// - face: u8
/// - block_id: u8
#[must_use]
pub fn encode_place_payload_v1(target: ActionTarget, block_id: u8) -> Vec<u8> {
    let mut out = Vec::with_capacity(17);
    write_var_i32(&mut out, target.pos.0);
    write_var_i32(&mut out, target.pos.1);
    write_var_i32(&mut out, target.pos.2);
    out.push(target.face);
    out.push(block_id);
    out
}

/// Decode place payload v1.
pub fn decode_place_payload_v1(payload: &[u8]) -> Result<PlacePayloadV1, ActionPayloadError> {
    let mut cursor = 0usize;
    let hit_x = read_var_i32(payload, &mut cursor, "hit_pos_x")?;
    let hit_y = read_var_i32(payload, &mut cursor, "hit_pos_y")?;
    let hit_z = read_var_i32(payload, &mut cursor, "hit_pos_z")?;
    let face = read_u8(payload, &mut cursor, "face")?;
    let block_id = read_u8(payload, &mut cursor, "block_id")?;
    if cursor != payload.len() {
        return Err(ActionPayloadError::TrailingBytes);
    }
    Ok(PlacePayloadV1 {
        target: ActionTarget {
            pos: (hit_x, hit_y, hit_z),
            face,
        },
        block_id,
    })
}

#[inline]
fn write_var_u32(out: &mut Vec<u8>, mut value: u32) {
    while value >= 0x80 {
        out.push((value as u8 & 0x7F) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

#[inline]
fn write_var_i32(out: &mut Vec<u8>, value: i32) {
    let zigzag = ((value << 1) ^ (value >> 31)) as u32;
    write_var_u32(out, zigzag);
}

fn read_u8(
    payload: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u8, ActionPayloadError> {
    let Some(byte) = payload.get(*cursor).copied() else {
        return Err(ActionPayloadError::UnexpectedEof { field });
    };
    *cursor += 1;
    Ok(byte)
}

fn read_var_u32(
    payload: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, ActionPayloadError> {
    let mut result = 0u32;
    let mut shift = 0u32;
    for i in 0..5 {
        let byte = read_u8(payload, cursor, field)?;
        if i == 4 && byte > 0x0F {
            return Err(ActionPayloadError::VarintTooLong { field });
        }
        result |= u32::from(byte & 0x7F) << shift;
        if (byte & 0x80) == 0 {
            return Ok(result);
        }
        shift += 7;
    }
    Err(ActionPayloadError::VarintTooLong { field })
}

#[inline]
fn read_var_i32(
    payload: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, ActionPayloadError> {
    let raw = read_var_u32(payload, cursor, field)?;
    Ok(((raw >> 1) as i32) ^ (-((raw & 1) as i32)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_break_payload_v1() {
        let payload = encode_break_payload_v1(
            2,
            ActionTarget {
                pos: (10, 64, -5),
                face: 3,
            },
        );
        let decoded = decode_break_payload_v1(&payload).expect("decode break");
        assert_eq!(
            decoded,
            BreakPayloadV1 {
                status: 2,
                target: ActionTarget {
                    pos: (10, 64, -5),
                    face: 3
                }
            }
        );
    }

    #[test]
    fn roundtrip_place_payload_v1() {
        let payload = encode_place_payload_v1(
            ActionTarget {
                pos: (10, 64, -5),
                face: 3,
            },
            1,
        );
        let decoded = decode_place_payload_v1(&payload).expect("decode place");
        assert_eq!(
            decoded,
            PlacePayloadV1 {
                target: ActionTarget {
                    pos: (10, 64, -5),
                    face: 3
                },
                block_id: 1
            }
        );
    }
}
