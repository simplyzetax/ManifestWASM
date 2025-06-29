use std::{
    collections::hash_map::DefaultHasher,
    fmt::Formatter,
    hash::{Hash, Hasher},
};

use sha1::{Digest, Sha1};

use crate::{helper, reader::ByteReader, ParseResult};

pub const SHA1_DIGEST_SIZE: usize = 20;
pub const MD5_DIGEST_SIZE: usize = 16;
pub const SHA256_DIGEST_SIZE: usize = 32;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]

/// This type is the same type used in the Unreal Engine 4 source code to represent a GUID.
/// Learn more here https://docs.unrealengine.com/4.27/en-US/API/Runtime/Core/Misc/FGuid

pub struct FGuid {
    ///Private:
    pub a: u32,
    ///	Holds the second component.
    pub b: u32,
    ///	Holds the third component.
    pub c: u32,
    ///    Holds the fourth component.
    pub d: u32,
}

impl std::fmt::Debug for FGuid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl ToString for FGuid {
    fn to_string(&self) -> String {
        let mut normalize = |str: String| -> String {
            let len = str.len();
            if len == 8 {
                str
            } else {
                "0".repeat(8 - len) + &str
            }
        };

        normalize(helper::to_hex(self.a))
            + &normalize(helper::to_hex(self.b))
            + &normalize(helper::to_hex(self.c))
            + &normalize(helper::to_hex(self.d))
    }
}

#[derive(Debug, PartialEq, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum EManifestStorageFlags {
    // Stored as raw data.
    None = 0,
    // Flag for compressed data.
    Compressed = 1,
    // Flag for encrypted. If also compressed, decrypt first. Encryption will ruin compressibility.
    Encrypted = 1 << 1,
}

impl From<u8> for EManifestStorageFlags {
    fn from(value: u8) -> Self {
        match value {
            0 => EManifestStorageFlags::None,
            1 => EManifestStorageFlags::Compressed,
            2 => EManifestStorageFlags::Encrypted,
            _ => panic!("Invalid EManifestStorageFlags value"),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone, serde::Serialize)]
pub enum EChunkStorageFlags {
    None,
    // Flag for compressed data.
    Compressed,
    // Flag for encrypted. If also compressed, decrypt first. Encryption will ruin compressibility.
    Encrypted,
}

impl Default for EChunkStorageFlags {
    fn default() -> Self {
        EChunkStorageFlags::None
    }
}

impl From<u8> for EChunkStorageFlags {
    fn from(value: u8) -> Self {
        match value {
            0 => EChunkStorageFlags::None,
            1 => EChunkStorageFlags::Compressed,
            2 => EChunkStorageFlags::Encrypted,
            _ => panic!("Invalid EChunkStorageFlags value"),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone, serde::Serialize)]
pub enum EChunkHashFlags {
    None,

    // Flag for FRollingHash class used, stored in RollingHash on header.
    RollingPoly64,

    // Flag for FSHA1 class used, stored in SHAHash on header.
    Sha1,

