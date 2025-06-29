use super::{chunk_part::FChunkPart, file_manifest::FFileManifest, shared::UnknownHash};
use crate::{error::ParseError, reader::ByteReader, ParseResult};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FFileManifestList {
    pub(crate) _version: u8,
    pub(crate) _size: u32,
    pub(crate) _count: u32,
    pub(crate) entries: Vec<FFileManifest>,
}

impl FFileManifestList {
    /// This function is used to parse a FFileManifestList from a ByteReader
    pub fn parse(reader: &mut ByteReader) -> ParseResult<FFileManifestList> {
        let reader_start = reader.tell();

        let size = reader.read()?;
        let version = reader.read()?;
        let count = reader.read()?;

        let mut entries: Vec<FFileManifest> = vec![Default::default(); count as usize];

        for entry in entries.iter_mut() {
            entry.filename = reader.read()?;
        }

        for entry in entries.iter_mut() {
            entry.syslink_target = reader.read()?;
        }

        for entry in entries.iter_mut() {
            entry.hash = reader.read()?;
        }

        for entry in entries.iter_mut() {
            entry.flags = reader.read()?;
        }

        for entry in entries.iter_mut() {
            entry.install_tags = reader.read_array(|reader| reader.read())?;
        }

        for entry in entries.iter_mut() {
            let part_count = reader.read::<u32>()?;
            let mut file_offset = 0;

            //make sure we have enough capacity to push every parts without reallocating
            entry
                .chunk_parts
                .reserve(part_count as usize - entry.chunk_parts.capacity());
            for _ in 0..part_count {
                let part = FChunkPart::parse(reader, file_offset)?;
                file_offset += part.size() as usize;
                entry.chunk_parts.push(part);
            }
        }

        if version >= 1 {
            for entry in entries.iter_mut() {
                let has_md5 = reader.read::<u32>()?;
                if has_md5 != 0 {
                    entry.hash_md5 = UnknownHash::from_byte_reader(reader).ok();
                }
            }

            for entry in entries.iter_mut() {
                entry.mime_type = reader.read().ok();
            }
        }

        if version >= 2 {
            for entry in entries.iter_mut() {
                entry.hash_sha256 = UnknownHash::from_byte_reader(reader).ok();
            }
        }

        for entry in entries.iter_mut() {
            entry.file_size = entry.chunk_parts.iter().map(|part| part.size()).sum();
        }

        if reader_start + size as usize != reader.tell() {
            println!("FileManifestList size mismatch: expected {} but got {}\nFileManifestList version : {}", size, reader.tell() - reader_start, version);
            return Err(ParseError::InvalidData);
        }

        Ok(FFileManifestList {
            _version: version,
            _size: size,
            _count: count,
            entries,
        })
    }

    /// Writes the FFileManifestList to a ByteWriter
    pub fn write(&self, writer: &mut crate::writer::ByteWriter) {
        use crate::writer::ByteWritable;

        // Calculate the size first by writing to a temporary buffer
        let mut temp_writer = crate::writer::ByteWriter::new();
        temp_writer.write(&self._version);
        temp_writer.write(&(self.entries.len() as u32));

        // Write filenames
        for entry in &self.entries {
            temp_writer.write(&entry.filename);
        }

        // Write symlink targets
        for entry in &self.entries {
            temp_writer.write(&entry.syslink_target);
        }

        // Write hashes
        for entry in &self.entries {
            temp_writer.write(&entry.hash);
        }

        // Write flags
        for entry in &self.entries {
            temp_writer.write(&entry.flags);
        }

        // Write install tags arrays
        for entry in &self.entries {
            temp_writer.write_array(&entry.install_tags);
        }

        // Write chunk parts
        for entry in &self.entries {
            temp_writer.write(&(entry.chunk_parts.len() as u32));
            for part in &entry.chunk_parts {
                part.write(&mut temp_writer);
            }
        }

        // Handle version-specific fields
        if self._version >= 1 {
            // Write MD5 hashes
            for entry in &self.entries {
                if let Some(ref md5_hash) = entry.hash_md5 {
                    temp_writer.write(&1u32); // has_md5 = true
                    temp_writer.write(md5_hash);
                } else {
                    temp_writer.write(&0u32); // has_md5 = false
                }
            }

            // Write MIME types
            for entry in &self.entries {
                if let Some(ref mime_type) = entry.mime_type {
                    temp_writer.write(mime_type);
                } else {
                    temp_writer.write(&String::new());
                }
            }
        }

        if self._version >= 2 {
            // Write SHA256 hashes
            for entry in &self.entries {
                if let Some(ref sha256_hash) = entry.hash_sha256 {
                    temp_writer.write(sha256_hash);
                }
            }
        }

        let size = (temp_writer.tell() + 4) as u32; // +4 for the size field itself

        // Write the actual data with correct size
        writer.write(&size);
        writer.write(&self._version);
        writer.write(&(self.entries.len() as u32));

        // Write filenames
        for entry in &self.entries {
            writer.write(&entry.filename);
        }

        // Write symlink targets
        for entry in &self.entries {
            writer.write(&entry.syslink_target);
        }

        // Write hashes
        for entry in &self.entries {
            writer.write(&entry.hash);
        }

        // Write flags
        for entry in &self.entries {
            writer.write(&entry.flags);
        }

        // Write install tags arrays
        for entry in &self.entries {
            writer.write_array(&entry.install_tags);
        }

        // Write chunk parts
        for entry in &self.entries {
            writer.write(&(entry.chunk_parts.len() as u32));
            for part in &entry.chunk_parts {
                part.write(writer);
            }
        }

        // Handle version-specific fields
        if self._version >= 1 {
            // Write MD5 hashes
            for entry in &self.entries {
                if let Some(ref md5_hash) = entry.hash_md5 {
                    writer.write(&1u32); // has_md5 = true
                    writer.write(md5_hash);
                } else {
                    writer.write(&0u32); // has_md5 = false
                }
            }

            // Write MIME types
            for entry in &self.entries {
                if let Some(ref mime_type) = entry.mime_type {
                    writer.write(mime_type);
                } else {
                    writer.write(&String::new());
                }
            }
        }

        if self._version >= 2 {
            // Write SHA256 hashes
            for entry in &self.entries {
                if let Some(ref sha256_hash) = entry.hash_sha256 {
                    writer.write(sha256_hash);
                }
            }
        }
    }

    pub fn entries(&self) -> &Vec<FFileManifest> {
        &self.entries
    }
}
