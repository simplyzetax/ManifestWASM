use super::{
    chunk_part::FChunkPart,
    shared::{FSHAHash, UnknownHash, MD5_DIGEST_SIZE, SHA256_DIGEST_SIZE},
};

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct FFileManifest {
    pub(crate) filename: String,
    pub(crate) syslink_target: String,
    pub(crate) hash: FSHAHash,
    pub(crate) flags: u8,
    pub(crate) install_tags: Vec<String>,
    pub(crate) chunk_parts: Vec<FChunkPart>,
    pub(crate) mime_type: Option<String>,
    pub(crate) hash_md5: Option<UnknownHash<MD5_DIGEST_SIZE>>,
    pub(crate) hash_sha256: Option<UnknownHash<SHA256_DIGEST_SIZE>>,
    pub(crate) file_size: u32,
}

impl PartialEq for FFileManifest {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && self.filename == other.filename
    }
}

impl FFileManifest {
    pub fn read_only(&self) -> bool {
        self.flags & 0x01 == 1
    }

    pub fn compressed(&self) -> bool {
        self.flags & 0x02 == 1
    }

    pub fn executable(&self) -> bool {
        self.flags & 0x04 == 1
    }

    pub fn sha_hash(&self) -> &FSHAHash {
        &self.hash
    }

    pub fn md5_hash(&self) -> Option<&UnknownHash<MD5_DIGEST_SIZE>> {
        self.hash_md5.as_ref()
    }

    pub fn sha256_hash(&self) -> Option<&UnknownHash<SHA256_DIGEST_SIZE>> {
        self.hash_sha256.as_ref()
    }

    pub fn file_size(&self) -> u32 {
        self.file_size
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn syslink_target(&self) -> &str {
        &self.syslink_target
    }

    pub fn mime_type(&self) -> Option<&str> {
        self.mime_type.as_deref()
    }

    pub fn chunk_parts(&self) -> &[FChunkPart] {
        &self.chunk_parts
    }

    pub fn install_tags(&self) -> &Vec<String> {
        &self.install_tags
    }

    pub fn hash(&self) -> &FSHAHash {
        &self.hash
    }

    pub fn raw_flags(&self) -> u8 {
        self.flags
    }
}
