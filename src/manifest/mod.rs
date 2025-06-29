use crate::{reader::ByteReader, ParseResult};

pub mod chunk_info;
pub mod chunk_list;
pub mod chunk_part;
pub mod chunks;
pub mod custom_fields;
pub mod file_manifest;
pub mod file_manifest_list;
pub mod header;
pub mod meta;
pub mod shared;

pub struct FManifestParser {
    pub data: Vec<u8>,
    pub reader: ByteReader,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FManifest {
    pub header: header::FManifestHeader,
    pub meta: meta::FManifestMeta,
    pub chunk_list: chunk_list::FChunkList,
    pub file_list: file_manifest_list::FFileManifestList,
    pub custom_fields: custom_fields::FCustomFields,
    pub data: Vec<u8>,
}

impl FManifestParser {
    pub fn new(data: &[u8]) -> FManifestParser {
        FManifestParser {
            data: data.to_vec(),
            reader: ByteReader::new(data.to_vec()),
        }
    }

    pub fn parse(mut self) -> ParseResult<FManifest> {
        let (header, mut reader) = header::FManifestHeader::parse(&mut self)?;

        let meta = meta::FManifestMeta::parse(&mut reader)?;
        let chunk_header = chunk_list::FChunkList::parse(&mut reader, header.version())?;
        let file_list = file_manifest_list::FFileManifestList::parse(&mut reader)?;
        let custom_fields = custom_fields::FCustomFields::parse(&mut reader)?;

        Ok(FManifest {
            header,
            meta,
            chunk_list: chunk_header,
            file_list,
            custom_fields,
            data: self.data,
        })
    }
}

impl FManifest {
    /// Serializes the FManifest back into a binary manifest file format
    ///
    /// This function recreates the original manifest file structure by writing
    /// each component in the correct order and format.
    pub fn serialize(&self) -> ParseResult<Vec<u8>> {
        use crate::manifest::shared::{EManifestStorageFlags, FSHAHash};
        use crate::writer::{ByteWritable, ByteWriter};
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::io::Write;

        // Create the manifest data (everything except the header)
        let mut data_writer = ByteWriter::new();

        // Write meta with appropriate version (assume version 2 for full compatibility)
        self.meta.write(&mut data_writer, 2);

        // Write chunk list
        self.chunk_list.write(&mut data_writer);

        // Write file list
        self.file_list.write(&mut data_writer);

        // Write custom fields
        self.custom_fields.write(&mut data_writer);

        let uncompressed_data = data_writer.into_bytes();
        let data_size_uncompressed = uncompressed_data.len() as u32;

        // Calculate SHA hash of the uncompressed data before potentially moving it
        let calculated_hash = FSHAHash::new_from_hashable(&uncompressed_data);

        // Compress data if the original was compressed
        let (final_data, data_size_compressed) = match self.header.stored_as() {
            EManifestStorageFlags::Compressed => {
                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                encoder
                    .write_all(&uncompressed_data)
                    .map_err(|_| crate::error::ParseError::InvalidData)?;
                let compressed = encoder
                    .finish()
                    .map_err(|_| crate::error::ParseError::InvalidData)?;
                let compressed_size = compressed.len() as u32;
                (compressed, compressed_size)
            }
            _ => {
                let size = uncompressed_data.len() as u32;
                (uncompressed_data, size)
            }
        };

        // Create updated header with correct sizes and hash
        let updated_header = header::FManifestHeader::new(
            self.header.magic(),
            0, // Will be calculated in write_with_data
            data_size_uncompressed,
            data_size_compressed,
            calculated_hash,
            self.header.stored_as(),
            self.header.version(),
        );

        // Write the complete manifest file
        let mut final_writer = ByteWriter::new();
        updated_header.write_with_data(&mut final_writer, &final_data);

        Ok(final_writer.into_bytes())
    }

    /// Convenience method to write the serialized manifest to a file
    pub fn write_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> ParseResult<()> {
        let data = self.serialize()?;
        std::fs::write(path, data).map_err(|_| crate::error::ParseError::InvalidData)?;
        Ok(())
    }
}
