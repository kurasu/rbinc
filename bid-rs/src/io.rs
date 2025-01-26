use std::io::{self, Read, Write};
use uuid::Uuid;

/// Extend `Write` with additional methods for writing primitive types.
pub trait WriteExt: Write {
    fn write_byte(&mut self, byte: u8) -> io::Result<()> {
        self.write_all(&[byte])
    }

    fn write_uint8(&mut self, d: u8) -> io::Result<()> {
        self.write_byte(d)
    }

    fn write_int8(&mut self, d: i8) -> io::Result<()> {
        self.write_byte(d as u8)
    }

    fn write_length(&mut self, value: u64) -> io::Result<()> {
        let mut wrote = false;
        for i in 1..=9 {
            let x = (value >> (7 * (10 - i))) & 0x7f;
            if wrote || x != 0 {
                self.write_uint8((x | 0x80) as u8)?;
                wrote = true;
            }
        }
        self.write_uint8((value & 0x7f) as u8)
    }

    fn write_string(&mut self, d: &str) -> io::Result<()> {
        let bytes = d.as_bytes();
        self.write_length(bytes.len() as u64)?;
        self.write_all(bytes)
    }

    fn write_uint16(&mut self, value: u16) -> io::Result<()> {
        self.write_all(&value.to_le_bytes())
    }

    fn write_int16(&mut self, value: i16) -> io::Result<()> {
        self.write_all(&value.to_le_bytes())
    }

    fn write_uint32(&mut self, value: u32) -> io::Result<()> {
        self.write_all(&value.to_le_bytes())
    }

    fn write_int32(&mut self, value: i32) -> io::Result<()> {
        self.write_all(&value.to_le_bytes())
    }

    fn write_uint64(&mut self, value: u64) -> io::Result<()> {
        self.write_all(&value.to_le_bytes())
    }

    fn write_int64(&mut self, value: i64) -> io::Result<()> {
        self.write_all(&value.to_le_bytes())
    }

    fn write_float(&mut self, value: f32) -> io::Result<()> {
        self.write_all(&value.to_le_bytes())
    }

    fn write_double(&mut self, value: f64) -> io::Result<()> {
        self.write_all(&value.to_le_bytes())
    }

    fn write_bool(&mut self, value: bool) -> io::Result<()> {
        self.write_byte(if value { 1 } else { 0 })
    }

    fn write_uuid(&mut self, value: &Uuid) -> io::Result<()> {
        self.write_all(value.as_bytes())
    }

    fn write_uuid_array(&mut self, value: &Vec<Uuid>) -> io::Result<()> {
        self.write_length(value.len() as u64)?;
        for id in value {
            self.write_uuid(id)?;
        }
        Ok(())
    }

    fn write_string_array(&mut self, value: &Vec<str>) -> io::Result<()> {
        self.write_length(value.len() as u64)?;
        for s in value {
            self.write_string(s)?;
        }
        Ok(())
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

    fn read_uint16(&mut self) -> io::Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    fn read_int16(&mut self) -> io::Result<i16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }

    fn read_uint32(&mut self) -> io::Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_int32(&mut self) -> io::Result<i32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }

    fn read_uint64(&mut self) -> io::Result<u64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    fn read_int64(&mut self) -> io::Result<i64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(i64::from_le_bytes(buf))
    }

    fn read_float(&mut self) -> io::Result<f32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    fn read_double(&mut self) -> io::Result<f64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(f64::from_le_bytes(buf))
    }

    fn read_bool(&mut self) -> io::Result<bool> {
        let byte = self.read_byte()?;
        match byte {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid boolean value")),
        }
    }

    fn read_length(&mut self) -> io::Result<u64> {
        let mut value = 0u64;
        let mut shift = 0;
        loop {
            let byte = self.read_byte()?;
            value |= ((byte & 0x7F) as u64) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift >= 64 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Varint too long",
                ));
            }
        }
        Ok(value)
    }

    fn read_string(&mut self) -> io::Result<String> {
        let length = self.read_length()? as usize;
        let mut buf = vec![0; length];
        self.read_exact(&mut buf)?;
        String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn read_uuid(&mut self) -> io::Result<Uuid> {
        let mut buf = [0; 16];
        self.read_exact(&mut buf)?;
        Uuid::from_slice(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
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
        buffer.write_uint16(12345).unwrap();
        buffer.write_int16(-12345).unwrap();
        buffer.write_uint32(123456789).unwrap();
        buffer.write_int32(-123456789).unwrap();
        buffer.write_uint64(1234567890123456789).unwrap();
        buffer.write_int64(-1234567890123456789).unwrap();
        buffer.write_float(3.14).unwrap();
        buffer.write_double(std::f64::consts::PI).unwrap();
        buffer.write_bool(true).unwrap();
        buffer.write_bool(false).unwrap();

        // Read
        let mut cursor = &buffer[..];
        assert_eq!(cursor.read_uint16().unwrap(), 12345);
        assert_eq!(cursor.read_int16().unwrap(), -12345);
        assert_eq!(cursor.read_uint32().unwrap(), 123456789);
        assert_eq!(cursor.read_int32().unwrap(), -123456789);
        assert_eq!(cursor.read_uint64().unwrap(), 1234567890123456789);
        assert_eq!(cursor.read_int64().unwrap(), -1234567890123456789);
        assert!((cursor.read_float().unwrap() - 3.14).abs() < f32::EPSILON);
        assert!((cursor.read_double().unwrap() - std::f64::consts::PI).abs() < f64::EPSILON);
        assert_eq!(cursor.read_bool().unwrap(), true);
        assert_eq!(cursor.read_bool().unwrap(), false);
    }

    #[test]
    fn test_write_and_read_string() {
        let mut buffer = Vec::new();
        buffer.write_string("hello").unwrap();

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
}
