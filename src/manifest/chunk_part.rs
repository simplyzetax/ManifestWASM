use crate::{error::ParseError, reader::ByteReader, ParseResult};

use super::shared::FGuid;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct FChunkPart {
    size: u32,
    guid: FGuid,
    offset: u32,
    file_offset: usize,
}

impl FChunkPart {
    /// This function is used to parse FChunkPart from a ByteReader
    pub fn parse(reader: &mut ByteReader, file_offset: usize) -> ParseResult<FChunkPart> {
        let start = reader.tell();

        let struct_size = reader.read::<u32>()?;
        let guid = reader.read()?;
        let offset = reader.read()?;
        let size = reader.read()?;

        if start + struct_size as usize != reader.tell() {
            println!(
                "ChunkPart size mismatch: expected {} but got {}",
                struct_size,
                reader.tell() - start
            );
            return Err(ParseError::SizeMismatch);
        }

        Ok(FChunkPart {
            size,
            guid,
            offset,
            file_offset,
        })
    }

    /// Writes the FChunkPart to a ByteWriter
    pub fn write(&self, writer: &mut crate::writer::ByteWriter) {
        use crate::writer::ByteWritable;

        // Calculate the size first by writing to a temporary buffer
        let mut temp_writer = crate::writer::ByteWriter::new();
        temp_writer.write(&self.guid);
        temp_writer.write(&self.offset);
        temp_writer.write(&self.size);

        let struct_size = (temp_writer.tell() + 4) as u32; // +4 for the size field itself

        // Write the actual data with correct size
        writer.write(&struct_size);
        writer.write(&self.guid);
        writer.write(&self.offset);
        writer.write(&self.size);
    }

    pub fn file_offset(&self) -> usize {
        self.file_offset
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn guid(&self) -> &FGuid {
        &self.guid
    }

    pub fn offset(&self) -> u32 {
        self.offset
    }
}
