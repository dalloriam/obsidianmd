use std::path::{Path, PathBuf};

use snafu::{ensure, ResultExt, Snafu};

use walkdir::WalkDir;

use crate::Note;

#[derive(Debug, Snafu)]
pub enum VaultError {
    #[snafu(display("failed to list vault entries"))]
    ListEntry { source: walkdir::Error },

    #[snafu(display("failed to open note"))]
    OpenNote { source: crate::note::NoteError },

    #[snafu(display("vault does not exist"))]
    VaultDoesNotExist,
}

type Result<T> = std::result::Result<T, VaultError>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub daily: Option<PathBuf>,
    pub templates: Option<PathBuf>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Vault {
    config: Config,
    path: PathBuf,
}

impl Vault {
    pub fn open(path: PathBuf, config: Config) -> Result<Self> {
        ensure!(path.exists(), VaultDoesNotExistSnafu);
        Ok(Self { config, path })
    }

    // Lookup a note by name in the vault, returning its path.
    pub fn lookup(&self, note_name: &str) -> Result<Option<PathBuf>> {
        // TODO: Cache?
        for entry in WalkDir::new(&self.path).into_iter() {
            let entry = entry.context(ListEntrySnafu)?;
            let name_lower = entry.file_name().to_string_lossy().to_lowercase();

            if entry.file_name().to_string_lossy() == format!("{note_name}.md")
                || name_lower == format!("{}.md", note_name.to_lowercase())
            {
                return Ok(Some(PathBuf::from(entry.path())));
            }
        }
        Ok(None)
    }

    pub fn note<P: AsRef<Path>>(&self, path: P) -> Result<Note> {
        Note::open(self.path.join(path.as_ref())).context(OpenNoteSnafu)
    }
}
