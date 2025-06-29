use std::io::Read;

use crate::{error::ParseError, reader::ByteReader, ParseResult};

use super::{
    shared::{EFeatureLevel, EManifestStorageFlags, FSHAHash},
    FManifestParser,
};
use flate2::read::ZlibDecoder;

pub const MANIFEST_MAGIC: u32 = 0x44BEC00C;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FManifestHeader {
    magic: u32,
    header_size: u32,
    data_size_uncompressed: u32,
    data_size_compressed: u32,
    sha_hash: FSHAHash,
    stored_as: EManifestStorageFlags,
    version: EFeatureLevel,
}

impl FManifestHeader {
    pub fn parse(manifest: &mut FManifestParser) -> ParseResult<(FManifestHeader, ByteReader)> {
        manifest.reader.seek(0);
        let magic = manifest.reader.read()?;

        if magic != MANIFEST_MAGIC {
            eprintln!(
                "Invalid magic: expected 0x{:08X}, got 0x{:08X}",
                MANIFEST_MAGIC, magic
            );
            return Err(ParseError::InvalidMagic);
        }

        let header_size = manifest.reader.read()?;
        let data_size_uncompressed = manifest.reader.read()?;
        let data_size_compressed = manifest.reader.read()?;
        let header_hash = manifest.reader.read()?;

        eprintln!(
            "Header info: size={}, data_uncompressed={}, data_compressed={}, total_manifest_size={}",
            header_size, data_size_uncompressed, data_size_compressed, manifest.reader.length()
        );

        let stored_as = EManifestStorageFlags::try_from(manifest.reader.read::<u8>()?)
            .map_err(|_| ParseError::InvalidStorageFlag)?;
        let version =
            EFeatureLevel::from_i32(manifest.reader.read()?).ok_or(ParseError::InvalidData)?;

        if header_size != manifest.reader.tell() as u32 {
            eprintln!(
                "Header size mismatch: expected {}, got {}",
                header_size,
                manifest.reader.tell()
            );
            return Err(ParseError::OffsetMismatch);
        }

        let remaining_data_size = manifest.reader.length() - header_size as usize;
        eprintln!(
            "About to read {} bytes of manifest data (expected compressed size: {})",
            remaining_data_size, data_size_compressed
        );

        let data = manifest.reader.read_bytes(remaining_data_size)?; //actual manifest data
        let proper_data = if stored_as == EManifestStorageFlags::Compressed {
            let mut decoder = ZlibDecoder::new(&data[..]);
            let mut buffer: Vec<u8> = Vec::with_capacity(data_size_uncompressed as usize);
            let length = decoder
                .read_to_end(&mut buffer)
                .map_err(|_| ParseError::DecompressionError)?;

            if length != data_size_uncompressed as usize {
                eprintln!(
                    "Decompression size mismatch: expected {}, got {}",
                    data_size_uncompressed, length
                );
                return Err(ParseError::DecompressionError);
            }

            let in_hash = FSHAHash::new_from_hashable(&buffer[..]);

            if in_hash != header_hash {
                eprintln!("Hash mismatch after decompression");
                return Err(ParseError::HashMismatch);
            }

            buffer
        } else {
            data
        };

        let header = FManifestHeader {
            magic,
            header_size,
            data_size_uncompressed,
            data_size_compressed,
            sha_hash: header_hash,
            stored_as,
            version,
        };

        Ok((header, ByteReader::new(proper_data)))
    }

    /// Creates a new FManifestHeader with the specified values
    pub fn new(
        magic: u32,
        header_size: u32,
        data_size_uncompressed: u32,
        data_size_compressed: u32,
        sha_hash: FSHAHash,
        stored_as: EManifestStorageFlags,
        version: EFeatureLevel,
    ) -> Self {
        FManifestHeader {
            magic,
            header_size,
            data_size_uncompressed,
            data_size_compressed,
            sha_hash,
            stored_as,
            version,
        }
    }

    /// Writes the FManifestHeader to a ByteWriter, but header_size should be calculated separately
    pub fn write_with_data(&self, writer: &mut crate::writer::ByteWriter, manifest_data: &[u8]) {
        use crate::writer::ByteWritable;
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::io::Write;

        writer.write(&self.magic);

        // Calculate header size first
        let mut temp_header_writer = crate::writer::ByteWriter::new();
        temp_header_writer.write(&0u32); // magic placeholder
        temp_header_writer.write(&0u32); // header_size placeholder
        temp_header_writer.write(&self.data_size_uncompressed);
        temp_header_writer.write(&self.data_size_compressed);
        temp_header_writer.write(&self.sha_hash);
        temp_header_writer.write(&self.stored_as);
        temp_header_writer.write(&self.version);

        let header_size = temp_header_writer.tell() as u32;

        // Write actual header
        writer.write(&header_size);
        writer.write(&self.data_size_uncompressed);
        writer.write(&self.data_size_compressed);
        writer.write(&self.sha_hash);
        writer.write(&self.stored_as);
        writer.write(&self.version);

        // Write the manifest data (should already be compressed if needed)
        writer.write_bytes(manifest_data);
    }

    pub fn version(&self) -> EFeatureLevel {
        self.version
    }

    pub fn data_size_uncompressed(&self) -> u32 {
        self.data_size_uncompressed
    }

    pub fn data_size_compressed(&self) -> u32 {
        self.data_size_compressed
    }

    pub fn sha_hash(&self) -> &FSHAHash {
        &self.sha_hash
    }

    pub fn stored_as(&self) -> EManifestStorageFlags {
        self.stored_as
    }

    pub fn magic(&self) -> u32 {
        self.magic
    }

    pub fn header_size(&self) -> u32 {
        self.header_size
    }
}