    Both,
}

impl From<u8> for EChunkHashFlags {
    fn from(value: u8) -> Self {
        match value {
            0 => EChunkHashFlags::None,
            1 => EChunkHashFlags::RollingPoly64,
            2 => EChunkHashFlags::Sha1,
            3 => EChunkHashFlags::Both,
            _ => panic!("Invalid EChunkHashFlags value"),
        }
    }
}

impl Default for EChunkHashFlags {
    fn default() -> Self {
        EChunkHashFlags::None
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum EChunkVersion {
    Invalid,
    Original,
    StoresShaAndHashType,
    StoresDataSizeUncompressed,

    // Always after the latest version, signifies the latest version plus 1 to allow initialization simplicity.
    LatestPlusOne,
    Latest,
}

impl Default for EChunkVersion {
    fn default() -> Self {
        EChunkVersion::Invalid
    }
}

impl EChunkVersion {
    pub fn to_i32(&self) -> i32 {
        match self {
            EChunkVersion::Invalid => 0,
            EChunkVersion::Original => 1,
            EChunkVersion::StoresShaAndHashType => 2,
            EChunkVersion::StoresDataSizeUncompressed => 3,
            EChunkVersion::LatestPlusOne => 4,
            EChunkVersion::Latest => EChunkVersion::LatestPlusOne.to_i32() - 1,
        }
    }

    pub fn from_i32(value: i32) -> EChunkVersion {
        match value {
            0 => EChunkVersion::Invalid,
            1 => EChunkVersion::Original,
            2 => EChunkVersion::StoresShaAndHashType,
            3 => EChunkVersion::StoresDataSizeUncompressed,
            4 => EChunkVersion::LatestPlusOne,
            5 => EChunkVersion::Latest,
            _ => EChunkVersion::Invalid,
        }
    }
}

impl PartialEq for EChunkVersion {
    fn eq(&self, other: &Self) -> bool {
        self.to_i32() == other.to_i32()
    }
}

/**
 * An enum type to describe supported features of a certain manifest.
 */
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum EFeatureLevel {
    // The original version.
    Original,
    // Support for custom fields.
    CustomFields,
    // Started storing the version number.
    StartStoringVersion,
    // Made after data files where renamed to include the hash value, these chunks now go to ChunksV2.
    DataFileRenames,
    // Manifest stores whether build was constructed with chunk or file data.
    StoresIfChunkOrFileData,
    // Manifest stores group number for each chunk/file data for reference so that external readers don't need to know how to calculate them.
    StoresDataGroupNumbers,
    // Added support for chunk compression, these chunks now go to ChunksV3. NB: Not File Data Compression yet.
    ChunkCompressionSupport,
    // Manifest stores product prerequisites info.
    StoresPrerequisitesInfo,
    // Manifest stores chunk download sizes.
    StoresChunkFileSizes,
    // Manifest can optionally be stored using UObject serialization and compressed.
    StoredAsCompressedUClass,
    // These two features were removed and never used.
    Unused0,
    Unused1,
    // Manifest stores chunk data SHA1 hash to use in place of data compare, for faster generation.
    StoresChunkDataShaHashes,
    // Manifest stores Prerequisite Ids.
    StoresPrerequisiteIds,
    // The first minimal binary format was added. UObject classes will no longer be saved out when binary selected.
    StoredAsBinaryData,
    // Temporary level where manifest can reference chunks with dynamic window size, but did not serialize them. Chunks from here onwards are stored in ChunksV4.
    VariableSizeChunksWithoutWindowSizeChunkInfo,
    // Manifest can reference chunks with dynamic window size, and also serializes them.
    VariableSizeChunks,
    // Manifest uses a build id generated from its metadata.
    UsesRuntimeGeneratedBuildId,
    // Manifest uses a build id generated unique at build time, and stored in manifest.
    UsesBuildTimeGeneratedBuildId,

    // !! Always after the latest version entry, signifies the latest version plus 1 to allow the following Latest alias.
    LatestPlusOne,
    // An alias for the actual latest version value.
    Latest,
    // An alias to provide the latest version of a manifest supported by file data (nochunks).
    LatestNoChunks,
    // An alias to provide the latest version of a manifest supported by a json serialized format.
    LatestJson,
    // An alias to provide the first available version of optimised delta manifest saving.
    FirstOptimisedDelta,

    // More aliases, but this time for values that have been renamed
    StoresUniqueBuildId,

    // JSON manifests were stored with a version of 255 during a certain CL range due to a bug.
    // We will treat this as being StoresChunkFileSizes in code.
    BrokenJsonVersion,
    // This is for UObject default, so that we always serialize it.
    Invalid,
}

impl PartialEq for EFeatureLevel {
    fn eq(&self, other: &Self) -> bool {
        self.to_i32() == other.to_i32()
    }
}

//impl FeatureLevel => int32
impl EFeatureLevel {
    pub fn to_i32(&self) -> i32 {
        match self {
            EFeatureLevel::Original => 0,
            EFeatureLevel::CustomFields => 1,
            EFeatureLevel::StartStoringVersion => 2,
            EFeatureLevel::DataFileRenames => 3,
            EFeatureLevel::StoresIfChunkOrFileData => 4,
            EFeatureLevel::StoresDataGroupNumbers => 5,
            EFeatureLevel::ChunkCompressionSupport => 6,
            EFeatureLevel::StoresPrerequisitesInfo => 7,
            EFeatureLevel::StoresChunkFileSizes => 8,
            EFeatureLevel::StoredAsCompressedUClass => 9,
            EFeatureLevel::Unused0 => 10,
            EFeatureLevel::Unused1 => 11,
            EFeatureLevel::StoresChunkDataShaHashes => 12,
            EFeatureLevel::StoresPrerequisiteIds => 13,
            EFeatureLevel::StoredAsBinaryData => 14,
            EFeatureLevel::VariableSizeChunksWithoutWindowSizeChunkInfo => 15,
            EFeatureLevel::VariableSizeChunks => 16,
            EFeatureLevel::UsesRuntimeGeneratedBuildId => 17,
            EFeatureLevel::UsesBuildTimeGeneratedBuildId => 18,
            EFeatureLevel::LatestPlusOne => 19,
            EFeatureLevel::Latest => EFeatureLevel::LatestPlusOne.to_i32() - 1,
            EFeatureLevel::LatestNoChunks => EFeatureLevel::StoresChunkFileSizes.to_i32(),
            EFeatureLevel::LatestJson => EFeatureLevel::StoresPrerequisiteIds.to_i32(),
            EFeatureLevel::FirstOptimisedDelta => {
                EFeatureLevel::UsesRuntimeGeneratedBuildId.to_i32()
            }
            EFeatureLevel::StoresUniqueBuildId => {
                EFeatureLevel::UsesRuntimeGeneratedBuildId.to_i32()
            }
            EFeatureLevel::BrokenJsonVersion => 255,
            EFeatureLevel::Invalid => -1,
        }
    }

    pub fn from_i32(value: i32) -> Option<EFeatureLevel> {
        match value {
            0 => Some(EFeatureLevel::Original),
            1 => Some(EFeatureLevel::CustomFields),
            2 => Some(EFeatureLevel::StartStoringVersion),
            3 => Some(EFeatureLevel::DataFileRenames),
            4 => Some(EFeatureLevel::StoresIfChunkOrFileData),
            5 => Some(EFeatureLevel::StoresDataGroupNumbers),
            6 => Some(EFeatureLevel::ChunkCompressionSupport),
            7 => Some(EFeatureLevel::StoresPrerequisitesInfo),
            8 => Some(EFeatureLevel::StoresChunkFileSizes),
            9 => Some(EFeatureLevel::StoredAsCompressedUClass),
            10 => Some(EFeatureLevel::Unused0),
            11 => Some(EFeatureLevel::Unused1),
            12 => Some(EFeatureLevel::StoresChunkDataShaHashes),
            13 => Some(EFeatureLevel::StoresPrerequisiteIds),
            14 => Some(EFeatureLevel::StoredAsBinaryData),
            15 => Some(EFeatureLevel::VariableSizeChunksWithoutWindowSizeChunkInfo),
            16 => Some(EFeatureLevel::VariableSizeChunks),
            17 => Some(EFeatureLevel::UsesRuntimeGeneratedBuildId),
            18 => Some(EFeatureLevel::UsesBuildTimeGeneratedBuildId),
            19 => Some(EFeatureLevel::LatestPlusOne),
            20 => Some(EFeatureLevel::Latest),
            21 => Some(EFeatureLevel::LatestNoChunks),
            22 => Some(EFeatureLevel::LatestJson),
            23 => Some(EFeatureLevel::FirstOptimisedDelta),
            24 => Some(EFeatureLevel::StoresUniqueBuildId),
            255 => Some(EFeatureLevel::BrokenJsonVersion),
            -1 => Some(EFeatureLevel::Invalid),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]

/// This type is used to represent a hash. The length of the hash is known at compile time.
/// It is used to represent both MD5 and SHA256 hashes.
pub struct UnknownHash<const DIGEST_LENGTH: usize> {
    pub data: [u8; DIGEST_LENGTH],
}

impl<const DIGEST_LENGTH: usize> serde::Serialize for UnknownHash<DIGEST_LENGTH> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, const DIGEST_LENGTH: usize> serde::Deserialize<'de> for UnknownHash<DIGEST_LENGTH> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.len() != DIGEST_LENGTH * 2 {
            return Err(serde::de::Error::custom(format!(
                "Expected hex string of length {}, got {}",
                DIGEST_LENGTH * 2,
                s.len()
            )));
        }

        let mut data = [0u8; DIGEST_LENGTH];
        for i in 0..DIGEST_LENGTH {
            let hex_byte = &s[i * 2..i * 2 + 2];
            data[i] = u8::from_str_radix(hex_byte, 16)
                .map_err(|e| serde::de::Error::custom(format!("Invalid hex: {}", e)))?;
        }
        Ok(UnknownHash { data })
    }
}

impl<const DIGEST_LENGTH: usize> UnknownHash<DIGEST_LENGTH> {
    pub fn new(data: [u8; DIGEST_LENGTH]) -> UnknownHash<DIGEST_LENGTH> {
        UnknownHash { data }
    }

