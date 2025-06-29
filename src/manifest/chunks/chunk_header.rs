use std::io::Read;

use crate::{manifest::shared::{EChunkHashFlags, EChunkStorageFlags, EChunkVersion, FGuid, FSHAHash}, reader::ByteReader, ParseResult};

pub const CHUNK_MAGIC: u32 = 0xB1FE3AA2;

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct FChunkHeader {
    magic: u32,
    version: EChunkVersion,
    header_size: u32,
    data_size_compressed:u32,
    guid: FGuid,
    rolling_hash: u64,
    stored_as: EChunkStorageFlags,
    hash_type: Option<EChunkHashFlags>,
    data_size_uncompressed: Option<u32>,
    sha_hash: Option<FSHAHash>,
}

impl FChunkHeader {
    pub fn parse(reader: &mut ByteReader) -> ParseResult<FChunkHeader> {
        let start = reader.tell();
        let magic = reader.read::<u32>()?;
        if magic != CHUNK_MAGIC {
            return Err(crate::error::ParseError::InvalidMagic)
        }

        let version = EChunkVersion::from_i32(reader.read()?);
        let header_size = reader.read()?;
        let data_size_compressed = reader.read()?;
        let guid: FGuid = reader.read()?;
        let rolling_hash = reader.read()?;
        let stored_as = EChunkStorageFlags::from(reader.read::<u8>()?);

        let mut chunk_header = FChunkHeader {
            magic,
            version,
            header_size,
            data_size_compressed,
            guid,
            rolling_hash,
            stored_as,
            ..Default::default()
        };

        if version.to_i32() >= EChunkVersion::StoresShaAndHashType.to_i32() 
        {
            chunk_header.sha_hash = reader.read::<FSHAHash>().ok();

            if let Some(hash_type) = reader.read::<u8>().ok() {
                chunk_header.hash_type = Some(EChunkHashFlags::from(hash_type));
            }
        }

        if version.to_i32() >= EChunkVersion::StoresDataSizeUncompressed.to_i32() 
        {
            chunk_header.data_size_uncompressed = reader.read::<u32>().ok();
        }

        if reader.tell() - start != chunk_header.header_size as usize {
            println!("{} bytes are missing/were not deserialized.", chunk_header.header_size - (reader.tell() - start) as u32);
            return Err(crate::error::ParseError::SizeMismatch)
        }

        Ok(chunk_header)
    }

    pub fn magic(&self) -> u32 {
        self.magic
    }

    pub fn version(&self) -> EChunkVersion {
        self.version
    }

    pub fn header_size(&self) -> u32 {
        self.header_size
    }

    pub fn data_size_compressed(&self) -> u32 {
        self.data_size_compressed
    }

    pub fn guid(&self) -> FGuid {
        self.guid
    }

    pub fn rolling_hash(&self) -> u64 {
        self.rolling_hash
    }

    pub fn stored_as(&self) -> EChunkStorageFlags {
        self.stored_as
    }

    pub fn hash_type(&self) -> Option<EChunkHashFlags> {
        self.hash_type
    }

    pub fn data_size_uncompressed(&self) -> Option<u32> {
        self.data_size_uncompressed
    }

    pub fn sha_hash(&self) -> Option<FSHAHash> {
        self.sha_hash.clone()
    }

    pub fn is_compressed(&self) -> bool {
        self.stored_as() == (EChunkStorageFlags::Compressed)
    }

    pub fn get_data(&self, reader:&mut ByteReader) -> Vec<u8> {
        match self.stored_as {
            EChunkStorageFlags::Compressed => {
                let compressed_data = reader.read_remaining();
                let mut decoder = flate2::read::ZlibDecoder::new(compressed_data.as_slice());
                let mut buffer:Vec<u8> = Vec::with_capacity(self.data_size_uncompressed().map(|x| x as usize).unwrap_or(0));
                decoder.read_to_end(&mut buffer).unwrap();

                buffer
            },
            EChunkStorageFlags::None => {
                reader.read_remaining()
            },
            _ => {
                panic!("Unsupported storage type: {:?}", self.stored_as);
            }
        }
    }
}