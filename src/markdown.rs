//! Utilities to manipulate obsidian-markdown structures.

/// Types implementing `ToMarkdown` can be converted to a markdown string.
pub trait ToMarkdown {
    /// Serializes the object to markdown.
    fn to_markdown(&self) -> String;
}

impl<T> ToMarkdown for &T
where
    T: ToMarkdown,
{
    fn to_markdown(&self) -> String {
        (*self).to_markdown()
    }
}

impl ToMarkdown for String {
    fn to_markdown(&self) -> String {
        self.clone()
    }
}

impl ToMarkdown for &str {
    fn to_markdown(&self) -> String {
        String::from(*self)
    }
}

/// Struct mapping to a markdown checklist item.
pub struct CheckListItem<T: AsRef<str>> {
    checked: bool,
    text: T,
}

impl<T: AsRef<str>> CheckListItem<T> {
    /// Creates a new checklist item.
    pub fn new(text: T) -> Self {
        Self {
            checked: false,
            text,
        }
    }

    /// Sets the state of the checkbox.
    pub fn set(&mut self, checked: bool) {
        self.checked = checked;
    }
}

impl<T: AsRef<str>> ToMarkdown for CheckListItem<T> {
    fn to_markdown(&self) -> String {
        if self.checked {
            format!("- [x] {}\n", self.text.as_ref())
        } else {
            format!("- [ ] {}\n", self.text.as_ref())
        }
    }
}

/// Struct mapping to an obsidian-style local link.
///
/// e.g. `[[Some Page]]`
pub struct LocalLink<T: AsRef<str>> {
    reference: T,
}

impl<T: AsRef<str>> LocalLink<T> {
    /// Creates a new local link.
    pub fn new(reference: T) -> Self {
        Self { reference }
    }
}

impl<T: AsRef<str>> ToMarkdown for LocalLink<T> {
    fn to_markdown(&self) -> String {
        format!("[[{}]]", self.reference.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checklist_item_unchecked() {
        let item = CheckListItem::new("bing bong");
        assert_eq!(item.to_markdown(), "- [ ] bing bong\n");
    }

    #[test]
    fn checklist_item_checked() {
        let mut item = CheckListItem::new("bing bong");
        item.set(true);

        assert_eq!(item.to_markdown(), "- [x] bing bong\n");
    }

    #[test]
    fn link() {
        let lnk = LocalLink::new("Some Page");
        assert_eq!(lnk.to_markdown(), "[[Some Page]]");
    }
}