    pub fn from_byte_reader(reader: &mut ByteReader) -> ParseResult<UnknownHash<DIGEST_LENGTH>> {
        Ok(UnknownHash {
            data: reader
                .read_bytes(DIGEST_LENGTH)?
                .try_into()
                .map_err(|_| crate::error::ParseError::InvalidData)?,
        })
    }

    pub fn data(&self) -> [u8; DIGEST_LENGTH] {
        self.data
    }
}

impl<const DIGEST_LENGTH: usize> ToString for UnknownHash<DIGEST_LENGTH> {
    fn to_string(&self) -> String {
        let mut result = String::with_capacity(DIGEST_LENGTH * 2);
        for byte in self.data.iter() {
            result.push_str(&format!("{:02x}", byte));
        }
        result
    }
}

#[derive(Clone)]

/// This type is the same type used in the Unreal Engine 4 source code to represent a SHA1 hash
/// Learn more here https://docs.unrealengine.com/4.27/en-US/API/Runtime/Core/Misc/FSHAHash/
pub struct FSHAHash {
    pub(crate) data: [u8; SHA1_DIGEST_SIZE],
}

impl std::fmt::Debug for FSHAHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl serde::Serialize for FSHAHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for FSHAHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.len() != SHA1_DIGEST_SIZE * 2 {
            return Err(serde::de::Error::custom(format!(
                "Expected hex string of length {}, got {}",
                SHA1_DIGEST_SIZE * 2,
                s.len()
            )));
        }

