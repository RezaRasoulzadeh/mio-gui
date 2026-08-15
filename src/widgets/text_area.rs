// text_area.rs

use crate::{
    Direction, FocusPolicy, Key, KeyState, KeyboardEvent, LogicalConstraints, LogicalPoint,
    ResolvedTheme, SemanticRole, Semantics, TextInput, TextInputDraws, TextInputLayout, TextSystem,
    TextWrap,
};

#[derive(Clone, Debug, PartialEq)]
pub struct TextArea {
    pub input: TextInput,
    minimum_lines: usize,
}

impl TextArea {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            input: TextInput::new(label),
            minimum_lines: 3,
        }
    }

    pub fn with_text(label: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            input: TextInput::with_text(label, text),
            minimum_lines: 3,
        }
    }

    pub fn minimum_lines(&self) -> usize {
        self.minimum_lines
    }

    pub fn set_minimum_lines(&mut self, minimum_lines: usize) {
        self.minimum_lines = minimum_lines.max(1);
    }

    pub fn handle_key(&mut self, event: &KeyboardEvent) -> bool {
        if event.state == KeyState::Pressed
            && event.key == Key::Enter
            && !self.input.disabled
            && !self.input.read_only
        {
            self.input.edit.replace_selection("\n");
            return true;
        }
        self.input.handle_key(event)
    }

    pub fn handle_key_with_layout(
        &mut self,
        event: &KeyboardEvent,
        layout: &TextInputLayout,
    ) -> bool {
        if event.state == KeyState::Pressed && !self.input.disabled {
            match event.key {
                Key::Arrow(crate::ArrowKey::Up) => {
                    return layout.move_vertical(&mut self.input, false, event.modifiers.shift);
                }
                Key::Arrow(crate::ArrowKey::Down) => {
                    return layout.move_vertical(&mut self.input, true, event.modifiers.shift);
                }
                Key::Home if event.modifiers.control || event.modifiers.meta => {
                    return self.input.edit.move_to_start(event.modifiers.shift);
                }
                Key::End if event.modifiers.control || event.modifiers.meta => {
                    return self.input.edit.move_to_end(event.modifiers.shift);
                }
                Key::Home if !event.modifiers.control && !event.modifiers.meta => {
                    return layout.move_to_line_edge(&mut self.input, false, event.modifiers.shift);
                }
                Key::End if !event.modifiers.control && !event.modifiers.meta => {
                    return layout.move_to_line_edge(&mut self.input, true, event.modifiers.shift);
                }
                _ => {}
            }
        }
        self.handle_key(event)
    }

    pub fn semantics(&self) -> Semantics {
        let mut semantics = self.input.semantics();
        semantics.role = SemanticRole::MultilineTextField;
        semantics
    }

    pub fn focus_policy(&self) -> FocusPolicy {
        self.input.focus_policy()
    }

    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> TextInputLayout {
        self.input.layout_with_lines(
            text_system,
            theme,
            inherited_direction,
            constraints,
            TextWrap::Word,
            self.minimum_lines,
        )
    }

    pub fn draws(
        &self,
        layout: &TextInputLayout,
        origin: LogicalPoint,
        theme: &ResolvedTheme,
    ) -> TextInputDraws {
        layout.draws(&self.input, origin, theme)
    }
}

#[cfg(test)]
mod tests {
    use super::TextArea;
    use crate::{
        Direction, Key, KeyboardEvent, LogicalConstraints, LogicalPoint, LogicalSize, TextSystem,
        ThemeController, ThemeDefinition, UserPreferences,
    };

