// Define a struct to represent a byte reader
// It will be used to parse the actual binary into a proper Manifest.

use std::ffi::CString;

use widestring::U16String;

use crate::{
    error::ParseError,
    manifest::shared::{FGuid, FSHAHash, SHA1_DIGEST_SIZE},
    ParseResult,
};

#[derive(Debug)]
pub struct ByteReader {
    data: Vec<u8>,
    position: usize,
}

impl ByteReader {
    /// Creates a new ByteReader from a Vec<u8>
    ///
    /// # Arguments
    ///
    /// * `data` - A Vec<u8> containing the binary data
    ///
    pub fn new(data: Vec<u8>) -> ByteReader {
        ByteReader { data, position: 0 }
    }

    /// This function is used to read a certain amount of bytes from the binary data and return it as a Vec<u8>
    pub fn read_bytes(&mut self, size: usize) -> ParseResult<Vec<u8>> {
        if self.position + size > self.data.len() {
            eprintln!(
                "ByteReader overflow: trying to read {} bytes at position {}, but data length is {}",
                size, self.position, self.data.len()
            );
            return Err(ParseError::Overflow);
        }

        let mut result = Vec::with_capacity(size);
        for i in 0..size {
            result.push(self.data[self.position + i]);
        }
        self.position += size;

        Ok(result)
    }

    /// This function is used to read any type that implements the ByteReadable trait
    pub fn read<T: ByteReadable>(&mut self) -> ParseResult<T> {
        T::read(self)
    }

    /// This function is used to get the current position of the reader
    pub fn tell(&self) -> usize {
        self.position
    }

    /// This function is used to get the length of the binary data
    pub fn length(&self) -> usize {
        self.data.len()
    }

    pub fn seek(&mut self, position: usize) {
        self.position = position;
    }

    /// This function is used to read an array. It takes a closure that will be used to read each item of the array
    /// # Exemples (from src/manifest/meta.rs)
    /// ```
    /// fn get_prereq_ids(reader: &mut ByteReader) -> ParseResult<Vec<String>> {
    ///     reader.read_array(|reader| reader.read())
    /// }
    /// ```

    pub fn read_array<T>(
        &mut self,
        mut read_item: impl FnMut(&mut Self) -> ParseResult<T>,
    ) -> ParseResult<Vec<T>> {
        let count = self.read::<u32>()? as usize;

        if count == 0 {
            return Ok(vec![]);
        } else {
            let mut result = Vec::with_capacity(count);
            for _ in 0..count {
                result.push(read_item(self)?);
            }
            Ok(result)
        }
    }

    pub fn read_remaining(&mut self) -> Vec<u8> {
        let result = self.data[self.position..].to_vec();
        self.position = self.data.len();
        result
    }
}

pub trait ByteReadable: Sized {
    fn read(reader: &mut ByteReader) -> ParseResult<Self>;
}

impl ByteReadable for u64 {
    fn read(reader: &mut ByteReader) -> ParseResult<Self> {
        let result = u64::from_le_bytes(
            reader
                .read_bytes(8)?
                .try_into()
                .map_err(|_| ParseError::InvalidData)?,
        );
        Ok(result)
    }
}

impl ByteReadable for u32 {
    fn read(reader: &mut ByteReader) -> ParseResult<Self> {
        let result = u32::from_le_bytes(
            reader
                .read_bytes(4)?
                .try_into()
                .map_err(|_| ParseError::InvalidData)?,
        );
        Ok(result)
    }
}

impl ByteReadable for u16 {
    fn read(reader: &mut ByteReader) -> ParseResult<Self> {
        let result = u16::from_le_bytes(
            reader
                .read_bytes(2)?
                .try_into()
                .map_err(|_| ParseError::InvalidData)?,
        );
        Ok(result)
    }
}

impl ByteReadable for u8 {
    fn read(reader: &mut ByteReader) -> ParseResult<Self> {
        let result = u8::from_le_bytes(
            reader
                .read_bytes(1)?
                .try_into()
                .map_err(|_| ParseError::InvalidData)?,
        );
        Ok(result)
    }
}

impl ByteReadable for i64 {
    fn read(reader: &mut ByteReader) -> ParseResult<Self> {
        let result = i64::from_le_bytes(
            reader
                .read_bytes(8)?
                .try_into()
                .map_err(|_| ParseError::InvalidData)?,
        );
        Ok(result)
    }
}

impl ByteReadable for i32 {
    fn read(reader: &mut ByteReader) -> ParseResult<Self> {
        let result = i32::from_le_bytes(
            reader
                .read_bytes(4)?
                .try_into()
                .map_err(|_| ParseError::InvalidData)?,
        );
        Ok(result)
    }
}

impl ByteReadable for i16 {
    fn read(reader: &mut ByteReader) -> ParseResult<Self> {
        let result = i16::from_le_bytes(
            reader
                .read_bytes(2)?
                .try_into()
                .map_err(|_| ParseError::InvalidData)?,
        );
        Ok(result)
    }
}

impl ByteReadable for i8 {
    fn read(reader: &mut ByteReader) -> ParseResult<Self> {
        let result = i8::from_le_bytes(
            reader
                .read_bytes(1)?
                .try_into()
                .map_err(|_| ParseError::InvalidData)?,
        );
        Ok(result)
    }
}

impl ByteReadable for String {
    fn read(reader: &mut ByteReader) -> ParseResult<Self> {
        let length = reader.read::<i32>()?;

        if length == 0 {
            return Ok(String::new());
        }

        let utf_8 = length > 0;

        let string = if utf_8 {
            let c_string = CString::from_vec_with_nul(reader.read_bytes(length as usize)?)
                .map_err(|_| ParseError::InvalidData)?;

            c_string
                .into_string()
                .map_err(|_| ParseError::InvalidData)?
        } else {
            let length = (length * -2) as usize;
            let byte_data = reader.read_bytes(length)?;

            //shouldn't panic
            unsafe {
                let u16_string =
                    U16String::from_ptr(byte_data.as_ptr() as *const u16, length.abs_diff(0));
                u16_string.to_string_lossy()
            }
        };

        Ok(string)
    }
}

impl ByteReadable for FGuid {
    fn read(reader: &mut ByteReader) -> ParseResult<Self> {
        let a = reader.read()?;
        let b = reader.read()?;
        let c = reader.read()?;
        let d = reader.read()?;

        Ok(FGuid { a, b, c, d })
    }
}

impl ByteReadable for FSHAHash {
    fn read(reader: &mut ByteReader) -> ParseResult<Self> {
        Ok(FSHAHash {
            data: reader
                .read_bytes(SHA1_DIGEST_SIZE)?
                .try_into()
                .map_err(|_| crate::error::ParseError::InvalidData)?,
        })
    }
}
