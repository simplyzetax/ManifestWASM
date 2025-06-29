use crate::{
    manifest::shared::{FGuid, FSHAHash},
    ParseResult,
};
use std::ffi::CString;
use widestring::U16String;

/// A struct for writing binary data in the same format as the parser expects
#[derive(Debug)]
pub struct ByteWriter {
    data: Vec<u8>,
}

impl ByteWriter {
    /// Creates a new ByteWriter
    pub fn new() -> ByteWriter {
        ByteWriter { data: Vec::new() }
    }

    /// Writes bytes to the buffer
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    /// Writes any type that implements the ByteWritable trait
    pub fn write<T: ByteWritable>(&mut self, value: &T) {
        value.write(self);
    }

    /// Returns the current position (length of written data)
    pub fn tell(&self) -> usize {
        self.data.len()
    }

    /// Returns the internal data as a Vec<u8>
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    /// Returns a reference to the internal data
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Writes an array with count prefix
    pub fn write_array<T: ByteWritable>(&mut self, items: &[T]) {
        self.write(&(items.len() as u32));
        for item in items {
            self.write(item);
        }
    }
}

/// Trait for types that can be written to a ByteWriter
pub trait ByteWritable {
    fn write(&self, writer: &mut ByteWriter);
}

impl ByteWritable for u64 {
    fn write(&self, writer: &mut ByteWriter) {
        writer.write_bytes(&self.to_le_bytes());
    }
}

impl ByteWritable for u32 {
    fn write(&self, writer: &mut ByteWriter) {
        writer.write_bytes(&self.to_le_bytes());
    }
}

impl ByteWritable for u16 {
    fn write(&self, writer: &mut ByteWriter) {
        writer.write_bytes(&self.to_le_bytes());
    }
}

impl ByteWritable for u8 {
    fn write(&self, writer: &mut ByteWriter) {
        writer.write_bytes(&self.to_le_bytes());
    }
}

impl ByteWritable for i64 {
    fn write(&self, writer: &mut ByteWriter) {
        writer.write_bytes(&self.to_le_bytes());
    }
}

impl ByteWritable for i32 {
    fn write(&self, writer: &mut ByteWriter) {
        writer.write_bytes(&self.to_le_bytes());
    }
}

impl ByteWritable for i16 {
    fn write(&self, writer: &mut ByteWriter) {
        writer.write_bytes(&self.to_le_bytes());
    }
}

impl ByteWritable for i8 {
    fn write(&self, writer: &mut ByteWriter) {
        writer.write_bytes(&self.to_le_bytes());
    }
}

impl ByteWritable for String {
    fn write(&self, writer: &mut ByteWriter) {
        if self.is_empty() {
            writer.write(&0i32);
            return;
        }

        // Write as UTF-8 (positive length)
        let c_string = CString::new(self.as_str()).unwrap();
        let bytes = c_string.into_bytes_with_nul();
        writer.write(&(bytes.len() as i32));
        writer.write_bytes(&bytes);
    }
}

impl ByteWritable for FGuid {
    fn write(&self, writer: &mut ByteWriter) {
        writer.write(&self.a);
        writer.write(&self.b);
        writer.write(&self.c);
        writer.write(&self.d);
    }
}

impl ByteWritable for FSHAHash {
    fn write(&self, writer: &mut ByteWriter) {
        writer.write_bytes(&self.data);
    }
}
