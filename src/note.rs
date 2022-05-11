use std::cell::RefCell;
use std::fs;
use std::io;
use std::path::PathBuf;

use snafu::{ResultExt, Snafu};

use xi_rope::tree::Node;
use xi_rope::{Rope, RopeInfo};

use crate::markdown::ToMarkdown;
use crate::section::Section;

#[derive(Debug, Snafu)]
pub enum NoteError {
    #[snafu(display("failed to open note {:?}", path))]
    Open { source: io::Error, path: PathBuf },

    #[snafu(display("failed to save note: {:?}", path))]
    Save { source: io::Error, path: PathBuf },

    #[snafu(display("section '{}' not found", section))]
    SectionNotFound { section: String },
}

type Result<T> = std::result::Result<T, NoteError>;

/// Low-level structure wrapping a markdown note.
pub struct Note {
    rope: RefCell<Node<RopeInfo>>,
    path: PathBuf,
}

impl Note {
    pub(crate) fn open(path: PathBuf) -> Result<Self> {
        let contents =
            fs::read_to_string(&path).with_context(|_| OpenSnafu { path: path.clone() })?;
        let rope = Rope::from(contents);

        // TODO: Parse YAML frontmatter and extract the metadata here.

        Ok(Self {
            rope: RefCell::new(rope),
            path,
        })
    }

    pub fn save(&self) -> Result<()> {
        // FIXME(0.1): We should validate that the contents of the note didn't change since it was opened.
        //             If it did, abort saving. (maybe add a `force` parameter to force saving?).
        fs::write(&self.path, &self.rope.borrow().to_string()).context(SaveSnafu {
            path: self.path.clone(),
        })
    }

    fn section<'a>(&'a self, name: Option<&str>) -> Option<Section<'a>> {
        let root_section: Section<'a> = Section::new(0, .., &self.rope);
        match name {
            Some(s) => root_section.subsection(s),
            None => Some(root_section),
        }
    }

    pub fn body(&self, section: Option<&str>) -> Result<String> {
        let section = self
            .section(section)
            .ok_or_else(|| NoteError::SectionNotFound {
                section: String::from(section.unwrap()),
            })?;
        Ok(section.body())
    }

    pub fn append<T: ToMarkdown>(&self, data: T, section: Option<&str>) -> Result<()> {
        let mut section = self
            .section(section)
            .ok_or_else(|| NoteError::SectionNotFound {
                section: String::from(section.unwrap()),
            })?;

        section.append(data);

        Ok(())
    }

    pub fn trim_end(&self, section: Option<&str>) -> Result<()> {
        let mut section = self
            .section(section)
            .ok_or_else(|| NoteError::SectionNotFound {
                section: String::from(section.unwrap()),
            })?;

        section.trim_end();

        Ok(())
    }
}

impl ToString for Note {
    fn to_string(&self) -> String {
        self.rope.borrow().to_string()
    }
}
