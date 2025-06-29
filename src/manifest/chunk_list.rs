use crate::{error::ParseError, manifest::shared::FGuid, reader::ByteReader, ParseResult};

use super::{chunk_info::FChunkInfo, shared::EFeatureLevel};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FChunkList {
    _manifest_version: EFeatureLevel,
    _size: u32,
    _version: u8,
    chunks: Vec<FChunkInfo>,
}

impl FChunkList {
    /// This function is used to parse FChunkInfos from a ByteReader
    pub fn parse(
        reader: &mut ByteReader,
        manifest_version: EFeatureLevel,
    ) -> ParseResult<FChunkList> {
        let reader_start = reader.tell();

        let size = reader.read()?;
        let version = reader.read()?;
        let count: u32 = reader.read()?;

        let mut chunks: Vec<FChunkInfo> = vec![Default::default(); count as usize];

        for chunk in chunks.iter_mut() {
            chunk.guid = reader.read()?;
        }

        for chunk in chunks.iter_mut() {
            chunk.hash = reader.read()?;
        }

        for chunk in chunks.iter_mut() {
            chunk.sha_hash = reader.read()?;
        }

        for chunk in chunks.iter_mut() {
            chunk.group_num = reader.read()?;
        }

        for chunk in chunks.iter_mut() {
            chunk.uncompressed_size = reader.read()?;
        }

        for chunk in chunks.iter_mut() {
            chunk.compressed_size = reader.read()?;
        }

        if reader_start + size as usize != reader.tell() {
            println!(
                "Chunk header size mismatch: expected {} but got {}\nChunkHeader version : {}",
                size,
                reader.tell() - reader_start,
                version
            );
            return Err(ParseError::InvalidData);
        }

        Ok(FChunkList {
            _manifest_version: manifest_version,
            _size: size,
            _version: version,
            chunks,
        })
    }

    /// Writes the FChunkList to a ByteWriter
    pub fn write(&self, writer: &mut crate::writer::ByteWriter) {
        use crate::writer::ByteWritable;

        // Calculate the size first by writing to a temporary buffer
        let mut temp_writer = crate::writer::ByteWriter::new();
        temp_writer.write(&self._version);
        temp_writer.write(&(self.chunks.len() as u32));

        // Write all GUIDs first
        for chunk in &self.chunks {
            temp_writer.write(&chunk.guid);
        }

        // Write all hashes
        for chunk in &self.chunks {
            temp_writer.write(&chunk.hash);
        }

        // Write all SHA hashes
        for chunk in &self.chunks {
            temp_writer.write(&chunk.sha_hash);
        }

        // Write all group numbers
        for chunk in &self.chunks {
            temp_writer.write(&chunk.group_num);
        }

        // Write all uncompressed sizes
        for chunk in &self.chunks {
            temp_writer.write(&chunk.uncompressed_size);
        }

        // Write all compressed sizes
        for chunk in &self.chunks {
            temp_writer.write(&chunk.compressed_size);
        }

        let size = (temp_writer.tell() + 4) as u32; // +4 for the size field itself

        // Write the actual data with correct size
        writer.write(&size);
        writer.write(&self._version);
        writer.write(&(self.chunks.len() as u32));

        // Write all GUIDs first
        for chunk in &self.chunks {
            writer.write(&chunk.guid);
        }

        // Write all hashes
        for chunk in &self.chunks {
            writer.write(&chunk.hash);
        }

        // Write all SHA hashes
        for chunk in &self.chunks {
            writer.write(&chunk.sha_hash);
        }

        // Write all group numbers
        for chunk in &self.chunks {
            writer.write(&chunk.group_num);
        }

        // Write all uncompressed sizes
        for chunk in &self.chunks {
            writer.write(&chunk.uncompressed_size);
        }

        // Write all compressed sizes
        for chunk in &self.chunks {
            writer.write(&chunk.compressed_size);
        }
    }

    pub fn find_by_guid(&self, guid: &FGuid) -> Option<&FChunkInfo> {
        self.chunks.iter().find(|chunk| chunk.guid() == guid)
    }

    pub fn chunks(&self) -> &Vec<FChunkInfo> {
        &self.chunks
    }
}
