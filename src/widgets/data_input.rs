// data_input.rs

use crate::{
    ArrowKey, Button, ButtonDraws, ButtonLayout, ComponentStyle, Direction, DirectionSetting,
    FocusPolicy, Key, KeyState, KeyboardEvent, LogicalConstraints, LogicalPoint, ResolvedTheme,
    SemanticAction, SemanticNumericValue, SemanticRole, Semantics, TextSystem, VisualVariant,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CivilDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CivilDateError;

impl std::fmt::Display for CivilDateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("date must contain a valid Gregorian year, month, and day")
    }
}

impl std::error::Error for CivilDateError {}

impl CivilDate {
    pub fn new(year: i32, month: u8, day: u8) -> Result<Self, CivilDateError> {
        let date = Self { year, month, day };
        (year != 0 && (1..=12).contains(&month) && (1..=date.days_in_month()).contains(&day))
            .then_some(date)
            .ok_or(CivilDateError)
    }

    pub fn days_in_month(self) -> u8 {
        match self.month {
            4 | 6 | 9 | 11 => 30,
            2 if is_leap_year(self.year) => 29,
            2 => 28,
            _ => 31,
        }
    }

    fn add_days(self, delta: i8) -> Self {
        let mut date = self;
        for _ in 0..delta.unsigned_abs() {
            date = if delta.is_positive() {
                date.next_day()
            } else {
                date.previous_day()
            };
        }
        date
    }

    fn next_day(self) -> Self {
        if self.day < self.days_in_month() {
            Self {
                day: self.day + 1,
                ..self
            }
        } else if self.month < 12 {
            Self {
                month: self.month + 1,
                day: 1,
                ..self
            }
        } else {
            Self {
                year: self.year + 1,
                month: 1,
                day: 1,
            }
        }
    }

    fn previous_day(self) -> Self {
        if self.day > 1 {
            Self {
                day: self.day - 1,
                ..self
            }
        } else {
            let (year, month) = if self.month > 1 {
                (self.year, self.month - 1)
            } else {
                (self.year - 1, 12)
            };
            let mut date = Self {
                year,
                month,
                day: 1,
            };
            date.day = date.days_in_month();
            date
        }
    }

    fn add_months(self, delta: i8) -> Self {
        let month_index = self.year * 12 + i32::from(self.month) - 1 + i32::from(delta);
        let year = month_index.div_euclid(12);
        let month = u8::try_from(month_index.rem_euclid(12) + 1).unwrap();
        let mut date = Self {
            year,
            month,
            day: 1,
        };
        date.day = self.day.min(date.days_in_month());
        date
    }
}

impl std::fmt::Display for CivilDate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        )
    }
}

