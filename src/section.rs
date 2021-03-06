use std::cell::RefCell;

use regex::RegexBuilder;

use xi_rope::interval::IntervalBounds;
use xi_rope::tree::Node;
use xi_rope::Rope;
use xi_rope::{Cursor, Interval, RopeInfo};

use crate::markdown as md;

pub(crate) struct Section<'a> {
    weight: usize,
    interval: Interval,
    rope: &'a RefCell<Node<RopeInfo>>,
}

impl<'a> Section<'a> {
    pub fn new<I: IntervalBounds>(
        weight: usize,
        interval: I,
        rope: &'a RefCell<Node<RopeInfo>>,
    ) -> Self {
        Self {
            weight,
            interval: interval.into_interval(rope.borrow().len()),
            rope,
        }
    }

    pub fn body(&self) -> String {
        self.rope.borrow().slice(self.interval).to_string()
    }

    fn find_subsection(&self, name: &str) -> Option<(Interval, usize)> {
        let rope = self.rope.borrow();

        // Step 1 - Find the section header of our target section.
        let mut cursor = Cursor::new(&rope, 0);
        let mut lines_raw = rope.lines_raw(..);

        let raw_pat = format!(r"^#{{{},}} {}$", self.weight + 1, name);
        let pattern = RegexBuilder::new(&raw_pat)
            .case_insensitive(true)
            .multi_line(true)
            .build()
            .unwrap(); // FIXME(pr): Handle error

        let section_offset = xi_rope::find::find(
            &mut cursor,
            &mut lines_raw,
            xi_rope::find::CaseMatching::CaseInsensitive,
            &raw_pat,
            Some(&pattern),
        )?;

        // Since we know by our regex that the cursor is on a linebreak, we skip it.
        cursor.next_codepoint()?;

        let section_header = rope.slice(section_offset..cursor.pos()).to_string();
        let section_header_weight = section_header.split(' ').next().unwrap().len(); // Unwrap is safe because it's the first one.

        let section_body_begin = cursor.pos();

        let raw_pat = format!(r"^#{{1,{section_header_weight}}} .*$");
        let pattern = RegexBuilder::new(&raw_pat)
            .multi_line(true)
            .build()
            .unwrap();

        let mut lines_raw = rope.lines_raw(cursor.pos()..);

        let next_section_offset = xi_rope::find::find(
            &mut cursor,
            &mut lines_raw,
            xi_rope::find::CaseMatching::CaseInsensitive,
            &raw_pat,
            Some(&pattern),
        )
        .unwrap_or(rope.len());

        Some((
            (section_body_begin..next_section_offset).into(),
            section_header_weight,
        ))
    }

    /// Find a section in a note by name.
    ///
    /// Note: Assumes the section name is unique.
    pub fn subsection(&self, name: &str) -> Option<Section<'a>> {
        let (interval, weight) = self.find_subsection(name)?;

        let r: &'a RefCell<Node<RopeInfo>> = self.rope;
        Some(Section::new(weight, interval, r))
    }

    /// Append text at the end of the section.
    pub fn append<T: md::ToMarkdown>(&mut self, data: T) {
        let mut rope = self.rope.borrow_mut();

        let diff = Rope::from(data.to_markdown());
        let new_end = self.interval.end + diff.len();

        rope.edit(self.interval.end..self.interval.end, diff);

        self.interval.end = new_end;
    }

    /// Trims trailing whitespace at the end of the section.
    pub fn trim_end(&mut self) {
        let mut rope = self.rope.borrow_mut();

        // Get the range of whitespace to remove.
        let interval = {
            let mut cursor = Cursor::new(&rope, self.interval.end);
            while let Some(codepoint) = cursor.prev_codepoint() {
                if !codepoint.is_whitespace() {
                    break;
                }
            }

            cursor.next_codepoint();

            // The last \n of a section is important so we skip it.
            let end = rope
                .at_or_prev_codepoint_boundary(self.interval.end - 1)
                .unwrap();

            cursor.pos()..end
        };

        self.interval.end = interval.start;
        rope.edit(interval, "");
    }

    // TODO: Add way to list checkboxes, recuperate and toggle their state.
    // TODO: Add way to extract all links.
    // TODO: Add way to extract code blocks.
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use xi_rope::{tree::Node, Rope, RopeInfo};

    use super::Section;

    const TEST_NOTE: &str = include_str!("data/test.md");

    fn get_rope() -> RefCell<Node<RopeInfo>> {
        RefCell::new(Rope::from(TEST_NOTE))
    }

    #[test]
    fn root_section_body() {
        let rope = get_rope();
        let s = Section::new(0, .., &rope);
        assert_eq!(s.body(), TEST_NOTE);
    }

    #[test]
    fn subsection_body() {
        let rope = get_rope();
        let s = Section::new(0, .., &rope);
        let section = s.subsection("subsection c").unwrap();

        assert_eq!(section.body(), "\nText For Section C\n\n");
    }

    #[test]
    fn subsection_trim_space() {
        let rope = get_rope();
        let s = Section::new(0, .., &rope);
        let mut sub = s.subsection("subsection c").unwrap();
        sub.trim_end();

        assert_eq!(sub.body(), "\nText For Section C");
    }

    #[test]
    fn subsection_append() {
        let rope = get_rope();
        let s = Section::new(0, .., &rope);
        let mut sub = s.subsection("subsection c").unwrap();
        sub.append("bing bong");

        assert_eq!(sub.body(), "\nText For Section C\n\nbing bong");
    }
}
