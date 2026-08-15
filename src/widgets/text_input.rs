// text_input.rs

use crate::{
    Direction, DirectionSetting, FocusPolicy, Key, KeyState, KeyboardEvent, LogicalConstraints,
    LogicalPoint, LogicalSize, RectDraw, ResolvedTheme, SemanticAction, SemanticEditableText,
    SemanticRole, Semantics, Text, TextDraw, TextEditState, TextStyle, TextSystem, TextWrap,
};

#[derive(Clone, Debug, PartialEq)]
pub struct TextInput {
    label: String,
    pub edit: TextEditState,
    placeholder: String,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub invalid: bool,
    pub focused: bool,
    pub direction: DirectionSetting,
}

impl TextInput {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            edit: TextEditState::default(),
            placeholder: String::new(),
            disabled: false,
            read_only: false,
            required: false,
            invalid: false,
            focused: false,
            direction: DirectionSetting::Inherit,
        }
    }

    pub fn with_text(label: impl Into<String>, text: impl Into<String>) -> Self {
        let mut input = Self::new(label);
        input.edit = TextEditState::new(text);
        input
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    pub fn text(&self) -> &str {
        self.edit.text()
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) -> bool {
        if self.disabled || self.read_only || self.text() == text.as_ref() {
            return false;
        }
        self.edit.select_all();
        self.edit.replace_selection(text.as_ref());
        true
    }

    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::TextField)
            .with_name(self.label.clone())
            .with_value(self.text().to_owned())
            .with_placeholder(self.placeholder.clone())
            .with_action(SemanticAction::Focus);
        semantics.editable_text =
            SemanticEditableText::new(self.text(), self.edit.selection_anchor(), self.edit.caret());
        if !self.disabled && !self.read_only {
            semantics.add_action(SemanticAction::SetValue);
            semantics.add_action(SemanticAction::SetTextSelection);
        }
        semantics.state.disabled = self.disabled;
        semantics.state.read_only = self.read_only;
        semantics.state.required = self.required;
        semantics.state.invalid = self.invalid;
        semantics.state.focused = self.focused;
        semantics
    }

    pub fn focus_policy(&self) -> FocusPolicy {
        FocusPolicy {
            focusable: true,
            disabled: self.disabled,
            ..FocusPolicy::default()
        }
    }

    pub fn handle_key(&mut self, event: &KeyboardEvent) -> bool {
        if self.disabled || event.state != KeyState::Pressed {
            return false;
        }
        match &event.key {
            Key::Character(text)
                if !self.read_only
                    && !event.modifiers.control
                    && !event.modifiers.meta
                    && !event.modifiers.alt
                    && !text.chars().any(char::is_control) =>
            {
                self.edit.replace_selection(text);
                true
            }
            Key::Space
                if !self.read_only
                    && !event.modifiers.control
                    && !event.modifiers.meta
                    && !event.modifiers.alt =>
            {
                self.edit.replace_selection(" ");
                true
            }
            Key::Backspace
                if !self.read_only
                    && !event.modifiers.control
                    && !event.modifiers.meta
                    && !event.modifiers.alt =>
            {
                self.edit.delete_backward()
            }
            Key::Delete
                if !self.read_only
                    && !event.modifiers.control
                    && !event.modifiers.meta
                    && !event.modifiers.alt =>
            {
                self.edit.delete_forward()
            }
            Key::Arrow(crate::ArrowKey::Left)
                if !event.modifiers.control && !event.modifiers.meta && !event.modifiers.alt =>
            {
                self.edit.move_backward(event.modifiers.shift)
            }
            Key::Arrow(crate::ArrowKey::Right)
                if !event.modifiers.control && !event.modifiers.meta && !event.modifiers.alt =>
            {
                self.edit.move_forward(event.modifiers.shift)
            }
            Key::Home if !event.modifiers.alt => self.edit.move_to_start(event.modifiers.shift),
            Key::End if !event.modifiers.alt => self.edit.move_to_end(event.modifiers.shift),
            _ => false,
        }
    }

    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> TextInputLayout {
        self.layout_with_lines(
            text_system,
            theme,
            inherited_direction,
            constraints,
            TextWrap::NoWrap,
            1,
        )
    }

    pub(crate) fn layout_with_lines(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
        wrap: TextWrap,
        minimum_lines: usize,
    ) -> TextInputLayout {
        let direction = self.direction.resolve(inherited_direction);
        let padding_inline = theme.spacing.medium;
        let padding_block = theme.spacing.small;
        let displayed_text = if self.text().is_empty() {
            self.placeholder()
        } else {
            self.text()
        };
        let mut text = Text::new(displayed_text);
        text.direction = match direction {
            Direction::Ltr => DirectionSetting::Ltr,
            Direction::Rtl => DirectionSetting::Rtl,
        };
        text.wrap = wrap;
        text.style = TextStyle {
            family: Some(theme.typography.family.clone()),
            font_size: theme.typography.size,
            line_height: theme.typography.line_height,
            letter_spacing: theme.typography.letter_spacing,
            weight: theme.typography.weight,
            ..TextStyle::default()
        };
        let text = text.layout(
            text_system,
            direction,
            LogicalConstraints::loose(LogicalSize::new(
                subtract(constraints.max.width, padding_inline * 2.0),
                subtract(constraints.max.height, padding_block * 2.0),
            )),
        );
        let natural = LogicalSize::new(
            (text.size.width + padding_inline * 2.0).max(160.0),
            text.size
                .height
                .max(text.style.line_height * minimum_lines.max(1) as f32)
                + padding_block * 2.0,
        );
        let size = constraints.constrain(natural);
        let text_x = match direction {
            Direction::Ltr => padding_inline,
            Direction::Rtl => (size.width - padding_inline - text.size.width).max(padding_inline),
        };
        TextInputLayout {
            size,
            direction,
            text,
            text_origin: LogicalPoint::new(
                text_x,
                ((size.height - natural.height + padding_block * 2.0) * 0.5).max(0.0),
            ),
            showing_placeholder: self.text().is_empty(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextInputLayout {
    pub size: LogicalSize,
    pub direction: Direction,
    pub text: crate::TextLayout,
    pub text_origin: LogicalPoint,
    pub showing_placeholder: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextInputDraws {
    pub background: RectDraw,
    pub editing: Vec<RectDraw>,
    pub text: Vec<TextDraw>,
}

impl TextInputLayout {
    pub fn move_to_line_edge(&self, input: &mut TextInput, end: bool, selecting: bool) -> bool {
        if input.disabled || self.showing_placeholder || self.text.lines.is_empty() {
            return false;
        }
        let anchor = input.edit.selection_anchor();
        let caret = input.edit.caret();
        let line = self
            .text
            .lines
            .iter()
            .filter(|line| line.source.start <= caret && caret <= line.source.end)
            .next_back()
            .unwrap_or(&self.text.lines[0]);
        let target = if end {
            line.source.end
        } else {
            line.source.start
        };
        if caret == target {
            return false;
        }
        if selecting {
            input.edit.set_selection_from_anchor(anchor, target);
        } else {
            input.edit.set_caret(target);
        }
        true
    }

    pub fn move_vertical(&self, input: &mut TextInput, down: bool, selecting: bool) -> bool {
        if input.disabled || self.showing_placeholder || self.text.lines.is_empty() {
            return false;
        }
        let anchor = input.edit.selection_anchor();
        let caret = input.edit.caret();
        let current = self
            .text
            .lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.source.start <= caret && caret <= line.source.end)
            .map(|(index, _)| index)
            .next_back()
            .unwrap_or(0);
        let target = if down {
            current
                .checked_add(1)
                .filter(|index| *index < self.text.lines.len())
        } else {
            current.checked_sub(1)
        };
        let Some(target) = target else {
            return false;
        };
        let line = &self.text.lines[current];
        let local_caret = caret.saturating_sub(line.source.start);
        let x = line
            .shaped
            .caret_positions(local_caret)
            .into_iter()
            .next()
            .unwrap_or(if line.shaped.rtl {
                line.shaped.width
            } else {
                0.0
            });
        let target_line = &self.text.lines[target];
        let target_caret = target_line.source.start + target_line.shaped.hit_test(x).byte_index;
        if selecting {
            input.edit.set_selection_from_anchor(anchor, target_caret);
        } else {
            input.edit.set_caret(target_caret);
        }
        true
    }

    pub fn hit_test(&self, input: &TextInput, origin: LogicalPoint, point: LogicalPoint) -> usize {
        if self.showing_placeholder || input.text().is_empty() {
            return 0;
        }
        let local_y = point.y - origin.y - self.text_origin.y;
        let line = self
            .text
            .lines
            .iter()
            .min_by(|left, right| {
                let left_distance = (local_y - left.offset.y).abs();
                let right_distance = (local_y - right.offset.y).abs();
                left_distance.total_cmp(&right_distance)
            })
            .unwrap();
        let local_x = point.x - origin.x - self.text_origin.x - line.offset.x;
        line.source.start + line.shaped.hit_test(local_x).byte_index
    }

    pub fn draws(
        &self,
        input: &TextInput,
        origin: LogicalPoint,
        theme: &ResolvedTheme,
    ) -> TextInputDraws {
        let opacity = if input.disabled { 0.45 } else { 1.0 };
        let content = if self.showing_placeholder {
            input.placeholder()
        } else {
            input.text()
        };
        let text_color = if self.showing_placeholder {
            theme.colors.text_muted
        } else {
            theme.colors.text
        };
        let border = if input.invalid {
            theme.colors.error
        } else if input.focused {
            theme.colors.focus
        } else {
            theme.colors.border
        };
        TextInputDraws {
            background: RectDraw {
                position: [origin.x, origin.y],
                size: [self.size.width, self.size.height],
                radii: [theme.radii.medium; 4],
                color: faded(theme.colors.surface.to_array(), opacity),
                border_width: theme.borders.thin,
                border_color: faded(border.to_array(), opacity),
            },
            editing: self.editing_draws(input, origin, theme, opacity),
            text: self.text.draws(
                content,
                add(origin, self.text_origin),
                faded(text_color.to_array(), opacity),
            ),
        }
    }

    fn editing_draws(
        &self,
        input: &TextInput,
        origin: LogicalPoint,
        theme: &ResolvedTheme,
        opacity: f32,
    ) -> Vec<RectDraw> {
        if !input.focused || input.disabled {
            return Vec::new();
        }
        let text_origin = add(origin, self.text_origin);
        if self.showing_placeholder {
            let x = match self.direction {
                Direction::Ltr => text_origin.x,
                Direction::Rtl => text_origin.x + self.text.size.width,
            };
            return vec![caret_draw(
                x,
                text_origin.y,
                self.text.style.line_height,
                theme,
                opacity,
            )];
        }
        let selection = input.edit.selection();
        if !selection.is_empty() {
            let mut color = theme.colors.primary.to_array();
            color[3] *= 0.25 * opacity;
            return self
                .text
                .lines
                .iter()
                .filter_map(|line| {
                    let start = selection.start.max(line.source.start);
                    let end = selection.end.min(line.source.end);
                    (start < end)
                        .then_some((line, start - line.source.start..end - line.source.start))
                })
                .flat_map(|(line, range)| {
                    let line_origin = LogicalPoint::new(
                        text_origin.x + line.offset.x,
                        text_origin.y + line.offset.y,
                    );
                    line.shaped
                        .selection_rects(range)
                        .into_iter()
                        .map(move |selection| RectDraw {
                            position: [line_origin.x + selection.x, line_origin.y],
                            size: [selection.width, self.text.style.line_height],
                            radii: [theme.radii.small; 4],
                            color,
                            border_width: 0.0,
                            border_color: [0.0; 4],
                        })
                })
                .collect();
        }
        let Some(line) = self.text.lines.iter().find(|line| {
            selection.start >= line.source.start && selection.start <= line.source.end
        }) else {
            return Vec::new();
        };
        let line_origin =
            LogicalPoint::new(text_origin.x + line.offset.x, text_origin.y + line.offset.y);
        let caret = selection.start - line.source.start;
        let x = line
            .shaped
            .caret_positions(caret)
            .into_iter()
            .next()
            .unwrap_or(match self.direction {
                Direction::Ltr => 0.0,
                Direction::Rtl => line.shaped.width,
            });
        vec![caret_draw(
            line_origin.x + x,
            line_origin.y,
            self.text.style.line_height,
            theme,
            opacity,
        )]
    }
}

fn caret_draw(x: f32, y: f32, height: f32, theme: &ResolvedTheme, opacity: f32) -> RectDraw {
    RectDraw {
        position: [x - 0.5, y],
        size: [1.0, height],
        radii: [0.5; 4],
        color: faded(theme.colors.focus.to_array(), opacity),
        border_width: 0.0,
        border_color: [0.0; 4],
    }
}

fn add(left: LogicalPoint, right: LogicalPoint) -> LogicalPoint {
    LogicalPoint::new(left.x + right.x, left.y + right.y)
}

fn subtract(value: f32, amount: f32) -> f32 {
    if value.is_finite() {
        (value - amount).max(0.0)
    } else {
        value
    }
}

fn faded(mut color: [f32; 4], opacity: f32) -> [f32; 4] {
    color[3] *= opacity;
    color
}

#[cfg(test)]
mod tests {
    use super::TextInput;
    use crate::{
        Direction, Key, KeyboardEvent, LogicalConstraints, LogicalPoint, SemanticRole, TextSystem,
        ThemeController, ThemeDefinition, UserPreferences,
    };

    fn theme() -> crate::ResolvedTheme {
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default())
    }

    #[test]
    fn keyboard_editing_uses_grapheme_safe_edit_state() {
        let mut input = TextInput::with_text("Name", "مُ");
        assert!(input.handle_key(&KeyboardEvent::pressed(Key::Character("ن".into()))));
        assert_eq!(input.text(), "مُن");
        assert!(input.handle_key(&KeyboardEvent::pressed(Key::Space)));
        assert_eq!(input.text(), "مُن ");
        assert!(input.handle_key(&KeyboardEvent::pressed(Key::Backspace)));
        assert_eq!(input.text(), "مُن");
        assert!(input.handle_key(&KeyboardEvent::pressed(Key::Backspace)));
        assert_eq!(input.text(), "مُ");
    }

    #[test]
    fn pointer_hit_testing_and_shift_arrows_expose_selection_feedback() {
        let mut input = TextInput::with_text("Name", "Mio");
        input.focused = true;
        let theme = theme();
        let mut text_system = TextSystem::new();
        let origin = LogicalPoint::new(10.0, 20.0);
        let layout = input.layout(
            &mut text_system,
            &theme,
            Direction::Ltr,
            LogicalConstraints::unconstrained(),
        );
        let end = layout.hit_test(
            &input,
            origin,
            LogicalPoint::new(origin.x + layout.size.width, origin.y),
        );
        input.edit.set_caret(end);
        let mut event = KeyboardEvent::pressed(Key::Arrow(crate::ArrowKey::Left));
        event.modifiers.shift = true;
        assert!(input.handle_key(&event));
        assert!(!input.edit.selection().is_empty());
        assert!(!layout.draws(&input, origin, &theme).editing.is_empty());
    }

    #[test]
    fn held_backspace_repeat_continues_grapheme_safe_deletion() {
        let mut input = TextInput::with_text("Name", "abمُ");
        let mut event = KeyboardEvent::pressed(Key::Backspace);
        event.repeat = true;

        assert!(input.handle_key(&event));
        assert_eq!(input.text(), "ab");
        assert!(input.handle_key(&event));
        assert_eq!(input.text(), "a");
    }

    #[test]
    fn read_only_and_disabled_inputs_reject_edits() {
        let mut input = TextInput::with_text("Name", "Mio");
        input.read_only = true;
        assert!(!input.handle_key(&KeyboardEvent::pressed(Key::Character("!".into()))));
        assert!(!input.semantics().supports(crate::SemanticAction::SetValue));
        assert!(input.handle_key(&KeyboardEvent::pressed(Key::Home)));
        assert_eq!(input.edit.caret(), 0);
        assert_eq!(input.text(), "Mio");
        input.read_only = false;
        input.disabled = true;
        assert!(!input.handle_key(&KeyboardEvent::pressed(Key::Delete)));
        assert_eq!(input.text(), "Mio");
    }

    #[test]
    fn semantics_expose_value_and_form_states() {
        let mut input = TextInput::with_text("Name", "Reza");
        input.set_placeholder("Enter name");
        input.required = true;
        input.invalid = true;
        let semantics = input.semantics();
        assert_eq!(semantics.role, SemanticRole::TextField);
        assert_eq!(semantics.placeholder.as_deref(), Some("Enter name"));
        assert_eq!(semantics.name.as_deref(), Some("Name"));
        assert_eq!(semantics.value.as_deref(), Some("Reza"));
        assert!(semantics.state.required);
        assert!(semantics.state.invalid);
    }

    #[test]
    fn empty_placeholder_layout_mirrors_and_uses_muted_paint() {
        let mut input = TextInput::new("Query");
        input.set_placeholder("Search");
        let theme = theme();
        let mut text_system = TextSystem::new();
        let ltr = input.layout(
            &mut text_system,
            &theme,
            Direction::Ltr,
            LogicalConstraints::unconstrained(),
        );
        let rtl = input.layout(
            &mut text_system,
            &theme,
            Direction::Rtl,
            LogicalConstraints::unconstrained(),
        );
        assert!(ltr.text_origin.x < rtl.text_origin.x);
        assert!(ltr.showing_placeholder);
        let draws = rtl.draws(&input, LogicalPoint::default(), &theme);
        assert_eq!(draws.text[0].color, theme.colors.text_muted.to_array());
    }

    #[test]
    fn focused_selection_emits_shaped_highlights_before_text() {
        let mut input = TextInput::with_text("Name", "abc אבג");
        input.focused = true;
        input.edit.set_selection(1.."abc אב".len());
        let theme = theme();
        let mut text_system = TextSystem::new();
        let layout = input.layout(
            &mut text_system,
            &theme,
            Direction::Ltr,
            LogicalConstraints::unconstrained(),
        );
        let draws = layout.draws(&input, LogicalPoint::new(3.0, 5.0), &theme);
        assert!(!draws.editing.is_empty());
        assert!(draws.editing.iter().all(|draw| draw.size[0] > 0.0));
        assert!(input.semantics().state.focused);
        assert_eq!(draws.background.border_color, theme.colors.focus.to_array());
    }

    #[test]
    fn focused_empty_rtl_input_places_caret_at_inline_start() {
        let mut input = TextInput::new("Name");
        input.set_placeholder("نام");
        input.focused = true;
        let theme = theme();
        let mut text_system = TextSystem::new();
        let layout = input.layout(
            &mut text_system,
            &theme,
            Direction::Rtl,
            LogicalConstraints::unconstrained(),
        );
        let draws = layout.draws(&input, LogicalPoint::default(), &theme);
        assert_eq!(draws.editing.len(), 1);
        assert!(draws.editing[0].position[0] > layout.size.width * 0.5);
        assert_eq!(draws.editing[0].size[0], 1.0);
    }
}
