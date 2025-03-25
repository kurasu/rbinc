use crate::node_id::NodeId;
use blake3::Hash;
use std::io::{self, Error, ErrorKind, Read, Write};
use uuid::Uuid;

/// Extend `Write` with additional methods for writing primitive types.
pub trait WriteExt: Write {
    fn write_byte(&mut self, byte: u8) -> io::Result<()> {
        self.write_all(&[byte])
    }

    fn write_u8(&mut self, d: u8) -> io::Result<()> {
        self.write_byte(d)
    }

    fn write_i8(&mut self, d: i8) -> io::Result<()> {
        self.write_byte(d as u8)
    }

    fn write_u16(&mut self, value: u16) -> io::Result<()> {
        self.write_all(&value.to_be_bytes())
    }

    fn write_i16(&mut self, value: i16) -> io::Result<()> {
        self.write_all(&value.to_be_bytes())
    }

    fn write_u32(&mut self, value: u32) -> io::Result<()> {
        self.write_all(&value.to_be_bytes())
    }

    fn write_i32(&mut self, value: i32) -> io::Result<()> {
        self.write_all(&value.to_be_bytes())
    }

    fn write_u64(&mut self, value: u64) -> io::Result<()> {
        self.write_all(&value.to_be_bytes())
    }

    fn write_i64(&mut self, value: i64) -> io::Result<()> {
        self.write_all(&value.to_be_bytes())
    }

    fn write_f32(&mut self, value: f32) -> io::Result<()> {
        self.write_all(&value.to_be_bytes())
    }

    fn write_f64(&mut self, value: f64) -> io::Result<()> {
        self.write_all(&value.to_be_bytes())
    }

    fn write_bool(&mut self, value: bool) -> io::Result<()> {
        self.write_byte(if value { 1 } else { 0 })
    }

