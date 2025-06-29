use core::fmt;

use crate::helper;

use super::shared::{FGuid, FSHAHash};

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct FChunkInfo {
    pub(crate) guid: FGuid,
    pub(crate) hash: u64,
    pub(crate) sha_hash: FSHAHash,
    pub(crate) group_num: u8,
    pub(crate) uncompressed_size: u32,
    pub(crate) compressed_size: i64,
}

impl PartialEq for FChunkInfo {
    fn eq(&self, other: &Self) -> bool {
        self.guid == other.guid
    }
}

impl fmt::Debug for FChunkInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FChunkInfo")
            .field("guid_str", &self.guid.to_string())
            .field("guid a", &self.guid.a)
            .field("guid b", &self.guid.b)
            .field("guid c", &self.guid.c)
            .field("guid d", &self.guid.d)
            .field("hash", &self.hash_str())
            .field("sha_hash", &self.sha_hash)
            .field("group_num_str", &self.group_num_str())
            .field("group_num", &self.group_num)
            .field("uncompressed_size", &self.uncompressed_size)
            .field("compressed_size", &self.compressed_size)
            .finish()
    }
}

impl FChunkInfo {
    pub fn guid(&self) -> &FGuid {
        &self.guid
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn hash_str(&self) -> String {
        let hex_hash = helper::to_hex(self.hash);
        let len = hex_hash.len();
        if len < 16 {
            "0".repeat(16 - len) + &hex_hash
        } else {
            hex_hash
        }
    }

    pub fn sha_hash(&self) -> &FSHAHash {
        &self.sha_hash
    }

    pub fn group_num(&self) -> u8 {
        self.group_num
    }

    pub fn group_num_str(&self) -> String {
        let str = self.group_num.to_string();
        if str.len() == 1 {
            "0".to_owned() + &str
        } else {
            str
        }
    }

    pub fn uncompressed_size(&self) -> u32 {
        self.uncompressed_size
    }

    pub fn compressed_size(&self) -> i64 {
        self.compressed_size
    }
}

// Add ByteWritable implementation for FChunkInfo
use crate::writer::ByteWritable;

impl ByteWritable for FChunkInfo {
    fn write(&self, writer: &mut crate::writer::ByteWriter) {
        writer.write(&self.guid);
        writer.write(&self.hash);
        writer.write(&self.sha_hash);
        writer.write(&self.group_num);
        writer.write(&self.uncompressed_size);
        writer.write(&self.compressed_size);
    }
}
