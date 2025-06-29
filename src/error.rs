
#[derive(Debug)]
pub enum ParseError {
    InvalidMagic,
    InvalidData,
    InvalidDigest,
    InvalidStorageFlag,
    OffsetMismatch,
    DecompressionError,
    HashMismatch,
    SizeMismatch,
    Overflow
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::InvalidMagic => write!(f, "Invalid magic"),
            ParseError::InvalidData => write!(f, "Invalid data"),
            ParseError::InvalidDigest => write!(f, "Invalid digest"),
            ParseError::Overflow => write!(f, "Overflow"),
            ParseError::InvalidStorageFlag => write!(f, "Invalid storage flag"),
            ParseError::OffsetMismatch => write!(f, "Offset mismatch"),
            ParseError::DecompressionError => write!(f, "Decompression failed"),
            ParseError::HashMismatch => write!(f, "Hash does not match"),
            ParseError::SizeMismatch => write!(f, "Sizes does not match"),
            
        }
    }
}

impl std::error::Error for ParseError {}