    /// Write a variable length integer value to the stream.
    fn write_length(&mut self, value: usize) -> io::Result<()> {
        if value > usize::MAX {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Value would extend native range",
            ));
        }
        self.write_varint(value as u64)
    }

    /// Write a variable length integer value to the stream. This version xor flips the bytes.
    fn write_length_flipped(&mut self, value: usize) -> io::Result<()> {
        if value > usize::MAX {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Value would extend native range",
            ));
        }
        self.write_varint_flipped(value as u64)
    }

    /// Write a variable length integer value to the stream.
    fn write_length_vlq(&mut self, value: u64) -> io::Result<()> {
        let mut wrote = false;
        for i in 1..=9 {
            let x = (value >> (7 * (10 - i))) & 0x7f;
            if wrote || x != 0 {
                let r = self.write_u8((x | 0x80) as u8);
                if r.is_err() {
                    return r;
                }

                wrote = true;
            }
        }
        self.write_u8((value & 0x7f) as u8)
    }

    /// Write a variable length integer value to the stream. This version xor flips the bytes.
    fn write_length_flipped_vlq(&mut self, value: u64) -> io::Result<()> {
        let mut wrote = false;
        for i in 1..=9 {
            let x = (value >> (7 * (10 - i))) & 0x7f;
            if wrote || x != 0 {
                let r = self.write_u8((x | 0x80) as u8 ^ 0xFF);
                if r.is_err() {
                    return r;
                }

                wrote = true;
            }
        }
        self.write_u8((value & 0x7f) as u8 ^ 0xFF)
    }

    fn write_varint(&mut self, value: u64) -> io::Result<()> {
        const T1: u64 = 204;
        const T2: u64 = 32 * 256 + T1;

        if value <= T1 {
            self.write_u8(value as u8)?
        } else if value < T2 {
            self.write_u8((((value - T1) >> 8) + T1 + 1) as u8)?;
            self.write_u8(((value - T1) & 0xFF) as u8)?
        } else if value < (16 * 65536 + T2) {
            self.write_u8((237 + ((value - T2) >> 16)) as u8)?;
            self.write_u16((value - T2) as u16)?
        } else if value < 16777216 {
            self.write_u8(253)?;
            self.write_u8((value >> 16) as u8)?;
            self.write_u8((value >> 8) as u8)?;
            self.write_u8(value as u8)?
        } else if value <= u32::MAX as u64 {
            self.write_u8(254)?;
            self.write_u32(value as u32)?
        } else {
            self.write_u8(255)?;
            self.write_u64(value)?
        }
        Ok(())
    }

    fn write_varint_flipped(&mut self, value: u64) -> io::Result<()> {
        const T1: u64 = 219;
        const T2: u64 = 32 * 256 + T1;

        if value <= T1 {
            self.write_u8(value as u8 ^ 0xFF)?
        } else if value < T2 {
            self.write_u8((((value - T1) >> 8) + T1 + 1) as u8 ^ 0xFF)?;
            self.write_u8(((value - T1) & 0xFF) as u8 ^ 0xFF)?
        } else if value < (65536 + T2) {
            self.write_u8(252 ^ 0xFF)?;
            self.write_u16((value - T2) as u16 ^ 0xFFFF)?
        } else if value < 16777216 {
            self.write_u8(253 ^ 0xFF)?;
            self.write_u8((value >> 16) as u8 ^ 0xFF)?;
            self.write_u8((value >> 8) as u8 ^ 0xFF)?;
            self.write_u8(value as u8 ^ 0xFF)?
        } else if value <= u32::MAX as u64 {
            self.write_u8(254 ^ 0xFF)?;
            self.write_u32(value as u32 ^ 0xFFFFFFFF)?
        } else {
            self.write_u8(255 ^ 0xFF)?;
            self.write_u64(value ^ 0xFFFFFFFFFFFFFFFF)?
        }
        Ok(())
    }

    fn write_str(&mut self, d: &str) -> io::Result<()> {
        let bytes = d.as_bytes();
        self.write_length(bytes.len())?;
        self.write_all(bytes)
    }

    fn write_string(&mut self, d: &String) -> io::Result<()> {
        let bytes = d.as_bytes();
        self.write_length(bytes.len())?;
        self.write_all(bytes)
    }

    fn write_id(&mut self, id: &NodeId) -> io::Result<()> {
        self.write_length(id.id)
    }

    fn write_uuid(&mut self, value: &Uuid) -> io::Result<()> {
        self.write_all(value.as_bytes())
    }

    fn write_uuid_array(&mut self, value: &Vec<Uuid>) -> io::Result<()> {
        self.write_length(value.len())?;
        for id in value {
            self.write_uuid(id)?;
        }
        Ok(())
    }

    fn write_string_array(&mut self, value: &Vec<String>) -> io::Result<()> {
        self.write_length(value.len())?;
        for s in value {
            self.write_string(s)?;
        }
        Ok(())
    }

    fn write_hash(&mut self, value: &Hash) -> io::Result<()> {
        self.write_all(value.as_bytes())?;
        Ok(())
    }

    fn write_bytes(&mut self, value: &[u8]) -> io::Result<()> {
        self.write_length(value.len())?;
        self.write_all(value)
    }
}

/// Implement `WriteExt` for all types that implement `Write`.
impl<T: Write> WriteExt for T {}

