use mio_gui::{
    Alert, Badge, Button, Calendar, Checkbox, CivilDate, Column, DateInput, Direction, Fieldset,
    Filter, FilterOption, Kbd, Loading, LogicalSize, Progress, RadialProgress, Radio, Rating, Row,
    SearchInput, Select, SelectOption, Skeleton, Slider, Swap, Switch, Text, TextArea, TextInput,
    ThemeSwitcher, Toast, Widget, WidgetTree, run_widget_tree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut root_layout = Column::default();
    root_layout.layout.gap = 12.0;
    let mut tree = WidgetTree::new(Widget::from(root_layout));
    let root = tree.root();
    tree.append(root, Widget::from(Text::new("Mio-GUI widget gallery")))?;

    let mut row_layout = Row::default();
    row_layout.layout.gap = 28.0;
    let row = tree.append(root, Widget::from(row_layout))?;
    let mut column_layout = Column::default();
    column_layout.layout.gap = 9.0;
    let inputs = tree.append(row, Widget::from(column_layout))?;
    let feedback = tree.append(row, Widget::from(column_layout))?;

    tree.append(inputs, Widget::from(Text::new("Inputs and actions")))?;
    let mut name = TextInput::new("Name");
    name.set_placeholder("Type your name");
    tree.append(inputs, Widget::from(name))?;
    tree.append(inputs, Widget::from(SearchInput::new("Search")))?;
    let mut notes = TextArea::new("Notes");
    notes.input.set_placeholder("Add notes");
    notes.set_minimum_lines(2);
    tree.append(inputs, Widget::from(notes))?;
    tree.append(
        inputs,
        Widget::from(Select::new(
            "Country",
            vec![
                SelectOption::new("Iran", "ir"),
                SelectOption::new("Japan", "jp"),
                SelectOption::new("Germany", "de"),
            ],
        )?),
    )?;
    let standard = tree.append(
        inputs,
        Widget::from(Radio::new("Standard").with_group("delivery", "standard")),
    )?;
    tree.append(
        inputs,
        Widget::from(Radio::new("Express").with_group("delivery", "express")),
    )?;
    tree.select_radio(standard);
    tree.append(inputs, Widget::from(Checkbox::new("Receive updates")))?;
    tree.append(inputs, Widget::from(Switch::new("Enable alerts")))?;
    tree.append(
        inputs,
        Widget::from(Slider::new("Amount", 0.0..=100.0, 55.0)?),
    )?;
    tree.append(inputs, Widget::from(Button::new("Submit")))?;

    tree.append(feedback, Widget::from(Text::new("States and feedback")))?;
    tree.append(feedback, Widget::from(Alert::new("Changes saved")))?;
    tree.append(feedback, Widget::from(Progress::new("Upload", 0.65)?))?;
    tree.append(
        feedback,
        Widget::from(RadialProgress::new("Download", 0.75)?),
    )?;
    let mut loading = Loading::new("Loading profile");
    loading.set_phase(0.4);
    tree.append(feedback, Widget::from(loading))?;
    tree.append(
        feedback,
        Widget::from(Skeleton::new(LogicalSize::new(180.0, 24.0))),
    )?;
    tree.append(feedback, Widget::from(Toast::new("Download complete")))?;
    tree.append(feedback, Widget::from(Badge::new("New")))?;
    tree.append(feedback, Widget::from(Kbd::new("Ctrl K")))?;
    tree.append(feedback, Widget::from(Rating::new("Quality", 5, 3)?))?;
    let mut filter = Filter::new(
        "Topics",
        [FilterOption::new("Rust"), FilterOption::new("GUI")],
    )?;
    filter.toggle_active();
    tree.append(feedback, Widget::from(filter))?;
    tree.append(
        feedback,
        Widget::from(Calendar::new("Calendar", CivilDate::new(2026, 8, 16)?)),
    )?;
    tree.append(
        feedback,
        Widget::from(DateInput::new("Appointment", CivilDate::new(2026, 8, 16)?)),
    )?;
    tree.append(feedback, Widget::from(Fieldset::new("Account", "Email")))?;
    tree.append(
        feedback,
        Widget::from(Swap::new("Playback", "Play", "Pause")),
    )?;
    tree.append(feedback, Widget::from(ThemeSwitcher::new("Theme")))?;

    let direction = if std::env::var("MIO_GUI_DIRECTION").is_ok_and(|value| value == "ltr") {
        Direction::Ltr
    } else {
        Direction::Rtl
    };
    run_widget_tree(tree, direction);
    Ok(())
}
