use super::constant::*;

use scroll::{ctx, Error, Pread, LE};
use binread::{};

#[derive(Debug, Clone, PartialEq)]
pub struct MagicHeader
{
    header1: [u8; 4],
    header2: [u8; 7]
}


impl MagicHeader {
    pub fn validate(&self) -> Result<(), Error> {
        if self.header1 != MAGIC {
            return Err(Error::BadInput {
                size: 4,
                msg: "magic mismatch",
            });
        }
        if self.header2 != HEADER {
            return Err(Error::BadInput {
                size: 7,
                msg: "header suffix mismatch",
            });
        }
        Ok(())
    }

    pub fn parser(data: &[u8]) -> Result<Self, Error> {
        let hdr = Self::try_from(data)?;
        hdr.validate()?;
        Ok(hdr)
    }
}

impl TryFrom<&[u8]> for MagicHeader {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        const EXPECTED_LEN: usize = 11;
        if data.len() < EXPECTED_LEN {
            return Err(Error::BadInput { size: data.len(), msg: "insufficient data for magic header" });
        }

        Ok(MagicHeader {
            header1: data.pread(0)?,
            header2: data.pread(4)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileIndexHeader
{
    zlib_status: bool,
    compress_length: u64,
    index_length: u64
}


impl FileIndexHeader {
    pub fn parser(data: &[u8]) -> Result<Self, Error> {
        let hdr = Self::try_from(data)?;
        Ok(hdr)
    }
}


impl TryFrom<&[u8]> for FileIndexHeader {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        const EXPECTED_LEN: usize = 17;
        if data.len() < EXPECTED_LEN {
            return Err(Error::BadInput {
                size: data.len(),
                msg: "insufficient data for file index header",
            });
        }

        let zlib_status = data.pread::<u8>(0)? != 0;
        let compress_length = data.pread_with(1, scroll::LE)?;
        let index_length = data.pread_with(9, scroll::LE)?;

        Ok(FileIndexHeader {
            zlib_status,
            compress_length,
            index_length,
        })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct IndexHeader
{
    flag: [u8; 4],
    index_length: u64,
}

