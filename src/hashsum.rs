use std::{
    fmt::{Formatter, Display},
    str::FromStr,
    path::Path,
    io::{self, BufReader, Read},
    fs::File,
};
use crate::error::{Error, IntoErrorContext};
use ring::digest::Context;
use walkdir::WalkDir;

pub const HASH_SIZE: usize = 32;

#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct HashSum ([u8; HASH_SIZE]);

impl HashSum {
    pub fn from_bytes(bytes: &[u8]) -> Result<HashSum, Error>  {
        let bytes = bytes.try_into().or(Err(Error::BadHashSize))?;
        Ok(HashSum(bytes))
    }
}

impl FromStr for HashSum {
    type Err = Error;

    fn from_str(data: &str) -> Result<HashSum, Error> {
        let bytes = (0..data.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&data[i..=i + 1], 16))
            .collect::<Result<Vec<_>, _>>()?;

        Self::from_bytes(&bytes)
    }
}

impl Display for HashSum {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for byte in self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

fn update_hash_file(
    path: &Path,
    context: &mut Context
) -> Result<(), io::Error> {
    // Adapted from the rust cookbook
    const CHUNK_SIZE: usize = 2048;
    let mut buffer = [0; CHUNK_SIZE];

    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }
    Ok(())
}

fn update_hash_direntry(
    entry: &walkdir::DirEntry,
    prefix: &Path,
    context: &mut Context
) -> Result<(), Error> {
    let relative_path = entry.path().strip_prefix(prefix)?;
    // println!("{}", relative_path.display());

    let meta = entry.metadata()?;
    let path_bytes = relative_path.to_str().unwrap().as_bytes();
    context.update(path_bytes);
    context.update(&[0]);
    context.update(
        &[if entry.file_type().is_file() {
            0x01
        } else {
            0x02
        }]
    );
    context.update(&meta.len().to_le_bytes());

    if entry.file_type().is_file() {
        update_hash_file(entry.path(), context).with_context(entry.path())?;
    }
    Ok(())
}

pub fn hash_directory(directory: &Path) -> Result<HashSum, Error> {
    let mut context = Context::new(&ring::digest::SHA256);
    for entry in WalkDir::new(directory).sort_by_file_name() {
        let entry = entry?;
        if entry.file_type().is_symlink() {
            return Err(Error::Symlink(entry.path().to_path_buf()))
        }
        update_hash_direntry(&entry, directory, &mut context)?;
    }
    let digest = context.finish();
    HashSum::from_bytes(digest.as_ref())
}
