pub struct ByteUtils;

impl ByteUtils {
    pub fn pack_u32(value: u32) -> Vec<u8> {
        vec![
            ((value >> 24) & 0xFF) as u8,
            ((value >> 16) & 0xFF) as u8,
            ((value >> 8) & 0xFF) as u8,
            (value & 0xFF) as u8,
        ]
    }

    pub fn unpack_u32(data: &[u8], offset: usize) -> Option<u32> {
        if offset + 4 > data.len() {
            return None;
        }

        Some(
            (data[offset] as u32) << 24 |
                (data[offset + 1] as u32) << 16 |
                (data[offset + 2] as u32) << 8 |
                (data[offset + 3] as u32)
        )
    }

    pub fn pack_str(s: &str) -> Vec<u8> {
        let bytes = s.as_bytes();
        let mut res = Self::pack_u32(bytes.len() as u32);
        res.extend_from_slice(bytes);

        res
    }

    pub fn unpack_str(data: &[u8], offset: usize) -> Option<(String, usize)> {
        let len = Self::unpack_u32(data, offset)? as usize;
        let str_start = offset + 4;

        if str_start + len > data.len() {
            return None;
        }

        let str_bytes = &data[str_start..str_start + len];
        let string = String::from_utf8(str_bytes.to_vec()).ok()?;
        Some((string, str_start + len))
    }
}