        let mut data = [0u8; SHA1_DIGEST_SIZE];
        for i in 0..SHA1_DIGEST_SIZE {
            let hex_byte = &s[i * 2..i * 2 + 2];
            data[i] = u8::from_str_radix(hex_byte, 16)
                .map_err(|e| serde::de::Error::custom(format!("Invalid hex: {}", e)))?;
        }
        Ok(FSHAHash { data })
    }
}

impl Default for FSHAHash {
    fn default() -> Self {
        FSHAHash {
            data: [0; SHA1_DIGEST_SIZE],
        }
    }
}

impl PartialEq for FSHAHash {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl FSHAHash {
    pub fn new(data: [u8; SHA1_DIGEST_SIZE]) -> FSHAHash {
        FSHAHash { data }
    }

    fn to_string(&self) -> String {
        let mut result = String::with_capacity(SHA1_DIGEST_SIZE * 2);
        for byte in self.data.iter() {
            result.push_str(&format!("{:02x}", byte));
        }
        result
    }

    pub fn new_from_hashable(data: impl Hash + std::convert::AsRef<[u8]>) -> FSHAHash {
        let mut hasher = Sha1::new();
        hasher.update(data);

        FSHAHash {
            data: hasher.finalize().into(),
        }
    }

    pub fn data(&self) -> [u8; SHA1_DIGEST_SIZE] {
        self.data
    }

    pub fn to_hex_string(&self) -> String {
        let mut result = String::with_capacity(SHA1_DIGEST_SIZE * 2);
        for byte in self.data.iter() {
            result.push_str(&format!("{:02x}", byte));
        }
        result
    }

    pub fn to_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        self.data.hash(&mut hasher);

        hasher.finish()
    }
}

// ByteWritable implementations for shared types
use crate::writer::ByteWritable;

impl<const DIGEST_LENGTH: usize> ByteWritable for UnknownHash<DIGEST_LENGTH> {
    fn write(&self, writer: &mut crate::writer::ByteWriter) {
        writer.write_bytes(&self.data);
    }
}

impl ByteWritable for EManifestStorageFlags {
    fn write(&self, writer: &mut crate::writer::ByteWriter) {
        writer.write(&(*self as u8));
    }
}

impl ByteWritable for EChunkStorageFlags {
    fn write(&self, writer: &mut crate::writer::ByteWriter) {
        let value = match self {
            EChunkStorageFlags::None => 0u8,
            EChunkStorageFlags::Compressed => 1u8,
            EChunkStorageFlags::Encrypted => 2u8,
        };
        writer.write(&value);
    }
}

impl ByteWritable for EChunkHashFlags {
    fn write(&self, writer: &mut crate::writer::ByteWriter) {
        let value = match self {
            EChunkHashFlags::None => 0u8,
            EChunkHashFlags::RollingPoly64 => 1u8,
            EChunkHashFlags::Sha1 => 2u8,
            EChunkHashFlags::Both => 3u8,
        };
        writer.write(&value);
    }
}

impl ByteWritable for EChunkVersion {
    fn write(&self, writer: &mut crate::writer::ByteWriter) {
        writer.write(&self.to_i32());
    }
}

impl ByteWritable for EFeatureLevel {
    fn write(&self, writer: &mut crate::writer::ByteWriter) {
        writer.write(&self.to_i32());
    }
}
