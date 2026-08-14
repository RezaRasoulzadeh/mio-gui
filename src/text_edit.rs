// text_edit.rs
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

use crate::{ClipboardError, TextClipboard};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TextEditState {
    text: String,
    selection: Range<usize>,
    composition: Option<Composition>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Composition {
    range: Range<usize>,
    original_selection: Range<usize>,
    replaced_text: String,
}

impl TextEditState {
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let caret = text.len();
        Self {
            text,
            selection: caret..caret,
            composition: None,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn selection(&self) -> Range<usize> {
        self.selection.clone()
    }

    pub fn set_selection(&mut self, selection: Range<usize>) {
        self.commit_composition();
        let start = snap_boundary(&self.text, selection.start, false);
        let end = snap_boundary(&self.text, selection.end, true);
        self.selection = start.min(end)..start.max(end);
    }

    pub fn set_caret(&mut self, byte_index: usize) {
        self.commit_composition();
        let caret = snap_boundary(&self.text, byte_index, false);
        self.selection = caret..caret;
    }

    pub fn selected_text(&self) -> &str {
        &self.text[self.selection.clone()]
    }

    pub fn copy(&self) -> String {
        self.selected_text().to_owned()
    }

    pub fn copy_to(&self, clipboard: &mut impl TextClipboard) -> Result<bool, ClipboardError> {
        if self.selection.is_empty() {
            return Ok(false);
        }
        clipboard.write_text(self.selected_text())?;
        Ok(true)
    }

    pub fn cut(&mut self) -> String {
        self.commit_composition();
        let copied = self.copy();
        self.replace_selection("");
        copied
    }

    pub fn cut_to(&mut self, clipboard: &mut impl TextClipboard) -> Result<bool, ClipboardError> {
        self.commit_composition();
        if !self.copy_to(clipboard)? {
            return Ok(false);
        }
        self.replace_selection_raw("");
        Ok(true)
    }

    pub fn paste(&mut self, text: &str) {
        self.replace_selection(text);
    }

    pub fn paste_from(
        &mut self,
        clipboard: &mut impl TextClipboard,
    ) -> Result<bool, ClipboardError> {
        let text = clipboard.read_text()?;
        if text.is_empty() {
            return Ok(false);
        }
        self.paste(&text);
        Ok(true)
    }

    pub fn delete_backward(&mut self) -> bool {
        self.commit_composition();
        if !self.selection.is_empty() {
            self.replace_selection_raw("");
            return true;
        }
        let caret = self.selection.start;
        let previous = previous_boundary(&self.text, caret);
        if previous == caret {
            return false;
        }
        self.selection = previous..caret;
        self.replace_selection_raw("");
        true
    }

    pub fn delete_forward(&mut self) -> bool {
        self.commit_composition();
        if !self.selection.is_empty() {
            self.replace_selection_raw("");
            return true;
        }
        let caret = self.selection.start;
        let next = next_boundary(&self.text, caret);
        if next == caret {
            return false;
        }
        self.selection = caret..next;
        self.replace_selection_raw("");
        true
    }

    pub fn select_all(&mut self) {
        self.commit_composition();
        self.selection = 0..self.text.len();
    }

    pub fn replace_selection(&mut self, replacement: &str) {
        self.commit_composition();
        self.replace_selection_raw(replacement);
    }

    pub fn composition_range(&self) -> Option<Range<usize>> {
        self.composition
            .as_ref()
            .map(|composition| composition.range.clone())
    }

    pub fn begin_composition(&mut self, preedit: &str) {
        self.cancel_composition();
        let original_selection = self.selection.clone();
        let replaced_text = self.selected_text().to_owned();
        let start = self.selection.start;
        self.replace_selection_raw(preedit);
        self.composition = Some(Composition {
            range: start..start + preedit.len(),
            original_selection,
            replaced_text,
        });
    }

    pub fn update_composition(&mut self, preedit: &str) {
        self.update_composition_with_selection(preedit, None);
    }

    pub fn update_composition_with_selection(
        &mut self,
        preedit: &str,
        selection: Option<Range<usize>>,
    ) {
        let Some(composition) = self.composition.as_mut() else {
            self.begin_composition(preedit);
            if let Some(selection) = selection {
                self.set_preedit_selection(selection);
            }
            return;
        };
        let start = composition.range.start;
        self.text.replace_range(composition.range.clone(), preedit);
        composition.range = start..start + preedit.len();
        if let Some(selection) = selection {
            self.set_preedit_selection(selection);
        } else {
            self.selection = composition.range.end..composition.range.end;
        }
    }

    pub fn commit_composition(&mut self) {
        if let Some(composition) = self.composition.take() {
            self.selection = composition.range.end..composition.range.end;
        }
    }

    pub fn cancel_composition(&mut self) {
        let Some(composition) = self.composition.take() else {
            return;
        };
        self.text
            .replace_range(composition.range, &composition.replaced_text);
        self.selection = composition.original_selection;
    }

    fn replace_selection_raw(&mut self, replacement: &str) {
        let start = self.selection.start;
        self.text.replace_range(self.selection.clone(), replacement);
        let caret = start + replacement.len();
        self.selection = caret..caret;
    }

    fn set_preedit_selection(&mut self, selection: Range<usize>) {
        let Some(composition) = &self.composition else {
            return;
        };
        let preedit = &self.text[composition.range.clone()];
        let start = snap_boundary(preedit, selection.start, false);
        let end = snap_boundary(preedit, selection.end, true);
        self.selection = composition.range.start + start..composition.range.start + end;
    }
}

fn snap_boundary(text: &str, byte_index: usize, trailing: bool) -> usize {
    let byte_index = byte_index.min(text.len());
    let mut previous = 0;
    for (boundary, _) in text.grapheme_indices(true) {
        if boundary == byte_index {
            return boundary;
        }
        if boundary > byte_index {
            return if trailing { boundary } else { previous };
        }
        previous = boundary;
    }
    text.len()
}

fn previous_boundary(text: &str, byte_index: usize) -> usize {
    text.grapheme_indices(true)
        .map(|(index, _)| index)
        .take_while(|index| *index < byte_index)
        .last()
        .unwrap_or(byte_index)
}

fn next_boundary(text: &str, byte_index: usize) -> usize {
    text.grapheme_indices(true)
        .map(|(index, _)| index)
        .chain(std::iter::once(text.len()))
        .find(|index| *index > byte_index)
        .unwrap_or(byte_index)
}

#[cfg(test)]
mod tests {
    use super::TextEditState;
    use crate::{ClipboardError, TextClipboard};

    #[derive(Default)]
    struct MemoryClipboard {
        text: String,
        fail_reads: bool,
        fail_writes: bool,
    }

    impl TextClipboard for MemoryClipboard {
        fn read_text(&mut self) -> Result<String, ClipboardError> {
            if self.fail_reads {
                Err(ClipboardError::new("read failed"))
            } else {
                Ok(self.text.clone())
            }
        }

        fn write_text(&mut self, text: &str) -> Result<(), ClipboardError> {
            if self.fail_writes {
                Err(ClipboardError::new("write failed"))
            } else {
                self.text = text.to_owned();
                Ok(())
            }
        }
    }

    #[test]
    fn copies_and_cuts_rtl_text_in_logical_source_order() {
        let mut state = TextEditState::new("سلام دنیا");
        state.set_selection(0.."سلام".len());

        assert_eq!(state.copy(), "سلام");
        assert_eq!(state.cut(), "سلام");
        assert_eq!(state.text(), " دنیا");
        assert_eq!(state.selection(), 0..0);
    }

    #[test]
    fn copies_cuts_and_pastes_through_clipboard_boundary() {
        let mut state = TextEditState::new("سلام دنیا");
        let mut clipboard = MemoryClipboard::default();
        state.set_selection(0.."سلام".len());

        assert_eq!(state.copy_to(&mut clipboard), Ok(true));
        assert_eq!(clipboard.text, "سلام");
        assert_eq!(state.cut_to(&mut clipboard), Ok(true));
        assert_eq!(state.text(), " دنیا");
        state.set_caret(state.text().len());
        assert_eq!(state.paste_from(&mut clipboard), Ok(true));
        assert_eq!(state.text(), " دنیاسلام");
    }

    #[test]
    fn failed_clipboard_operations_do_not_mutate_editor() {
        let mut state = TextEditState::new("Mio GUI");
        state.set_selection(4..7);
        let original = state.clone();
        let mut clipboard = MemoryClipboard {
            fail_reads: true,
            fail_writes: true,
            ..MemoryClipboard::default()
        };

        assert!(state.cut_to(&mut clipboard).is_err());
        assert_eq!(state, original);
        assert!(state.paste_from(&mut clipboard).is_err());
        assert_eq!(state, original);
    }

    #[test]
    fn empty_selection_and_empty_clipboard_are_noops() {
        let mut state = TextEditState::new("Mio");
        let mut clipboard = MemoryClipboard::default();

        assert_eq!(state.copy_to(&mut clipboard), Ok(false));
        assert_eq!(state.cut_to(&mut clipboard), Ok(false));
        assert_eq!(state.paste_from(&mut clipboard), Ok(false));
        assert_eq!(state.text(), "Mio");
    }

    #[test]
    fn paste_replaces_selection_and_places_caret_after_input() {
        let mut state = TextEditState::new("نسخه old");
        let start = state.text().find("old").unwrap();
        state.set_selection(start..state.text().len());
        state.paste("جدید");

        assert_eq!(state.text(), "نسخه جدید");
        assert_eq!(state.selection(), state.text().len()..state.text().len());
    }

    #[test]
    fn selection_expands_to_combining_grapheme_boundaries() {
        let mut state = TextEditState::new("مُح");
        state.set_selection("م".len().."مُ".len());

        assert_eq!(state.selected_text(), "مُ");
    }

    #[test]
    fn caret_never_splits_an_emoji_zwj_sequence() {
        let emoji = "👩‍💻";
        let mut state = TextEditState::new(format!("{emoji}A"));
        state.set_caret("👩".len());

        assert_eq!(state.selection(), 0..0);
        state.paste("X");
        assert_eq!(state.text(), format!("X{emoji}A"));
    }

    #[test]
    fn clamps_out_of_range_selection() {
        let mut state = TextEditState::new("Mio");
        state.set_selection(2..usize::MAX);

        assert_eq!(state.selection(), 2..3);
        assert_eq!(state.selected_text(), "o");
    }

    #[test]
    fn updates_and_commits_ime_preedit() {
        let mut state = TextEditState::new("سلام ");
        state.begin_composition("د");
        state.update_composition("دن");
        state.update_composition("دنیا");

        assert_eq!(state.text(), "سلام دنیا");
        assert_eq!(
            state.composition_range(),
            Some("سلام ".len()..state.text().len())
        );
        state.commit_composition();
        assert_eq!(state.composition_range(), None);
        assert_eq!(state.selection(), state.text().len()..state.text().len());
    }

    #[test]
    fn cancelling_composition_restores_replaced_selection() {
        let mut state = TextEditState::new("نسخه old");
        let start = state.text().find("old").unwrap();
        state.set_selection(start..state.text().len());
        let original_selection = state.selection();
        state.begin_composition("جدید");

        assert_eq!(state.text(), "نسخه جدید");
        state.cancel_composition();
        assert_eq!(state.text(), "نسخه old");
        assert_eq!(state.selection(), original_selection);
    }

    #[test]
    fn ordinary_edit_commits_active_composition_first() {
        let mut state = TextEditState::new("Mio");
        state.begin_composition(" GUI");
        state.paste("!");

        assert_eq!(state.text(), "Mio GUI!");
        assert_eq!(state.composition_range(), None);
    }

    #[test]
    fn update_without_active_composition_starts_one() {
        let mut state = TextEditState::new("متن ");
        state.update_composition("آزمایشی");

        assert_eq!(state.text(), "متن آزمایشی");
        assert!(state.composition_range().is_some());
    }

    #[test]
    fn preedit_selection_is_relative_and_grapheme_safe() {
        let mut state = TextEditState::new("متن ");
        state.update_composition_with_selection("مُوقت", Some("م".len().."مُ".len()));

        let start = "متن ".len();
        assert_eq!(state.selection(), start..start + "مُ".len());
        assert_eq!(state.selected_text(), "مُ");
    }

    #[test]
    fn out_of_range_preedit_selection_clamps_to_composition() {
        let mut state = TextEditState::new("Mio ");
        state.update_composition_with_selection("GUI", Some(1..usize::MAX));

        assert_eq!(state.selected_text(), "UI");
        assert_eq!(state.selection().end, state.text().len());
    }

    #[test]
    fn backspace_deletes_one_rtl_grapheme_in_logical_order() {
        let mut state = TextEditState::new("سلام");

        assert!(state.delete_backward());
        assert_eq!(state.text(), "سلا");
        assert_eq!(state.selection(), state.text().len()..state.text().len());
    }

    #[test]
    fn forward_delete_removes_one_grapheme() {
        let mut state = TextEditState::new("سلام");
        state.set_caret(0);

        assert!(state.delete_forward());
        assert_eq!(state.text(), "لام");
        assert_eq!(state.selection(), 0..0);
    }

    #[test]
    fn deletion_never_splits_combining_or_emoji_sequences() {
        let emoji = "👩‍💻";
        let mut state = TextEditState::new(format!("مُ{emoji}"));

        assert!(state.delete_backward());
        assert_eq!(state.text(), "مُ");
        assert!(state.delete_backward());
        assert_eq!(state.text(), "");
        assert!(!state.delete_backward());
        assert!(!state.delete_forward());
    }

    #[test]
    fn deletion_replaces_selection_before_adjacent_graphemes() {
        let mut state = TextEditState::new("Mio GUI");
        state.set_selection(4..7);

        assert!(state.delete_backward());
        assert_eq!(state.text(), "Mio ");
        assert_eq!(state.selection(), 4..4);
    }

    #[test]
    fn select_all_commits_composition_and_selects_source() {
        let mut state = TextEditState::new("متن ");
        state.begin_composition("جدید");
        state.select_all();

        assert_eq!(state.composition_range(), None);
        assert_eq!(state.selected_text(), "متن جدید");
    }
}