/// Extend `Read` with additional methods for reading primitive types.
pub trait ReadExt: Read {
    fn read_byte(&mut self) -> io::Result<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_i8(&mut self) -> io::Result<i8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(i8::from_be_bytes(buf))
    }

    fn read_u16(&mut self) -> io::Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    fn read_i16(&mut self) -> io::Result<i16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_be_bytes(buf))
    }

    fn read_u32(&mut self) -> io::Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }

    fn read_i32(&mut self) -> io::Result<i32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_be_bytes(buf))
    }

    fn read_u64(&mut self) -> io::Result<u64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_be_bytes(buf))
    }

    fn read_i64(&mut self) -> io::Result<i64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(i64::from_be_bytes(buf))
    }

    fn read_f32(&mut self) -> io::Result<f32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_be_bytes(buf))
    }

    fn read_f64(&mut self) -> io::Result<f64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(f64::from_be_bytes(buf))
    }

    fn read_bool(&mut self) -> io::Result<bool> {
        let byte = self.read_byte()?;
        match byte {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid boolean value",
            )),
        }
    }

    /// Read a variable length integer value from the stream.
    fn read_length(&mut self) -> io::Result<usize> {
        match self.read_varint() {
            Ok(value) => {
                if value > usize::MAX as u64 {
                    Err(Error::new(
                        ErrorKind::InvalidData,
                        "Value would extend 64-bit range",
                    ))
                } else {
                    Ok(value as usize)
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Read a variable length integer value from the stream. This version xor flips the bytes.
    fn read_length_flipped(&mut self) -> io::Result<usize> {
        match self.read_varint_flipped() {
            Ok(value) => {
                if value > usize::MAX as u64 {
                    Err(Error::new(
                        ErrorKind::InvalidData,
                        "Value would extend 64-bit range",
                    ))
                } else {
                    Ok(value as usize)
                }
            }
            Err(e) => Err(e),
        }
    }

    fn read_length_vlq(&mut self) -> io::Result<u64> {
        let mut value: u64 = 0;
        loop {
            let x = self.read_u8()?;
            value = value | u64::from(x & 0x7f);
            if x & 0x80 == 0 {
                return Ok(value);
            }
            if value & 0xFE00000000000000 != 0 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Value would extend 64-bit range",
                ));
            }

            value = value << 7
        }
    }

    /// Read a variable length integer value from the stream. This version xor flips the bytes.
    fn read_length_u64_flipped(&mut self) -> io::Result<u64> {
        let mut value: u64 = 0;
        loop {
            let x = self.read_u8()? ^ 0xFF;
            value = value | u64::from(x & 0x7f);
            if x & 0x80 == 0 {
                return Ok(value);
            }
            if value & 0xFE00000000000000 != 0 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Value would extend 64-bit range",
                ));
            }

            value = value << 7
        }
    }

    fn read_varint(&mut self) -> io::Result<u64> {
        const T1: u8 = 204;
        const T11: u8 = T1 + 1;
        const T2: u64 = 32 * 256 + T1 as u64;

        assert_eq!(8396, T2);

        let a0 = self.read_u8()?;

        match a0 {
            0..=T1 => Ok(a0 as u64),
            205..=236 => {
                let a1 = self.read_u8()?;
                Ok((((a0 as u64 - T11 as u64) << 8) | a1 as u64) + T1 as u64)
            }
            237..253 => {
                let a1 = self.read_u16()?;
                Ok(a1 as u64 + T2 + ((a0 as u64 - 237) << 16))
            }
            253 => {
                let a1 = self.read_u8()?;
                let a2 = self.read_u8()?;
                let a3 = self.read_u8()?;
                Ok(((a1 as u64) << 16) | ((a2 as u64) << 8) | a3 as u64)
            }
            254 => {
                let a1 = self.read_u32()?;
                Ok(a1 as u64)
            }
            255 => {
                let a1 = self.read_u64()?;
                Ok(a1)
            }
            _ => Err(Error::new(ErrorKind::InvalidData, "Invalid value")),
        }
    }

    fn read_varint_flipped(&mut self) -> io::Result<u64> {
        const T1: u8 = 219;
        const T11: u8 = T1 + 1;
        const T2: u64 = 32 * 256 + T1 as u64;

        assert_eq!(8411, T2);

        let a0 = self.read_u8()? ^ 0xFF;

        match a0 {
            0..=T1 => Ok(a0 as u64),
            T11..=251 => {
                let a1 = self.read_u8()? ^ 0xFF;
                Ok((((a0 as u64 - T11 as u64) << 8) | a1 as u64) + T1 as u64)
            }
            252 => {
                let a1 = self.read_u16()? ^ 0xFFFF;
                Ok(a1 as u64 + T2)
            }
            253 => {
                let a1 = self.read_u8()? ^ 0xFF;
                let a2 = self.read_u8()? ^ 0xFF;
                let a3 = self.read_u8()? ^ 0xFF;
                Ok(((a1 as u64) << 16) | ((a2 as u64) << 8) | a3 as u64)
            }
            254 => {
                let a1 = self.read_u32()? ^ 0xFFFFFFFF;
                Ok(a1 as u64)
            }
            255 => {
                let a1 = self.read_u64()? ^ 0xFFFFFFFFFFFFFFFF;
                Ok(a1)
            }
            _ => Err(Error::new(ErrorKind::InvalidData, "Invalid value")),
        }
    }

    fn read_string(&mut self) -> io::Result<String> {
        let length = self.read_length()? as usize;
        let mut buf = vec![0; length];
        self.read_exact(&mut buf)?;
        String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn read_id(&mut self) -> io::Result<NodeId> {
        self.read_length().map(|id| NodeId { id: id as usize })
    }

    fn read_uuid(&mut self) -> io::Result<Uuid> {
        let mut buf = [0; 16];
        self.read_exact(&mut buf)?;
        Uuid::from_slice(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn read_uuid_array(&mut self) -> io::Result<Vec<Uuid>> {
        let length = self.read_length()? as usize;
        let mut uuids = Vec::with_capacity(length);
        for _ in 0..length {
            uuids.push(self.read_uuid()?);
        }
        Ok(uuids)
    }

    fn read_string_array(&mut self) -> io::Result<Vec<String>> {
        let length = self.read_length()? as usize;
        let mut strings = Vec::with_capacity(length);
        for _ in 0..length {
            strings.push(self.read_string()?);
        }
        Ok(strings)
    }

    fn read_hash(&mut self) -> io::Result<Hash> {
        let mut buf = [0; 32];
        self.read_exact(&mut buf)?;
        Ok(Hash::from(buf))
    }

    fn read_bytes(&mut self) -> io::Result<Vec<u8>> {
        let length = self.read_length()? as usize;
        let mut buf = vec![0; length];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }
}

/// Implement `ReadExt` for all types that implement `Read`.
impl<T: Read> ReadExt for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_and_read() {
        let mut buffer = Vec::new();

        // Write
        buffer.write_u16(12345).unwrap();
        buffer.write_i16(-12345).unwrap();
        buffer.write_u32(123456789).unwrap();
        buffer.write_i32(-123456789).unwrap();
        buffer.write_u64(1234567890123456789).unwrap();
        buffer.write_i64(-1234567890123456789).unwrap();
        buffer.write_f32(3.14).unwrap();
        buffer.write_f64(std::f64::consts::PI).unwrap();
        buffer.write_bool(true).unwrap();
        buffer.write_bool(false).unwrap();

        // Read
        let mut cursor = &buffer[..];
        assert_eq!(cursor.read_u16().unwrap(), 12345);
        assert_eq!(cursor.read_i16().unwrap(), -12345);
        assert_eq!(cursor.read_u32().unwrap(), 123456789);
        assert_eq!(cursor.read_i32().unwrap(), -123456789);
        assert_eq!(cursor.read_u64().unwrap(), 1234567890123456789);
        assert_eq!(cursor.read_i64().unwrap(), -1234567890123456789);
        assert!((cursor.read_f32().unwrap() - 3.14).abs() < f32::EPSILON);
        assert!((cursor.read_f64().unwrap() - std::f64::consts::PI).abs() < f64::EPSILON);
        assert_eq!(cursor.read_bool().unwrap(), true);
        assert_eq!(cursor.read_bool().unwrap(), false);
    }

    #[test]
    fn test_write_and_read_string() {
        let mut buffer = Vec::new();
        buffer.write_str("hello").unwrap();

        // Read
        let mut cursor = &buffer[..];
        assert_eq!(cursor.read_string().unwrap(), "hello");
    }

    #[test]
    fn test_write_and_read_uuid() {
        let mut buffer = Vec::new();
        let id = Uuid::new_v4();
        buffer.write_uuid(&id).unwrap();

        // Read
        let mut cursor = &buffer[..];
        assert_eq!(cursor.read_uuid().unwrap(), id);
    }

    #[test]
    fn test_length() {
        let values = [
            0,
            1,
            9,
            127,
            128,
            129,
            254,
            255,
            256,
            267,
            333,
            513,
            1000,
            10000,
            100000,
            1000000000,
            1000000000000000,
        ];
        for value in values {
            let mut w: Vec<u8> = Vec::new();
            w.write_length(value).unwrap();

            let mut r = w.as_slice();
            assert_eq!(value, r.read_length().unwrap());
        }
    }

    fn test_length_128() {
        let bytes = [0x81u8, 0];
        assert_eq!(128, bytes.as_slice().read_length().unwrap());
    }

    fn test_length_flipped_128() {
        let bytes = [0x7eu8, 0xffu8];
        assert_eq!(128, bytes.as_slice().read_length_flipped().unwrap());
    }

    #[test]
    fn test_length_flipped() {
        let values = [
            0,
            1,
            9,
            127,
            128,
            129,
            254,
            255,
            256,
            267,
            333,
            513,
            1000,
            10000,
            100000,
            1000000000,
            1000000000000000,
        ];
        for value in values {
            let mut w: Vec<u8> = Vec::new();
            w.write_length_flipped(value).unwrap();

            let mut r = w.as_slice();
            assert_eq!(value, r.read_length_flipped().unwrap());
        }
    }

    #[test]
    fn test_string() {
        let values = ["hej", "crazy fox", "", "goodbye"];
        for value in values {
            let mut w: Vec<u8> = Vec::new();
            w.write_str(value).unwrap();

            let mut r = w.as_slice();
            assert_eq!(value, r.read_string().unwrap().as_str());
        }
    }

    #[test]
    fn test_u8() {
        let values: [u8; 11] = [0, 1, 2, 5, 88, 109, 127, 128, 129, 223, 255];
        let mut w: Vec<u8> = Vec::new();
        for value in values {
            w.write_u8(value).unwrap()
        }

        let mut r = w.as_slice();
        for value in values {
            assert_eq!(value, r.read_u8().unwrap());
        }
    }

    #[test]
    fn test_i8() {
        let values: [i8; 11] = [0, 1, 2, 5, 88, 109, 127, -128, -127, -1, -23];
        let mut w: Vec<u8> = Vec::new();
        for value in values {
            w.write_i8(value).unwrap()
        }

        let mut r = w.as_slice();
        for value in values {
            assert_eq!(value, r.read_i8().unwrap());
        }
    }

    #[test]
    fn test_u16() {
        let values: [u16; 15] = [
            0, 1, 2, 5, 88, 109, 127, 128, 129, 223, 255, 256, 3434, 32767, 65535,
        ];
        let mut w: Vec<u8> = Vec::new();
        for value in values {
            w.write_u16(value).unwrap()
        }

        let mut r = w.as_slice();
        for value in values {
            assert_eq!(value, r.read_u16().unwrap());
        }
    }

    #[test]
    fn test_i16() {
        let values: [i16; 15] = [
            0, 1, 2, 5, 88, 109, 127, -128, -127, -1, -23, 32767, -1000, 2000, -32768,
        ];
        let mut w: Vec<u8> = Vec::new();
        for value in values {
            w.write_i16(value).unwrap()
        }

        let mut r = w.as_slice();
        for value in values {
            assert_eq!(value, r.read_i16().unwrap());
        }
    }

    #[test]
    fn test_varint() {
        let values = [
            0,
            1,
            9,
            127,
            128,
            129,
            218,
            219,
            220,
            254,
            255,
            256,
            267,
            333,
            513,
            1000,
            8410,
            8411,
            8412,
            10000,
            65536 + 8410,
            65536 + 8411,
            65536 + 8412,
            100000,
            12323003,
            16777215,
            16777216,
            1000000000,
            1000000000000000,
        ];

        let mut w: Vec<u8> = Vec::new();
        for value in values {
            w.clear();
            w.write_varint(value).unwrap();

            let mut r = w.as_slice();
            assert_eq!(value, r.read_varint().expect("Could not read varint"));
        }

        for value in 0..200000 as u64 {
            w.clear();
            w.write_varint(value).unwrap();

            let mut r = w.as_slice();
            assert_eq!(value, r.read_varint().expect("Could not read varint"));
        }
    }

    #[test]
    fn test_varint_size() {
        let values = [
            (0u64, 1),
            (204, 1),
            (205, 2),
            (8395, 2),
            (8396, 3),
            (1056971, 3),
            (1056972, 4),
            (16777215, 4),
            (16777216, 5),
            (u32::MAX as u64, 5),
            ((u32::MAX as u64) + 1, 9),
            (u64::MAX, 9),
        ];

        for (value, size) in values {
            let mut w: Vec<u8> = Vec::new();
            w.write_varint(value).unwrap();

            let mut r = w.as_slice();
            assert_eq!(size, r.len(), "Size of varint {} is not {}", value, size);
        }
    }
}