fn is_leap_year(year: i32) -> bool {
    year.rem_euclid(4) == 0 && (year.rem_euclid(100) != 0 || year.rem_euclid(400) == 0)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Calendar {
    label: String,
    selected: CivilDate,
    pub disabled: bool,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
}

impl Calendar {
    pub fn new(label: impl Into<String>, selected: CivilDate) -> Self {
        Self {
            label: label.into(),
            selected,
            disabled: false,
            style: ComponentStyle {
                variant: VisualVariant::Outline,
                ..ComponentStyle::default()
            },
            direction: DirectionSetting::Inherit,
        }
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn selected(&self) -> CivilDate {
        self.selected
    }
    pub fn set_selected(&mut self, selected: CivilDate) -> bool {
        if self.disabled {
            return false;
        }
        let changed = self.selected != selected;
        self.selected = selected;
        changed
    }
    pub fn next_day(&mut self) -> bool {
        self.shift_days(1)
    }
    pub fn previous_day(&mut self) -> bool {
        self.shift_days(-1)
    }
    pub fn next_month(&mut self) -> bool {
        self.shift_months(1)
    }
    pub fn previous_month(&mut self) -> bool {
        self.shift_months(-1)
    }
    pub fn handle_key(&mut self, event: &KeyboardEvent, direction: Direction) -> bool {
        if self.disabled || event.state != KeyState::Pressed {
            return false;
        }
        match event.key {
            Key::Arrow(ArrowKey::Right) if direction == Direction::Ltr => self.next_day(),
            Key::Arrow(ArrowKey::Right) => self.previous_day(),
            Key::Arrow(ArrowKey::Left) if direction == Direction::Ltr => self.previous_day(),
            Key::Arrow(ArrowKey::Left) => self.next_day(),
            Key::Arrow(ArrowKey::Down) => self.shift_days(7),
            Key::Arrow(ArrowKey::Up) => self.shift_days(-7),
            Key::PageDown => self.next_month(),
            Key::PageUp => self.previous_month(),
            Key::Home => self.set_selected(CivilDate {
                day: 1,
                ..self.selected
            }),
            Key::End => self.set_selected(CivilDate {
                day: self.selected.days_in_month(),
                ..self.selected
            }),
            _ => false,
        }
    }
    pub fn semantics(&self) -> Semantics {
        let mut semantics = (1..=self.selected.days_in_month()).fold(
            Semantics::new(SemanticRole::Group)
                .with_name(self.label.clone())
                .with_value(self.selected.to_string())
                .with_action(SemanticAction::Focus)
                .with_action(SemanticAction::Increment)
                .with_action(SemanticAction::Decrement),
            |semantics, day| {
                let mut child = Semantics::new(SemanticRole::Button).with_name(day.to_string());
                child.state.selected = day == self.selected.day;
                semantics.with_virtual_child(child)
            },
        );
        semantics.state.disabled = self.disabled;
        semantics
    }
    pub fn focus_policy(&self) -> FocusPolicy {
        FocusPolicy {
            focusable: true,
            disabled: self.disabled,
            ..FocusPolicy::default()
        }
    }
    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> DataInputDisplayLayout {
        display_layout(
            self.display_text(),
            self.resolved_style(),
            self.direction,
            text_system,
            theme,
            inherited_direction,
            constraints,
        )
    }
    pub fn draws(&self, layout: &DataInputDisplayLayout, origin: LogicalPoint) -> ButtonDraws {
        layout.draws(
            self.display_text(),
            self.resolved_style(),
            self.direction,
            origin,
        )
    }
    fn shift_days(&mut self, delta: i8) -> bool {
        self.set_selected(self.selected.add_days(delta))
    }
    fn shift_months(&mut self, delta: i8) -> bool {
        self.set_selected(self.selected.add_months(delta))
    }
    fn display_text(&self) -> String {
        format!("{}  {}", self.label, self.selected)
    }
    fn resolved_style(&self) -> ComponentStyle {
        let mut style = self.style;
        style.state.disabled = self.disabled;
        style
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DateInput {
    pub calendar: Calendar,
    pub open: bool,
}

impl DateInput {
    pub fn new(label: impl Into<String>, selected: CivilDate) -> Self {
        Self {
            calendar: Calendar::new(label, selected),
            open: false,
        }
    }
    pub fn selected(&self) -> CivilDate {
        self.calendar.selected()
    }
    pub fn activate(&mut self) -> bool {
        if self.calendar.disabled {
            false
        } else {
            self.open = !self.open;
            true
        }
    }
    pub fn handle_key(&mut self, event: &KeyboardEvent, direction: Direction) -> bool {
        if event.state != KeyState::Pressed {
            return false;
        }
        match event.key {
            Key::Space | Key::Enter if !event.repeat => self.activate(),
            Key::Escape if self.open => {
                self.open = false;
                true
            }
            _ if self.open => self.calendar.handle_key(event, direction),
            _ => false,
        }
    }
    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::ComboBox)
            .with_name(self.calendar.label.clone())
            .with_value(self.selected().to_string())
            .with_action(SemanticAction::Focus)
            .with_action(SemanticAction::Activate)
            .with_action(SemanticAction::ShowMenu);
        semantics.state.disabled = self.calendar.disabled;
        semantics.state.expanded = Some(self.open);
        if self.open {
            semantics = semantics.with_virtual_child(self.calendar.semantics());
        }
        semantics
    }
    pub fn focus_policy(&self) -> FocusPolicy {
        self.calendar.focus_policy()
    }
    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> DataInputDisplayLayout {
        display_layout(
            self.display_text(),
            self.calendar.resolved_style(),
            self.calendar.direction,
            text_system,
            theme,
            inherited_direction,
            constraints,
        )
    }
    pub fn draws(&self, layout: &DataInputDisplayLayout, origin: LogicalPoint) -> ButtonDraws {
        layout.draws(
            self.display_text(),
            self.calendar.resolved_style(),
            self.calendar.direction,
            origin,
        )
    }
    fn display_text(&self) -> String {
        format!(
            "{}  {}  {}",
            self.calendar.label,
            self.selected(),
            if self.open { "▲" } else { "▼" }
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fieldset {
    legend: String,
    label: String,
    validation_message: Option<String>,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
}

impl Fieldset {
    pub fn new(legend: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            legend: legend.into(),
            label: label.into(),
            validation_message: None,
            style: ComponentStyle {
                variant: VisualVariant::Outline,
                ..ComponentStyle::default()
            },
            direction: DirectionSetting::Inherit,
        }
    }
    pub fn legend(&self) -> &str {
        &self.legend
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn validation_message(&self) -> Option<&str> {
        self.validation_message.as_deref()
    }
    pub fn set_validation_message(&mut self, message: Option<String>) {
        self.validation_message = message.filter(|message| !message.trim().is_empty());
    }
    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::Group)
            .with_name(self.legend.clone())
            .with_value(self.label.clone());
        semantics.state.invalid = self.validation_message.is_some();
        if let Some(message) = &self.validation_message {
            semantics = semantics
                .with_virtual_child(Semantics::new(SemanticRole::Text).with_name(message.clone()));
        }
        semantics
    }
    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> DataInputDisplayLayout {
        display_layout(
            self.display_text(),
            self.resolved_style(),
            self.direction,
            text_system,
            theme,
            inherited_direction,
            constraints,
        )
    }
    pub fn draws(&self, layout: &DataInputDisplayLayout, origin: LogicalPoint) -> ButtonDraws {
        layout.draws(
            self.display_text(),
            self.resolved_style(),
            self.direction,
            origin,
        )
    }
    fn display_text(&self) -> String {
        match &self.validation_message {
            Some(message) => format!("{}\n{}\n{}", self.legend, self.label, message),
            None => format!("{}\n{}", self.legend, self.label),
        }
    }
    fn resolved_style(&self) -> ComponentStyle {
        let mut style = self.style;
        style.state.error = self.validation_message.is_some();
        style
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RatingError;

impl std::fmt::Display for RatingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("rating maximum must be positive and value must not exceed it")
    }
}

impl std::error::Error for RatingError {}

#[derive(Clone, Debug, PartialEq)]
pub struct Rating {
    label: String,
    maximum: u8,
    value: u8,
    pub disabled: bool,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
}

impl Rating {
    pub fn new(label: impl Into<String>, maximum: u8, value: u8) -> Result<Self, RatingError> {
        if maximum == 0 || value > maximum {
            return Err(RatingError);
        }
        Ok(Self {
            label: label.into(),
            maximum,
            value,
            disabled: false,
            style: ComponentStyle {
                variant: VisualVariant::Ghost,
                ..ComponentStyle::default()
            },
            direction: DirectionSetting::Inherit,
        })
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn value(&self) -> u8 {
        self.value
    }
    pub fn maximum(&self) -> u8 {
        self.maximum
    }
    pub fn set_value(&mut self, value: u8) -> Result<bool, RatingError> {
        if value > self.maximum {
            return Err(RatingError);
        }
        if self.disabled {
            return Ok(false);
        }
        let changed = self.value != value;
        self.value = value;
        Ok(changed)
    }
    pub fn increment(&mut self) -> bool {
        self.set_value(self.value.saturating_add(1).min(self.maximum))
            .unwrap_or(false)
    }
    pub fn decrement(&mut self) -> bool {
        self.set_value(self.value.saturating_sub(1))
            .unwrap_or(false)
    }
    pub fn handle_key(&mut self, event: &KeyboardEvent, direction: Direction) -> bool {
        if event.state != KeyState::Pressed {
            return false;
        }
        match event.key {
            Key::Arrow(ArrowKey::Right) if direction == Direction::Ltr => self.increment(),
            Key::Arrow(ArrowKey::Right) => self.decrement(),
            Key::Arrow(ArrowKey::Left) if direction == Direction::Ltr => self.decrement(),
            Key::Arrow(ArrowKey::Left) => self.increment(),
            Key::Home => self.set_value(0).unwrap_or(false),
            Key::End => self.set_value(self.maximum).unwrap_or(false),
            _ => false,
        }
    }
    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::Slider)
            .with_name(self.label.clone())
            .with_value(format!("{} of {}", self.value, self.maximum))
            .with_numeric_value(
                SemanticNumericValue::new(
                    f64::from(self.value),
                    0.0,
                    f64::from(self.maximum),
                    Some(1.0),
                )
                .unwrap(),
            )
            .with_action(SemanticAction::Focus)
            .with_action(SemanticAction::Increment)
            .with_action(SemanticAction::Decrement)
            .with_action(SemanticAction::SetValue);
        semantics.state.disabled = self.disabled;
        semantics
    }
    pub fn focus_policy(&self) -> FocusPolicy {
        FocusPolicy {
            focusable: true,
            disabled: self.disabled,
            ..FocusPolicy::default()
        }
    }
    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> DataInputDisplayLayout {
        display_layout(
            self.display_text(),
            self.resolved_style(),
            self.direction,
            text_system,
            theme,
            inherited_direction,
            constraints,
        )
    }
    pub fn draws(&self, layout: &DataInputDisplayLayout, origin: LogicalPoint) -> ButtonDraws {
        layout.draws(
            self.display_text(),
            self.resolved_style(),
            self.direction,
            origin,
        )
    }
    fn display_text(&self) -> String {
        format!(
            "{}  {}{}",
            self.label,
            "★".repeat(usize::from(self.value)),
            "☆".repeat(usize::from(self.maximum - self.value))
        )
    }
    fn resolved_style(&self) -> ComponentStyle {
        let mut style = self.style;
        style.state.disabled = self.disabled;
        style
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DataInputDisplayLayout {
    pub button: ButtonLayout,
}

impl DataInputDisplayLayout {
    pub fn draws(
        &self,
        text: String,
        style: ComponentStyle,
        direction: DirectionSetting,
        origin: LogicalPoint,
    ) -> ButtonDraws {
        let mut button = Button::new(text);
        button.style = style;
        button.direction = direction;
        self.button.draws(&button, origin)
    }
}

fn display_layout(
    text: String,
    style: ComponentStyle,
    direction: DirectionSetting,
    text_system: &mut TextSystem,
    theme: &ResolvedTheme,
    inherited_direction: Direction,
    constraints: LogicalConstraints,
) -> DataInputDisplayLayout {
    let mut button = Button::new(text);
    button.style = style;
    button.direction = direction;
    DataInputDisplayLayout {
        button: button.layout(text_system, theme, inherited_direction, constraints),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilterOption {
    pub label: String,
    pub selected: bool,
    pub disabled: bool,
}

impl FilterOption {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            selected: false,
            disabled: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FilterError;

impl std::fmt::Display for FilterError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("filter requires at least one non-empty option")
    }
}

impl std::error::Error for FilterError {}

#[derive(Clone, Debug, PartialEq)]
pub struct Filter {
    label: String,
    options: Vec<FilterOption>,
    active: usize,
    pub disabled: bool,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
}

impl Filter {
    pub fn new(
        label: impl Into<String>,
        options: impl IntoIterator<Item = FilterOption>,
    ) -> Result<Self, FilterError> {
        let options = options.into_iter().collect::<Vec<_>>();
        if options.is_empty() || options.iter().any(|option| option.label.trim().is_empty()) {
            return Err(FilterError);
        }
        Ok(Self {
            label: label.into(),
            options,
            active: 0,
            disabled: false,
            style: ComponentStyle {
                variant: VisualVariant::Outline,
                ..ComponentStyle::default()
            },
            direction: DirectionSetting::Inherit,
        })
    }
    pub fn options(&self) -> &[FilterOption] {
        &self.options
    }
    pub fn active_index(&self) -> usize {
        self.active
    }
    pub fn toggle_active(&mut self) -> bool {
        if self.disabled || self.options[self.active].disabled {
            false
        } else {
            self.options[self.active].selected = !self.options[self.active].selected;
            true
        }
    }
    pub fn next_option(&mut self) -> bool {
        self.move_active(true)
    }
    pub fn previous_option(&mut self) -> bool {
        self.move_active(false)
    }
    pub fn handle_key(&mut self, event: &KeyboardEvent, direction: Direction) -> bool {
        if event.state != KeyState::Pressed {
            return false;
        }
        match event.key {
            Key::Space if !event.repeat => self.toggle_active(),
            Key::Arrow(ArrowKey::Right) if direction == Direction::Ltr => self.next_option(),
            Key::Arrow(ArrowKey::Right) => self.previous_option(),
            Key::Arrow(ArrowKey::Left) if direction == Direction::Ltr => self.previous_option(),
            Key::Arrow(ArrowKey::Left) => self.next_option(),
            Key::Home if !self.disabled => {
                let changed = self.active != 0;
                self.active = 0;
                changed
            }
            Key::End if !self.disabled => {
                let last = self.options.len() - 1;
                let changed = self.active != last;
                self.active = last;
                changed
            }
            _ => false,
        }
    }
    pub fn semantics(&self) -> Semantics {
        let mut semantics = self.options.iter().enumerate().fold(
            Semantics::new(SemanticRole::Group)
                .with_name(self.label.clone())
                .with_action(SemanticAction::Focus)
                .with_action(SemanticAction::Activate)
                .with_action(SemanticAction::Increment)
                .with_action(SemanticAction::Decrement),
            |semantics, (index, option)| {
                let mut child =
                    Semantics::new(SemanticRole::Checkbox).with_name(option.label.clone());
                child.state.checked = Some(option.selected);
                child.state.disabled = option.disabled;
                child.state.selected = index == self.active;
                semantics.with_virtual_child(child)
            },
        );
        semantics.state.disabled = self.disabled;
        semantics
    }
    pub fn focus_policy(&self) -> FocusPolicy {
        FocusPolicy {
            focusable: true,
            disabled: self.disabled,
            ..FocusPolicy::default()
        }
    }
    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> DataInputDisplayLayout {
        display_layout(
            self.display_text(),
            self.resolved_style(),
            self.direction,
            text_system,
            theme,
            inherited_direction,
            constraints,
        )
    }
    pub fn draws(&self, layout: &DataInputDisplayLayout, origin: LogicalPoint) -> ButtonDraws {
        layout.draws(
            self.display_text(),
            self.resolved_style(),
            self.direction,
            origin,
        )
    }
    fn move_active(&mut self, forward: bool) -> bool {
        if self.disabled {
            return false;
        }
        let before = self.active;
        for _ in 0..self.options.len() {
            self.active = if forward {
                (self.active + 1) % self.options.len()
            } else {
                (self.active + self.options.len() - 1) % self.options.len()
            };
            if !self.options[self.active].disabled {
                break;
            }
        }
        before != self.active
    }
    fn display_text(&self) -> String {
        self.options
            .iter()
            .enumerate()
            .map(|(index, option)| {
                let check = if option.selected { "x" } else { " " };
                if index == self.active {
                    format!(">[{check}] {}<", option.label)
                } else {
                    format!("[{check}] {}", option.label)
                }
            })
            .collect::<Vec<_>>()
            .join("  ")
    }
    fn resolved_style(&self) -> ComponentStyle {
        let mut style = self.style;
        style.state.disabled = self.disabled;
        style
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileInputError;

impl std::fmt::Display for FileInputError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("selected file paths must be non-empty and respect single selection")
    }
}

impl std::error::Error for FileInputError {}

#[derive(Clone, Debug, PartialEq)]
pub struct FileInput {
    label: String,
    files: Vec<String>,
    pub multiple: bool,
    pub disabled: bool,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
}

impl FileInput {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            files: Vec::new(),
            multiple: false,
            disabled: false,
            style: ComponentStyle::default(),
            direction: DirectionSetting::Inherit,
        }
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn files(&self) -> &[String] {
        &self.files
    }
    pub fn request_selection(&self) -> bool {
        !self.disabled
    }
    pub fn set_files(
        &mut self,
        files: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<bool, FileInputError> {
        let files = files.into_iter().map(Into::into).collect::<Vec<_>>();
        if files.iter().any(|file| file.trim().is_empty()) || (!self.multiple && files.len() > 1) {
            return Err(FileInputError);
        }
        let changed = self.files != files;
        self.files = files;
        Ok(changed)
    }
    pub fn semantics(&self) -> Semantics {
        let value = if self.files.is_empty() {
            "No file selected".to_owned()
        } else {
            self.files.join(", ")
        };
        let mut semantics = Semantics::new(SemanticRole::Button)
            .with_name(self.label.clone())
            .with_value(value)
            .with_action(SemanticAction::Focus)
            .with_action(SemanticAction::Activate);
        semantics.state.disabled = self.disabled;
        semantics
    }
    pub fn focus_policy(&self) -> FocusPolicy {
        FocusPolicy {
            focusable: true,
            disabled: self.disabled,
            ..FocusPolicy::default()
        }
    }
    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> DataInputDisplayLayout {
        display_layout(
            self.display_text(),
            self.resolved_style(),
            self.direction,
            text_system,
            theme,
            inherited_direction,
            constraints,
        )
    }
    pub fn draws(&self, layout: &DataInputDisplayLayout, origin: LogicalPoint) -> ButtonDraws {
        layout.draws(
            self.display_text(),
            self.resolved_style(),
            self.direction,
            origin,
        )
    }
    fn display_text(&self) -> String {
        if self.files.is_empty() {
            format!("{}  —  No file selected", self.label)
        } else {
            format!("{}  —  {}", self.label, self.files.join(", "))
        }
    }
    fn resolved_style(&self) -> ComponentStyle {
        let mut style = self.style;
        style.state.disabled = self.disabled;
        style
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Calendar, CivilDate, DateInput, Fieldset, FileInput, Filter, FilterOption, Rating,
    };
    use crate::{ArrowKey, Direction, Key, KeyboardEvent, SemanticRole};

    #[test]
    fn fieldset_exposes_validation_and_rating_mirrors_keys() {
        let mut fieldset = Fieldset::new("Account", "Email");
        fieldset.set_validation_message(Some("Email is required".into()));
        assert!(fieldset.semantics().state.invalid);
        assert_eq!(fieldset.semantics().virtual_children().len(), 1);
        let mut rating = Rating::new("Quality", 5, 2).unwrap();
        assert_eq!(rating.semantics().role, SemanticRole::Slider);
        assert!(rating.handle_key(
            &KeyboardEvent::pressed(Key::Arrow(ArrowKey::Right)),
            Direction::Ltr
        ));
        assert_eq!(rating.value(), 3);
        assert!(rating.handle_key(
            &KeyboardEvent::pressed(Key::Arrow(ArrowKey::Right)),
            Direction::Rtl
        ));
        assert_eq!(rating.value(), 2);
        assert!(Rating::new("Invalid", 0, 0).is_err());
    }

    #[test]
    fn filter_and_file_input_validate_selection_models() {
        let mut filter = Filter::new(
            "Tags",
            [FilterOption::new("Rust"), FilterOption::new("GUI")],
        )
        .unwrap();
        assert!(filter.toggle_active());
        assert_eq!(
            filter.semantics().virtual_children()[0].state.checked,
            Some(true)
        );
        assert!(filter.handle_key(
            &KeyboardEvent::pressed(Key::Arrow(ArrowKey::Right)),
            Direction::Ltr
        ));
        assert_eq!(filter.active_index(), 1);
        let mut input = FileInput::new("Attachment");
        assert!(input.set_files(["report.pdf"]).unwrap());
        assert_eq!(input.files(), &["report.pdf"]);
        assert!(input.set_files(["one", "two"]).is_err());
    }

    #[test]
    fn calendar_validates_dates_and_navigates_civil_boundaries() {
        assert!(CivilDate::new(2025, 2, 29).is_err());
        let leap_day = CivilDate::new(2024, 2, 29).unwrap();
        let mut calendar = Calendar::new("Appointment", leap_day);
        assert!(calendar.next_day());
        assert_eq!(calendar.selected(), CivilDate::new(2024, 3, 1).unwrap());
        assert!(calendar.handle_key(
            &KeyboardEvent::pressed(Key::Arrow(ArrowKey::Right)),
            Direction::Rtl
        ));
        assert_eq!(calendar.selected(), leap_day);
        assert_eq!(calendar.semantics().virtual_children().len(), 29);
    }

    #[test]
    fn date_input_exposes_popup_state_and_delegates_navigation() {
        let mut input = DateInput::new("Birthday", CivilDate::new(2026, 8, 16).unwrap());
        assert!(input.handle_key(&KeyboardEvent::pressed(Key::Enter), Direction::Ltr));
        assert_eq!(input.semantics().state.expanded, Some(true));
        assert!(input.handle_key(&KeyboardEvent::pressed(Key::PageDown), Direction::Ltr));
        assert_eq!(input.selected(), CivilDate::new(2026, 9, 16).unwrap());
        assert!(input.handle_key(&KeyboardEvent::pressed(Key::Escape), Direction::Ltr));
        assert!(!input.open);
    }
}
