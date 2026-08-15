// representative_form.rs

use mio_gui::{
    Button, Checkbox, Column, Direction, Radio, SearchInput, Select, SelectOption, Slider, Switch,
    TextArea, TextInput, Widget, WidgetTree, run_widget_tree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut column = Column::default();
    column.layout.gap = 12.0;
    let mut tree = WidgetTree::new(Widget::from(column));
    let root = tree.root();
    let mut name = TextInput::new("Name");
    name.set_placeholder("Type your name");
    tree.append(root, Widget::from(name))?;
    tree.append(root, Widget::from(SearchInput::new("Search")))?;
    let mut notes = TextArea::new("Notes");
    notes.input.set_placeholder("Add notes");
    notes.set_minimum_lines(2);
    tree.append(root, Widget::from(notes))?;
    tree.append(
        root,
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
        root,
        Widget::from(Radio::new("Standard delivery").with_group("delivery", "standard")),
    )?;
    tree.append(
        root,
        Widget::from(Radio::new("Express delivery").with_group("delivery", "express")),
    )?;
    tree.select_radio(standard);
    tree.append(root, Widget::from(Checkbox::new("Receive updates")))?;
    tree.append(root, Widget::from(Switch::new("Enable alerts")))?;
    tree.append(
        root,
        Widget::from(Slider::new("Amount", 0.0..=100.0, 50.0)?),
    )?;
    tree.append(root, Widget::from(Button::new("Submit")))?;

    let direction = if std::env::var("MIO_GUI_DIRECTION").is_ok_and(|value| value == "ltr") {
        Direction::Ltr
    } else {
        Direction::Rtl
    };
    run_widget_tree(tree, direction);
    Ok(())
}
