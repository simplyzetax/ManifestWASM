use std::collections::HashMap;

use crate::{error::ParseError, reader::ByteReader, ParseResult};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FCustomFields {
    _size: u32,
    _version: u8,
    pub fields: HashMap<String, String>,
}

impl FCustomFields {
    /// This function is used to parse Custom Fields from a ByteReader
    pub fn parse(reader: &mut ByteReader) -> ParseResult<FCustomFields> {
        let start = reader.tell();

        let size = reader.read()?;
        let version = reader.read()?;
        let count = reader.read()?;

        let mut fields = HashMap::new();
        fields.reserve(count as usize);

        for _ in 0..count {
            let key = reader.read()?;
            let value = reader.read()?;

            fields.insert(key, value);
        }

        if start + size as usize != reader.tell() {
            println!(
                "CustomFields size mismatch: expected {} but got {}",
                size,
                reader.tell() - start
            );
            return Err(ParseError::SizeMismatch);
        }

        Ok(FCustomFields {
            _size: size,
            _version: version,
            fields,
        })
    }

    /// Writes the FCustomFields to a ByteWriter
    pub fn write(&self, writer: &mut crate::writer::ByteWriter) {
        use crate::writer::ByteWritable;

        // Calculate the size first by writing to a temporary buffer
        let mut temp_writer = crate::writer::ByteWriter::new();
        temp_writer.write(&self._version);
        temp_writer.write(&(self.fields.len() as u32));

        for (key, value) in &self.fields {
            temp_writer.write(key);
            temp_writer.write(value);
        }

        let size = (temp_writer.tell() + 4) as u32; // +4 for the size field itself

        // Write the actual data with correct size
        writer.write(&size);
        writer.write(&self._version);
        writer.write(&(self.fields.len() as u32));

        for (key, value) in &self.fields {
            writer.write(key);
            writer.write(value);
        }
    }
}
