pub trait ToMarkdown {
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

pub struct CheckListItem<T: AsRef<str>> {
    checked: bool,
    text: T,
}

impl<T: AsRef<str>> CheckListItem<T> {
    pub fn new(text: T) -> Self {
        Self {
            checked: false,
            text,
        }
    }

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

pub struct Link<T: AsRef<str>> {
    reference: T,
}

impl<T: AsRef<str>> ToMarkdown for Link<T> {
    fn to_markdown(&self) -> String {
        format!("[[{}]]", self.reference.as_ref())
    }
}
