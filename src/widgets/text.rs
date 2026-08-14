// text.rs

use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use crate::{
    Direction, DirectionSetting, InlineAlignment, LogicalConstraints, LogicalPoint, LogicalSize,
    SemanticColorToken, SemanticRole, Semantics, ShapedLine, TextAlign, TextDirection, TextDraw,
    TextStyle, TextSystem,
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextWrap {
    NoWrap,
    #[default]
    Word,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Text {
    content: String,
    pub style: TextStyle,
    pub color: SemanticColorToken,
    pub align: InlineAlignment,
    pub direction: DirectionSetting,
    pub wrap: TextWrap,
    pub max_lines: Option<usize>,
}

impl Text {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            style: TextStyle::default(),
            color: SemanticColorToken::Text,
            align: InlineAlignment::Start,
            direction: DirectionSetting::Inherit,
            wrap: TextWrap::Word,
            max_lines: None,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }

    pub fn semantics(&self) -> Semantics {
        Semantics::new(SemanticRole::Text).with_name(self.content.clone())
    }

    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> TextLayout {
        let direction = self.direction.resolve(inherited_direction);
        let mut style = self.style.clone();
        style.direction = match direction {
            Direction::Ltr => TextDirection::Ltr,
            Direction::Rtl => TextDirection::Rtl,
        };
        let available_width = constraints.max.width;
        let ranges = match self.wrap {
            TextWrap::NoWrap => explicit_line_ranges(&self.content),
            TextWrap::Word => {
                wrapped_line_ranges(&self.content, available_width, &style, text_system)
            }
        };
        let height_limit = if constraints.max.height.is_finite() {
            (constraints.max.height / style.line_height.max(1.0)).floor() as usize
        } else {
            usize::MAX
        };
        let line_limit = self.max_lines.unwrap_or(usize::MAX).min(height_limit);
        let truncated = ranges.len() > line_limit;
        let shaped = ranges
            .into_iter()
            .take(line_limit)
            .map(|source| {
                let line = text_system.shape_line_with_style(&self.content[source.clone()], &style);
                (source, line)
            })
            .collect::<Vec<_>>();
        let natural_width = shaped
            .iter()
            .map(|(_, line)| line.width)
            .fold(0.0_f32, f32::max);
        let natural_height = shaped.len() as f32 * style.line_height.max(1.0);
        let size = constraints.constrain(LogicalSize::new(natural_width, natural_height));
        let lines = shaped
            .into_iter()
            .enumerate()
            .map(|(index, (source, shaped))| TextLayoutLine {
                offset: LogicalPoint::new(
                    self.align.offset(direction, size.width, shaped.width),
                    index as f32 * style.line_height.max(1.0),
                ),
                source,
                shaped,
            })
            .collect();

        TextLayout {
            size,
            direction,
            align: self.align,
            style,
            color: self.color,
            lines,
            truncated,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextLayoutLine {
    pub source: Range<usize>,
    pub offset: LogicalPoint,
    pub shaped: ShapedLine,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextLayout {
    pub size: LogicalSize,
    pub direction: Direction,
    pub align: InlineAlignment,
    pub style: TextStyle,
    pub color: SemanticColorToken,
    pub lines: Vec<TextLayoutLine>,
    pub truncated: bool,
}

impl TextLayout {
    pub fn draws(&self, content: &str, origin: LogicalPoint, color: [f32; 4]) -> Vec<TextDraw> {
        self.lines
            .iter()
            .map(|line| {
                let rtl = line.shaped.rtl;
                let baseline_x = if rtl {
                    origin.x + line.offset.x + line.shaped.width
                } else {
                    origin.x + line.offset.x
                };
                TextDraw {
                    text: content[line.source.clone()].to_owned(),
                    style: self.style.clone(),
                    baseline: [baseline_x, origin.y + line.offset.y + line.shaped.baseline],
                    align: TextAlign::Start,
                    color,
                }
            })
            .collect()
    }
}

fn explicit_line_ranges(text: &str) -> Vec<Range<usize>> {
    if text.is_empty() {
        return std::iter::once(0..0).collect();
    }
    let mut ranges = Vec::new();
    let mut start = 0;
    for (index, character) in text.char_indices() {
        if character == '\n' {
            ranges.push(start..index);
            start = index + character.len_utf8();
        }
    }
    ranges.push(start..text.len());
    ranges
}

fn wrapped_line_ranges(
    text: &str,
    maximum_width: f32,
    style: &TextStyle,
    text_system: &mut TextSystem,
) -> Vec<Range<usize>> {
    let paragraphs = explicit_line_ranges(text);
    if !maximum_width.is_finite() {
        return paragraphs;
    }
    let maximum_width = maximum_width.max(0.0);
    let mut lines = Vec::new();
    for paragraph in paragraphs {
        if paragraph.is_empty() {
            lines.push(paragraph);
            continue;
        }
        let paragraph_text = &text[paragraph.clone()];
        let boundaries = paragraph_text
            .grapheme_indices(true)
            .map(|(index, _)| paragraph.start + index)
            .chain(std::iter::once(paragraph.end))
            .collect::<Vec<_>>();
        let mut start_index = 0;
        while start_index + 1 < boundaries.len() {
            let mut end_index = start_index + 1;
            let mut accepted_end = end_index;
            let mut whitespace_end = None;
            while end_index < boundaries.len() {
                let candidate = boundaries[start_index]..boundaries[end_index];
                let width = text_system
                    .shape_line_with_style(&text[candidate.clone()], style)
                    .width;
                if width > maximum_width && accepted_end > start_index + 1 {
                    break;
                }
                accepted_end = end_index;
                if text[candidate.clone()]
                    .graphemes(true)
                    .next_back()
                    .is_some_and(|grapheme| grapheme.chars().all(char::is_whitespace))
                {
                    whitespace_end = Some(end_index);
                }
                end_index += 1;
            }
            let chosen_end = if end_index < boundaries.len() {
                whitespace_end
                    .filter(|index| *index > start_index)
                    .unwrap_or(accepted_end)
            } else {
                accepted_end
            };
            lines.push(boundaries[start_index]..boundaries[chosen_end]);
            start_index = chosen_end;
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use crate::{Direction, DirectionSetting, InlineAlignment, LogicalConstraints, LogicalSize};

    use super::{Text, TextWrap};

    #[test]
    fn semantics_expose_complete_source_text() {
        let text = Text::new("رابط کاربری");
        let semantics = text.semantics();

        assert_eq!(semantics.role, crate::SemanticRole::Text);
        assert_eq!(semantics.name.as_deref(), Some("رابط کاربری"));
        assert_eq!(semantics.actions().len(), 0);
    }

    #[test]
    fn inherited_direction_controls_base_direction_and_start_alignment() {
        let mut system = crate::TextSystem::new();
        let mut text = Text::new("Mio-GUI 2");
        text.wrap = TextWrap::NoWrap;
        text.align = InlineAlignment::Start;
        let constraints = LogicalConstraints::tight(LogicalSize::new(200.0, 24.0));

        let ltr = text.layout(&mut system, Direction::Ltr, constraints);
        let rtl = text.layout(&mut system, Direction::Rtl, constraints);

        assert_eq!(ltr.lines[0].offset.x, 0.0);
        assert_eq!(rtl.lines[0].offset.x, 200.0 - rtl.lines[0].shaped.width);
        assert_eq!(ltr.style.direction, crate::TextDirection::Ltr);
        assert_eq!(rtl.style.direction, crate::TextDirection::Rtl);
    }

    #[test]
    fn local_direction_override_wins_over_inheritance() {
        let mut system = crate::TextSystem::new();
        let mut text = Text::new("نسخه Mio-GUI 2");
        text.direction = DirectionSetting::Rtl;
        let layout = text.layout(
            &mut system,
            Direction::Ltr,
            LogicalConstraints::unconstrained(),
        );

        assert_eq!(layout.direction, Direction::Rtl);
        assert_eq!(layout.style.direction, crate::TextDirection::Rtl);
    }

    #[test]
    fn wrapping_preserves_grapheme_boundaries_and_source_order() {
        let mut system = crate::TextSystem::new();
        let text = Text::new("می‌روم خانه");
        let layout = text.layout(
            &mut system,
            Direction::Rtl,
            LogicalConstraints::loose(LogicalSize::new(48.0, 200.0)),
        );

        assert!(layout.lines.len() > 1);
        assert!(
            layout
                .lines
                .windows(2)
                .all(|lines| lines[0].source.end == lines[1].source.start)
        );
        assert_eq!(layout.lines.first().unwrap().source.start, 0);
        assert_eq!(
            layout.lines.last().unwrap().source.end,
            text.content().len()
        );
    }

    #[test]
    fn max_lines_and_height_constraints_report_truncation() {
        let mut system = crate::TextSystem::new();
        let mut text = Text::new("one two three four five six");
        text.max_lines = Some(2);
        let layout = text.layout(
            &mut system,
            Direction::Ltr,
            LogicalConstraints::loose(LogicalSize::new(45.0, 200.0)),
        );

        assert_eq!(layout.lines.len(), 2);
        assert!(layout.truncated);
        assert_eq!(layout.size.height, 48.0);
    }

    #[test]
    fn paint_draws_preserve_line_source_ranges_and_resolved_baselines() {
        let mut system = crate::TextSystem::new();
        let text = Text::new("first\nsecond");
        let layout = text.layout(
            &mut system,
            Direction::Ltr,
            LogicalConstraints::unconstrained(),
        );
        let draws = layout.draws(
            text.content(),
            crate::LogicalPoint::new(10.0, 20.0),
            [1.0; 4],
        );

        assert_eq!(draws.len(), 2);
        assert_eq!(draws[0].text, "first");
        assert_eq!(draws[1].text, "second");
        assert!(draws[1].baseline[1] > draws[0].baseline[1]);
    }
}
