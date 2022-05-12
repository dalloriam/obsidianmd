use std::path::{Path, PathBuf};

use snafu::{ensure, ResultExt, Snafu};

use walkdir::WalkDir;

use crate::Note;

#[derive(Debug, Snafu)]
pub enum VaultError {
    #[snafu(display("multiple daily notes of the same name found"))]
    DuplicateDailyNote,

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

/// Struct for interacting with an obsidian vault.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Vault {
    config: Config,
    path: PathBuf,
}

impl Vault {
    /// Opens an obsidian vault.
    ///
    /// Opening a vault touches no file in the vault.
    ///
    /// # Errors
    /// Will return an error if the vault path does not exist on disk.
    pub fn open(path: PathBuf, config: Config) -> Result<Self> {
        ensure!(path.exists(), VaultDoesNotExistSnafu);
        Ok(Self { config, path })
    }

    /// Lookup a note by name in the vault, returning its path.
    ///
    /// # Errors
    /// Will return an error if a file in the vault is inaccessible.
    pub fn lookup(&self, note_name: &str) -> Result<Vec<PathBuf>> {
        // TODO: Cache?
        let mut buf = Vec::new();

        for entry in WalkDir::new(&self.path).into_iter() {
            let entry = entry.context(ListEntrySnafu)?;
            let name_lower = entry.file_name().to_string_lossy().to_lowercase();

            if entry.file_name().to_string_lossy() == format!("{note_name}.md")
                || name_lower == format!("{}.md", note_name.to_lowercase())
            {
                buf.push(PathBuf::from(entry.path()));
            }
        }
        Ok(buf)
    }

    /// Get a note by its path relative to the root of the vault.
    ///
    /// # Errors
    /// Will return an error if the note does not exist, or if the user does not have
    /// permission to open the note for reading.
    pub fn note<P: AsRef<Path>>(&self, path: P) -> Result<Note> {
        Note::open(self.path.join(path.as_ref())).context(OpenNoteSnafu)
    }

    /// Get the daily note for a given date.
    ///
    /// If a daily note directory was specified in the vault configuration, the note will
    /// be looked up from there. If not, `Vault::lookup` will be called to search for
    /// it in the whole vault.
    ///
    /// # Errors
    /// Will return an error if multiple notes with the same date exist, or if the user
    /// does not have permission to open the note for reading.
    pub fn daily(&self, date: time::Date) -> Result<Option<Note>> {
        match &self.config.daily {
            Some(dailies) => self.note(dailies.join(format!("{date}.md"))).map(Some),
            None => {
                let hits = self.lookup(&date.to_string())?;
                ensure!(hits.len() <= 1, DuplicateDailyNoteSnafu);
                hits.get(0).map(|p| self.note(p)).transpose()
            }
        }
    }
}