    fn theme() -> crate::ResolvedTheme {
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default())
    }

    #[test]
    fn enter_inserts_a_newline_unless_read_only() {
        let mut area = TextArea::with_text("Notes", "first");
        assert!(area.handle_key(&KeyboardEvent::pressed(Key::Enter)));
        assert_eq!(area.input.text(), "first\n");
        area.input.read_only = true;
        assert!(!area.handle_key(&KeyboardEvent::pressed(Key::Enter)));
    }

    #[test]
    fn vertical_keys_move_between_visual_lines_and_extend_selection() {
        let mut area = TextArea::with_text("Notes", "first line\n👩‍💻 second\nthird");
        area.input.edit.set_caret(2);
        let theme = theme();
        let mut text_system = TextSystem::new();
        let layout = area.layout(
            &mut text_system,
            &theme,
            Direction::Ltr,
            LogicalConstraints::tight(LogicalSize::new(240.0, 120.0)),
        );

        assert!(area.handle_key_with_layout(
            &KeyboardEvent::pressed(Key::Arrow(crate::ArrowKey::Down)),
            &layout,
        ));
        let second_line = area.input.edit.selection().start;
        assert!(second_line >= "first line\n".len());
        assert!(area.input.text().is_char_boundary(second_line));

        let mut selecting = KeyboardEvent::pressed(Key::Arrow(crate::ArrowKey::Down));
        selecting.modifiers.shift = true;
        assert!(area.handle_key_with_layout(&selecting, &layout));
        assert!(!area.input.edit.selection().is_empty());
        let mut shrinking = KeyboardEvent::pressed(Key::Arrow(crate::ArrowKey::Up));
        shrinking.modifiers.shift = true;
        assert!(area.handle_key_with_layout(&shrinking, &layout));
        assert!(area.input.edit.selection().is_empty());
        assert!(area.handle_key_with_layout(&shrinking, &layout));
        assert!(!area.input.edit.selection().is_empty());
        assert!(area.input.edit.caret() < area.input.edit.selection_anchor());
    }

    #[test]
    fn home_and_end_use_visual_line_edges_unless_primary_modified() {
        let mut area = TextArea::with_text("Notes", "first\nsecond\nthird");
        area.input.edit.set_caret("first\nsec".len());
        let theme = theme();
        let mut text_system = TextSystem::new();
        let layout = area.layout(
            &mut text_system,
            &theme,
            Direction::Ltr,
            LogicalConstraints::tight(LogicalSize::new(240.0, 120.0)),
        );

        assert!(area.handle_key_with_layout(&KeyboardEvent::pressed(Key::Home), &layout));
        assert_eq!(area.input.edit.caret(), "first\n".len());
        assert!(area.handle_key_with_layout(&KeyboardEvent::pressed(Key::End), &layout));
        assert_eq!(area.input.edit.caret(), "first\nsecond".len());

        let mut document_home = KeyboardEvent::pressed(Key::Home);
        document_home.modifiers.control = true;
        assert!(area.handle_key_with_layout(&document_home, &layout));
        assert_eq!(area.input.edit.caret(), 0);
    }

    #[test]
    fn minimum_lines_and_wrapping_define_multiline_height() {
        let mut area = TextArea::with_text("Notes", "one two three four five six");
        area.set_minimum_lines(4);
        let theme = theme();
        let mut text_system = TextSystem::new();
        let layout = area.layout(
            &mut text_system,
            &theme,
            Direction::Ltr,
            LogicalConstraints::loose(LogicalSize::new(80.0, 400.0)),
        );
        assert!(layout.text.lines.len() > 1);
        assert!(layout.size.height >= theme.typography.line_height * 4.0);
    }

    #[test]
    fn focused_selection_emits_rectangles_across_rtl_lines() {
        let mut area = TextArea::with_text("Notes", "سطر اول\nسطر دوم");
        area.input.focused = true;
        area.input.edit.set_selection(0..area.input.text().len());
        let theme = theme();
        let mut text_system = TextSystem::new();
        let layout = area.layout(
            &mut text_system,
            &theme,
            Direction::Rtl,
            LogicalConstraints::loose(LogicalSize::new(120.0, 300.0)),
        );
        let draws = area.draws(&layout, LogicalPoint::default(), &theme);
        assert!(layout.text.lines.len() >= 2);
        assert!(draws.editing.len() >= 2);
    }
}
