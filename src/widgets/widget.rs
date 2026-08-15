// widget.rs

use std::collections::HashMap;

use crate::{
    Direction, FrameNode, FrameSnapshot, ImageDraw, LayoutChild, LogicalConstraints, LogicalPoint,
    LogicalRect, LogicalSize, Overflow, RectDraw, ResolvedTheme, SemanticSnapshot, Semantics,
    StackChild, TextDraw, TextSystem, WidgetGeometry, WidgetId, WidgetTree,
};

use super::{
    Button, ButtonLayout, Checkbox, CheckboxLayout, Column, Container, ContextMenu,
    ContextMenuLayout, Divider, Drawer, DrawerLayout, Dropdown, DropdownLayout, Icon, IconButton,
    IconButtonLayout, IconLayout, Image, ImageLayout, Menu, MenuLayout, Modal, ModalLayout,
    Popover, PopoverLayout, Radio, RadioLayout, Row, ScrollLayout, ScrollView, SearchInput,
    SearchInputLayout, Select, SelectLayout, Slider, SliderLayout, Spacer, Stack, StackLayout,
    Surface, Switch, SwitchLayout, Text, TextArea, TextInput, TextInputLayout, TextLayout, Tooltip,
    TooltipLayout,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Widget {
    Text(Text),
    Image(Image),
    Icon(Icon),
    Spacer(Spacer),
    Divider(Divider),
    Surface(Surface),
    Container(Container),
    Row(Row),
    Column(Column),
    Stack(Stack),
    ScrollView(ScrollView),
    Button(Button),
    IconButton(IconButton),
    Checkbox(Checkbox),
    Radio(Radio),
    Switch(Switch),
    Slider(Slider),
    TextInput(TextInput),
    TextArea(TextArea),
    SearchInput(SearchInput),
    Select(Select),
    Menu(Menu),
    Dropdown(Dropdown),
    ContextMenu(ContextMenu),
    Tooltip(Tooltip),
    Popover(Popover),
    Modal(Modal),
    Drawer(Drawer),
}

impl WidgetTree<Widget> {
    pub fn radio_tab_stop(&self, group: &str) -> Option<WidgetId> {
        let radios = self.depth_first(self.root()).filter_map(|id| {
            let Widget::Radio(radio) = &self.get(id).unwrap().state else {
                return None;
            };
            (radio.group() == Some(group) && !radio.disabled).then_some((id, radio.selected))
        });
        let radios = radios.collect::<Vec<_>>();
        radios
            .iter()
            .find_map(|(id, selected)| selected.then_some(*id))
            .or_else(|| radios.first().map(|(id, _)| *id))
    }

    pub fn select_radio(&mut self, target: WidgetId) -> bool {
        let Some(Widget::Radio(target_radio)) = self.get(target).map(|node| &node.state) else {
            return false;
        };
        if target_radio.disabled {
            return false;
        }
        let group = target_radio.group().map(str::to_owned);
        if let Some(group) = group.as_deref() {
            let peers = self
                .depth_first(self.root())
                .filter(|id| {
                    matches!(
                        &self.get(*id).unwrap().state,
                        Widget::Radio(radio) if radio.group() == Some(group)
                    )
                })
                .collect::<Vec<_>>();
            for peer in peers {
                let Widget::Radio(radio) = &mut self.get_mut(peer).unwrap().state else {
                    unreachable!()
                };
                radio.selected = peer == target;
            }
            true
        } else {
            let Widget::Radio(radio) = &mut self.get_mut(target).unwrap().state else {
                unreachable!()
            };
            radio.activate()
        }
    }

    pub fn adjacent_radio(&self, current: WidgetId, forward: bool) -> Option<WidgetId> {
        let Widget::Radio(current_radio) = &self.get(current)?.state else {
            return None;
        };
        let group = current_radio.group()?;
        let radios = self
            .depth_first(self.root())
            .filter(|id| {
                matches!(
                    &self.get(*id).unwrap().state,
                    Widget::Radio(radio) if radio.group() == Some(group) && !radio.disabled
                )
            })
            .collect::<Vec<_>>();
        let index = radios.iter().position(|id| *id == current)?;
        Some(if forward {
            radios[(index + 1) % radios.len()]
        } else {
            radios[(index + radios.len() - 1) % radios.len()]
        })
    }
}

impl From<Text> for Widget {
    fn from(text: Text) -> Self {
        Self::Text(text)
    }
}

impl From<Image> for Widget {
    fn from(image: Image) -> Self {
        Self::Image(image)
    }
}

impl From<Icon> for Widget {
    fn from(icon: Icon) -> Self {
        Self::Icon(icon)
    }
}
impl From<Spacer> for Widget {
    fn from(spacer: Spacer) -> Self {
        Self::Spacer(spacer)
    }
}
impl From<Divider> for Widget {
    fn from(divider: Divider) -> Self {
        Self::Divider(divider)
    }
}
impl From<Surface> for Widget {
    fn from(surface: Surface) -> Self {
        Self::Surface(surface)
    }
}
impl From<Container> for Widget {
    fn from(value: Container) -> Self {
        Self::Container(value)
    }
}
impl From<Row> for Widget {
    fn from(value: Row) -> Self {
        Self::Row(value)
    }
}
impl From<Column> for Widget {
    fn from(value: Column) -> Self {
        Self::Column(value)
    }
}
impl From<Stack> for Widget {
    fn from(value: Stack) -> Self {
        Self::Stack(value)
    }
}
impl From<ScrollView> for Widget {
    fn from(value: ScrollView) -> Self {
        Self::ScrollView(value)
    }
}
impl From<Button> for Widget {
    fn from(value: Button) -> Self {
        Self::Button(value)
    }
}
impl From<IconButton> for Widget {
    fn from(value: IconButton) -> Self {
        Self::IconButton(value)
    }
}
impl From<Checkbox> for Widget {
    fn from(value: Checkbox) -> Self {
        Self::Checkbox(value)
    }
}
impl From<Radio> for Widget {
    fn from(value: Radio) -> Self {
        Self::Radio(value)
    }
}
impl From<Switch> for Widget {
    fn from(value: Switch) -> Self {
        Self::Switch(value)
    }
}
impl From<Slider> for Widget {
    fn from(value: Slider) -> Self {
        Self::Slider(value)
    }
}
impl From<TextInput> for Widget {
    fn from(value: TextInput) -> Self {
        Self::TextInput(value)
    }
}
impl From<TextArea> for Widget {
    fn from(value: TextArea) -> Self {
        Self::TextArea(value)
    }
}
impl From<SearchInput> for Widget {
    fn from(value: SearchInput) -> Self {
        Self::SearchInput(value)
    }
}
impl From<Select> for Widget {
    fn from(value: Select) -> Self {
        Self::Select(value)
    }
}
impl From<Menu> for Widget {
    fn from(value: Menu) -> Self {
        Self::Menu(value)
    }
}
impl From<Dropdown> for Widget {
    fn from(value: Dropdown) -> Self {
        Self::Dropdown(value)
    }
}
impl From<ContextMenu> for Widget {
    fn from(value: ContextMenu) -> Self {
        Self::ContextMenu(value)
    }
}
impl From<Tooltip> for Widget {
    fn from(value: Tooltip) -> Self {
        Self::Tooltip(value)
    }
}
impl From<Popover> for Widget {
    fn from(value: Popover) -> Self {
        Self::Popover(value)
    }
}
impl From<Modal> for Widget {
    fn from(value: Modal) -> Self {
        Self::Modal(value)
    }
}
impl From<Drawer> for Widget {
    fn from(value: Drawer) -> Self {
        Self::Drawer(value)
    }
}

impl Widget {
    pub fn semantics(&self) -> Semantics {
        match self {
            Self::Text(text) => text.semantics(),
            Self::Image(image) => image.semantics(),
            Self::Icon(icon) => icon.semantics(),
            Self::Button(button) => button.semantics(),
            Self::IconButton(button) => button.semantics(),
            Self::Checkbox(checkbox) => checkbox.semantics(),
            Self::Radio(radio) => radio.semantics(),
            Self::Switch(switch) => switch.semantics(),
            Self::Slider(slider) => slider.semantics(),
            Self::TextInput(input) => input.semantics(),
            Self::TextArea(area) => area.semantics(),
            Self::SearchInput(search) => search.semantics(),
            Self::Select(select) => select.semantics(),
            Self::Menu(menu) => menu.semantics(),
            Self::Dropdown(dropdown) => dropdown.semantics(),
            Self::ContextMenu(menu) => menu.semantics(),
            Self::Tooltip(tooltip) => tooltip.semantics(),
            Self::Popover(popover) => popover.semantics(),
            Self::Modal(modal) => modal.semantics(),
            Self::Drawer(drawer) => drawer.semantics(),
            Self::Spacer(_)
            | Self::Divider(_)
            | Self::Surface(_)
            | Self::Container(_)
            | Self::Row(_)
            | Self::Column(_)
            | Self::Stack(_)
            | Self::ScrollView(_) => Semantics::default(),
        }
    }

    pub fn focus_policy(&self) -> crate::FocusPolicy {
        match self {
            Self::Button(button) => button.focus_policy(),
            Self::IconButton(button) => button.focus_policy(),
            Self::Checkbox(checkbox) => checkbox.focus_policy(),
            Self::Radio(radio) => radio.focus_policy(),
            Self::Switch(switch) => switch.focus_policy(),
            Self::Slider(slider) => slider.focus_policy(),
            Self::TextInput(input) => input.focus_policy(),
            Self::TextArea(area) => area.focus_policy(),
            Self::SearchInput(search) => search.focus_policy(),
            Self::Select(select) => select.focus_policy(),
            Self::Menu(menu) => menu.focus_policy(),
            Self::Dropdown(dropdown) => dropdown.focus_policy(),
            Self::ContextMenu(menu) => menu.focus_policy(),
            Self::Popover(popover) => popover.focus_policy(),
            Self::Modal(modal) => modal.focus_policy(),
            Self::Drawer(drawer) => drawer.focus_policy(),
            _ => crate::FocusPolicy::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WidgetPlacement {
    pub origin: LogicalPoint,
    pub constraints: LogicalConstraints,
    pub inherited_direction: Direction,
}

impl WidgetPlacement {
    pub fn new(
        origin: LogicalPoint,
        constraints: LogicalConstraints,
        inherited_direction: Direction,
    ) -> Self {
        Self {
            origin,
            constraints,
            inherited_direction,
        }
    }
}

#[derive(Clone, Debug)]
enum WidgetLayout {
    Text(TextLayout),
    Image(ImageLayout),
    Icon(IconLayout),
    Spacer(crate::LogicalSize),
    Divider(crate::LogicalSize),
    Surface(crate::LogicalSize),
    Container(crate::LogicalSize),
    Row(LogicalSize),
    Column(LogicalSize),
    Stack(StackLayout),
    ScrollView(ScrollLayout),
    Button(ButtonLayout),
    IconButton(IconButtonLayout),
    Checkbox(CheckboxLayout),
    Radio(RadioLayout),
    Switch(SwitchLayout),
    Slider(SliderLayout),
    TextInput(TextInputLayout),
    TextArea(TextInputLayout),
    SearchInput(SearchInputLayout),
    Select(SelectLayout),
    Menu(MenuLayout),
    Dropdown(DropdownLayout),
    ContextMenu(ContextMenuLayout),
    Tooltip(TooltipLayout),
    Popover(PopoverLayout),
    Modal(ModalLayout),
    Drawer(DrawerLayout),
}

#[derive(Clone, Debug)]
pub struct WidgetFrame {
    pub geometry: FrameSnapshot,
    pub semantics: SemanticSnapshot,
    pub rectangles: Vec<RectDraw>,
    pub text: Vec<TextDraw>,
    pub images: Vec<ImageDraw>,
}

impl WidgetFrame {
    pub fn build(
        tree: &WidgetTree<Widget>,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        mut place: impl FnMut(WidgetId, Option<FrameNode>) -> WidgetPlacement,
    ) -> Self {
        let mut layouts = HashMap::with_capacity(tree.len());
        let geometry = FrameSnapshot::build(tree, |id, parent| {
            let placement = place(id, parent);
            let layout = match &tree.get(id).unwrap().state {
                Widget::Text(text) => WidgetLayout::Text(text.layout(
                    text_system,
                    placement.inherited_direction,
                    placement.constraints,
                )),
                Widget::Image(image) => WidgetLayout::Image(
                    image.layout(placement.inherited_direction, placement.constraints),
                ),
                Widget::Icon(icon) => WidgetLayout::Icon(
                    icon.layout(placement.inherited_direction, placement.constraints),
                ),
                Widget::Spacer(spacer) => {
                    WidgetLayout::Spacer(spacer.layout(placement.constraints))
                }
                Widget::Divider(divider) => {
                    WidgetLayout::Divider(divider.layout(placement.constraints))
                }
                Widget::Surface(surface) => {
                    WidgetLayout::Surface(surface.layout(placement.constraints))
                }
                Widget::Container(container) => {
                    WidgetLayout::Container(container.layout(placement.constraints))
                }
                Widget::Row(row) => WidgetLayout::Row(
                    row.layout(placement.inherited_direction, &[], placement.constraints)
                        .size,
                ),
                Widget::Column(column) => WidgetLayout::Column(
                    column
                        .layout(placement.inherited_direction, &[], placement.constraints)
                        .size,
                ),
                Widget::Stack(stack) => WidgetLayout::Stack(stack.layout(
                    placement.inherited_direction,
                    &[],
                    placement.constraints,
                )),
                Widget::ScrollView(scroll) => WidgetLayout::ScrollView(scroll.layout(
                    placement.inherited_direction,
                    LogicalSize::default(),
                    placement.constraints,
                )),
                Widget::Button(button) => WidgetLayout::Button(button.layout(
                    text_system,
                    theme,
                    placement.inherited_direction,
                    placement.constraints,
                )),
                Widget::IconButton(button) => WidgetLayout::IconButton(button.layout(
                    theme,
                    placement.inherited_direction,
                    placement.constraints,
                )),
                Widget::Checkbox(checkbox) => WidgetLayout::Checkbox(checkbox.layout(
                    text_system,
                    theme,
                    placement.inherited_direction,
                    placement.constraints,
                )),
                Widget::Radio(radio) => WidgetLayout::Radio(radio.layout(
                    text_system,
                    theme,
                    placement.inherited_direction,
                    placement.constraints,
                )),
                Widget::Switch(switch) => WidgetLayout::Switch(switch.layout(
                    text_system,
                    theme,
                    placement.inherited_direction,
                    placement.constraints,
                )),
                Widget::Slider(slider) => WidgetLayout::Slider(slider.layout(
                    theme,
                    placement.inherited_direction,
                    placement.constraints,
                )),
                Widget::TextInput(input) => WidgetLayout::TextInput(input.layout(
                    text_system,
                    theme,
                    placement.inherited_direction,
                    placement.constraints,
                )),
                Widget::TextArea(area) => WidgetLayout::TextArea(area.layout(
                    text_system,
                    theme,
                    placement.inherited_direction,
                    placement.constraints,
                )),
                Widget::SearchInput(search) => WidgetLayout::SearchInput(search.layout(
                    text_system,
                    theme,
                    placement.inherited_direction,
                    placement.constraints,
                )),
                Widget::Select(select) => WidgetLayout::Select(select.layout(
                    text_system,
                    theme,
                    placement.inherited_direction,
                    placement.constraints,
                )),
                Widget::Menu(menu) => WidgetLayout::Menu(menu.layout(
                    text_system,
                    theme,
                    placement.inherited_direction,
                    placement.constraints,
                )),
                Widget::Dropdown(dropdown) => WidgetLayout::Dropdown(dropdown.layout(
                    text_system,
                    theme,
                    placement.inherited_direction,
                    placement.constraints,
                )),
                Widget::ContextMenu(menu) => WidgetLayout::ContextMenu(menu.layout(
                    text_system,
                    theme,
                    placement.inherited_direction,
                    placement.constraints,
                )),
                Widget::Tooltip(tooltip) => WidgetLayout::Tooltip(tooltip.layout(
                    text_system,
                    theme,
                    placement.inherited_direction,
                    LogicalRect::new(placement.origin, LogicalSize::default()),
                    placement.constraints.max,
                )),
                Widget::Popover(popover) => WidgetLayout::Popover(popover.layout(
                    theme,
                    placement.inherited_direction,
                    LogicalRect::new(placement.origin, LogicalSize::default()),
                    placement.constraints.max,
                )),
                Widget::Modal(modal) => {
                    WidgetLayout::Modal(modal.layout(theme, placement.constraints.max))
                }
                Widget::Drawer(drawer) => WidgetLayout::Drawer(drawer.layout(
                    theme,
                    placement.inherited_direction,
                    placement.constraints.max,
                )),
            };
            let size = match &layout {
                WidgetLayout::Text(layout) => layout.size,
                WidgetLayout::Image(layout) => layout.size,
                WidgetLayout::Icon(layout) => layout.size(),
                WidgetLayout::Spacer(size)
                | WidgetLayout::Divider(size)
                | WidgetLayout::Surface(size)
                | WidgetLayout::Container(size)
                | WidgetLayout::Row(size)
                | WidgetLayout::Column(size) => *size,
                WidgetLayout::Stack(layout) => layout.size,
                WidgetLayout::ScrollView(layout) => layout.viewport,
                WidgetLayout::Button(layout) => layout.size,
                WidgetLayout::IconButton(layout) => layout.size,
                WidgetLayout::Checkbox(layout) => layout.size,
                WidgetLayout::Radio(layout) => layout.size,
                WidgetLayout::Switch(layout) => layout.size,
                WidgetLayout::Slider(layout) => layout.size,
                WidgetLayout::TextInput(layout) => layout.size,
                WidgetLayout::TextArea(layout) => layout.size,
                WidgetLayout::SearchInput(layout) => layout.size,
                WidgetLayout::Select(layout) => layout.size,
                WidgetLayout::Menu(layout) => layout.size,
                WidgetLayout::Dropdown(layout) => layout.size,
                WidgetLayout::ContextMenu(layout) => layout.size,
                WidgetLayout::Tooltip(layout) => layout.size,
                WidgetLayout::Popover(layout) => layout.size,
                WidgetLayout::Modal(layout) => layout.size,
                WidgetLayout::Drawer(layout) => layout.size,
            };
            layouts.insert(id, (placement.origin, layout));
            let geometry_origin = match &layouts[&id].1 {
                WidgetLayout::Tooltip(layout) => layout.origin,
                _ => placement.origin,
            };
            let mut geometry = WidgetGeometry::new(LogicalRect::new(geometry_origin, size));
            if matches!(&layouts[&id].1, WidgetLayout::ScrollView(_)) {
                geometry.overflow = Overflow::Clip;
            }
            geometry
        });
        let mut semantics = SemanticSnapshot::build(tree, |_, widget| widget.semantics());
        for (id, (origin, layout)) in &layouts {
            let menu = match layout {
                WidgetLayout::Menu(menu) => Some((*origin, menu)),
                WidgetLayout::Select(select) => Some((
                    LogicalPoint::new(
                        origin.x + select.menu_origin.x,
                        origin.y + select.menu_origin.y,
                    ),
                    &select.menu,
                )),
                WidgetLayout::Dropdown(dropdown) => Some((
                    LogicalPoint::new(
                        origin.x + dropdown.menu_origin.x,
                        origin.y + dropdown.menu_origin.y,
                    ),
                    &dropdown.menu,
                )),
                WidgetLayout::ContextMenu(context) => Some((
                    LogicalPoint::new(
                        origin.x + context.menu_origin.x,
                        origin.y + context.menu_origin.y,
                    ),
                    &context.menu,
                )),
                _ => None,
            };
            if let Some((menu_origin, menu)) = menu {
                for index in 0..menu.labels.len() {
                    if let Some(bounds) = menu.item_bounds(menu_origin, index) {
                        semantics.set_virtual_child_bounds(*id, index, bounds);
                    }
                }
            }
        }
        let mut rectangles = Vec::new();
        let mut text = Vec::new();
        let mut images = Vec::new();
        geometry.paint(|node| {
            let (origin, layout) = layouts.get(&node.id).unwrap();
            match layout {
                WidgetLayout::Text(layout) => {
                    let widget = &tree.get(node.id).unwrap().state;
                    let Widget::Text(widget) = widget else {
                        unreachable!();
                    };
                    text.extend(layout.draws(
                        widget.content(),
                        *origin,
                        theme.colors.resolve(layout.color).to_array(),
                    ));
                }
                WidgetLayout::Image(layout) => {
                    let widget = &tree.get(node.id).unwrap().state;
                    let Widget::Image(widget) = widget else {
                        unreachable!();
                    };
                    images.push(layout.draw(widget.source.clone(), *origin));
                }
                WidgetLayout::Icon(layout) => {
                    let widget = &tree.get(node.id).unwrap().state;
                    let Widget::Icon(widget) = widget else {
                        unreachable!();
                    };
                    images.push(layout.draw(
                        widget.source.clone(),
                        *origin,
                        theme.colors.resolve(widget.color).to_array(),
                    ));
                }
                WidgetLayout::Spacer(_) => {}
                WidgetLayout::Divider(size) => {
                    let widget = &tree.get(node.id).unwrap().state;
                    let Widget::Divider(widget) = widget else {
                        unreachable!()
                    };
                    rectangles.push(widget.draw(
                        *origin,
                        *size,
                        theme.colors.resolve(widget.color).to_array(),
                    ));
                }
                WidgetLayout::Surface(size) => {
                    let Widget::Surface(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    rectangles.push(widget.draw(*origin, *size, theme));
                }
                WidgetLayout::Container(_) => {}
                WidgetLayout::Row(_) | WidgetLayout::Column(_) => {}
                WidgetLayout::Stack(_) | WidgetLayout::ScrollView(_) => {}
                WidgetLayout::Button(layout) => {
                    let Widget::Button(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    let draws = layout.draws(widget, *origin);
                    rectangles.push(draws.background);
                    text.extend(draws.text);
                    images.extend(draws.icon);
                }
                WidgetLayout::IconButton(layout) => {
                    let Widget::IconButton(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    let draws = layout.draws(widget, *origin);
                    rectangles.push(draws.background);
                    images.extend(draws.icon);
                }
                WidgetLayout::Checkbox(layout) => {
                    let Widget::Checkbox(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    let draws = layout.draws(widget, *origin, theme);
                    rectangles.extend(draws.indicator);
                    text.extend(draws.label);
                }
                WidgetLayout::Radio(layout) => {
                    let Widget::Radio(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    let draws = layout.draws(widget, *origin, theme);
                    rectangles.extend(draws.indicator);
                    text.extend(draws.label);
                }
                WidgetLayout::Switch(layout) => {
                    let Widget::Switch(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    let draws = layout.draws(widget, *origin, theme);
                    rectangles.extend(draws.control);
                    text.extend(draws.label);
                }
                WidgetLayout::Slider(layout) => {
                    let Widget::Slider(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    rectangles.extend(layout.draws(widget, *origin, theme));
                }
                WidgetLayout::TextInput(layout) => {
                    let Widget::TextInput(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    let draws = layout.draws(widget, *origin, theme);
                    rectangles.push(draws.background);
                    rectangles.extend(draws.editing);
                    text.extend(draws.text);
                }
                WidgetLayout::TextArea(layout) => {
                    let Widget::TextArea(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    let draws = widget.draws(layout, *origin, theme);
                    rectangles.push(draws.background);
                    rectangles.extend(draws.editing);
                    text.extend(draws.text);
                }
                WidgetLayout::SearchInput(layout) => {
                    let Widget::SearchInput(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    let draws = layout.draws(widget, *origin, theme);
                    rectangles.push(draws.input.background);
                    rectangles.extend(draws.input.editing);
                    text.extend(draws.input.text);
                    images.push(draws.icon);
                }
                WidgetLayout::Select(layout) => {
                    let Widget::Select(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    let draws = widget.draws(layout, *origin, theme);
                    rectangles.extend(draws.rectangles);
                    text.extend(draws.text);
                    images.extend(draws.images);
                }
                WidgetLayout::Menu(layout) => {
                    let Widget::Menu(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    let draws = layout.draws(widget, *origin, theme);
                    rectangles.extend(draws.rectangles);
                    text.extend(draws.text);
                }
                WidgetLayout::Dropdown(layout) => {
                    let Widget::Dropdown(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    let draws = layout.draws(widget, *origin, theme);
                    rectangles.extend(draws.rectangles);
                    text.extend(draws.text);
                    images.extend(draws.images);
                }
                WidgetLayout::ContextMenu(layout) => {
                    let Widget::ContextMenu(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    let draws = layout.draws(widget, *origin, theme);
                    rectangles.extend(draws.rectangles);
                    text.extend(draws.text);
                }
                WidgetLayout::Tooltip(layout) => {
                    let Widget::Tooltip(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    let draws = layout.draws(widget, theme);
                    rectangles.extend(draws.background);
                    text.extend(draws.text);
                }
                WidgetLayout::Popover(layout) => {
                    let Widget::Popover(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    rectangles.extend(layout.draw(widget, theme));
                }
                WidgetLayout::Modal(layout) => {
                    let Widget::Modal(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    rectangles.extend(layout.draws(widget, theme).into_iter().map(|mut draw| {
                        draw.position[0] += origin.x;
                        draw.position[1] += origin.y;
                        draw
                    }));
                }
                WidgetLayout::Drawer(layout) => {
                    let Widget::Drawer(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    rectangles.extend(layout.draws(widget, theme).into_iter().map(|mut draw| {
                        draw.position[0] += origin.x;
                        draw.position[1] += origin.y;
                        draw
                    }));
                }
            }
        });
        rectangles.shrink_to_fit();
        images.shrink_to_fit();

        Self {
            geometry,
            semantics,
            rectangles,
            text,
            images,
        }
    }

    pub fn build_composed(
        tree: &WidgetTree<Widget>,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        root: WidgetPlacement,
    ) -> Self {
        let direction = root.inherited_direction;
        let mut measured = HashMap::with_capacity(tree.len());
        let ids = tree.depth_first(tree.root()).collect::<Vec<_>>();
        for id in ids.iter().rev().copied() {
            let node = tree.get(id).unwrap();
            let children = node
                .children()
                .iter()
                .map(|child| LayoutChild::new(measured[child]))
                .collect::<Vec<_>>();
            let constraints = if id == tree.root()
                || matches!(
                    &node.state,
                    Widget::Popover(_) | Widget::Modal(_) | Widget::Drawer(_)
                ) {
                root.constraints
            } else {
                LogicalConstraints::unconstrained()
            };
            let size = match &node.state {
                Widget::Text(widget) => widget.layout(text_system, direction, constraints).size,
                Widget::Image(widget) => widget.layout(direction, constraints).size,
                Widget::Icon(widget) => widget.layout(direction, constraints).size(),
                Widget::Spacer(widget) => widget.layout(constraints),
                Widget::Divider(widget) => widget.layout(constraints),
                Widget::Surface(widget) => widget.layout(constraints),
                Widget::Container(widget) => widget.layout(constraints),
                Widget::Row(widget) => widget.layout(direction, &children, constraints).size,
                Widget::Column(widget) => widget.layout(direction, &children, constraints).size,
                Widget::Stack(widget) => {
                    let children = children
                        .iter()
                        .map(|child| StackChild::new(child.preferred))
                        .collect::<Vec<_>>();
                    widget.layout(direction, &children, constraints).size
                }
                Widget::ScrollView(widget) => {
                    let content = children.iter().fold(LogicalSize::default(), |size, child| {
                        LogicalSize::new(
                            size.width.max(child.preferred.width),
                            size.height.max(child.preferred.height),
                        )
                    });
                    widget.layout(direction, content, constraints).viewport
                }
                Widget::Button(widget) => {
                    widget
                        .layout(text_system, theme, direction, constraints)
                        .size
                }
                Widget::IconButton(widget) => widget.layout(theme, direction, constraints).size,
                Widget::Checkbox(widget) => {
                    widget
                        .layout(text_system, theme, direction, constraints)
                        .size
                }
                Widget::Radio(widget) => {
                    widget
                        .layout(text_system, theme, direction, constraints)
                        .size
                }
                Widget::Switch(widget) => {
                    widget
                        .layout(text_system, theme, direction, constraints)
                        .size
                }
                Widget::Slider(widget) => widget.layout(theme, direction, constraints).size,
                Widget::TextInput(widget) => {
                    widget
                        .layout(text_system, theme, direction, constraints)
                        .size
                }
                Widget::TextArea(widget) => {
                    widget
                        .layout(text_system, theme, direction, constraints)
                        .size
                }
                Widget::SearchInput(widget) => {
                    widget
                        .layout(text_system, theme, direction, constraints)
                        .size
                }
                Widget::Select(widget) => {
                    widget
                        .layout(text_system, theme, direction, constraints)
                        .size
                }
                Widget::Menu(widget) => {
                    widget
                        .layout(text_system, theme, direction, constraints)
                        .size
                }
                Widget::Dropdown(widget) => {
                    widget
                        .layout(text_system, theme, direction, constraints)
                        .size
                }
                Widget::ContextMenu(widget) => {
                    widget
                        .layout(text_system, theme, direction, constraints)
                        .size
                }
                Widget::Tooltip(widget) => {
                    widget
                        .layout(
                            text_system,
                            theme,
                            direction,
                            LogicalRect::new(root.origin, LogicalSize::default()),
                            constraints.max,
                        )
                        .size
                }
                Widget::Popover(widget) => {
                    widget
                        .layout(
                            theme,
                            direction,
                            LogicalRect::new(root.origin, LogicalSize::default()),
                            constraints.max,
                        )
                        .size
                }
                Widget::Modal(widget) => widget.layout(theme, constraints.max).size,
                Widget::Drawer(widget) => widget.layout(theme, direction, constraints.max).size,
            };
            measured.insert(id, size);
        }

        let mut placements = HashMap::with_capacity(tree.len());
        let root_size = measured[&tree.root()];
        placements.insert(
            tree.root(),
            WidgetPlacement::new(root.origin, LogicalConstraints::tight(root_size), direction),
        );
        for id in ids {
            let node = tree.get(id).unwrap();
            if node.children().is_empty() {
                continue;
            }
            let parent = placements[&id];
            let children = node
                .children()
                .iter()
                .map(|child| LayoutChild::new(measured[child]))
                .collect::<Vec<_>>();
            let child_bounds = match &node.state {
                Widget::Row(widget) => {
                    widget
                        .layout(direction, &children, parent.constraints)
                        .children
                }
                Widget::Column(widget) => {
                    widget
                        .layout(direction, &children, parent.constraints)
                        .children
                }
                Widget::Stack(widget) => {
                    let stack_children = children
                        .iter()
                        .map(|child| StackChild::new(child.preferred))
                        .collect::<Vec<_>>();
                    widget
                        .layout(direction, &stack_children, parent.constraints)
                        .children
                }
                Widget::ScrollView(widget) => {
                    let content = children.iter().fold(LogicalSize::default(), |size, child| {
                        LogicalSize::new(
                            size.width.max(child.preferred.width),
                            size.height.max(child.preferred.height),
                        )
                    });
                    let layout = widget.layout(direction, content, parent.constraints);
                    children
                        .iter()
                        .map(|child| {
                            LogicalRect::new(layout.content_bounds.origin, child.preferred)
                        })
                        .collect()
                }
                Widget::Popover(widget) => {
                    if !widget.open {
                        children.iter().map(|_| LogicalRect::default()).collect()
                    } else {
                        let layout = widget.layout(
                            theme,
                            direction,
                            LogicalRect::new(parent.origin, LogicalSize::default()),
                            parent.constraints.max,
                        );
                        let content = layout.content_bounds();
                        children
                            .iter()
                            .map(|child| {
                                LogicalRect::new(
                                    LogicalPoint::new(
                                        content.origin.x - parent.origin.x,
                                        content.origin.y - parent.origin.y,
                                    ),
                                    LogicalSize::new(
                                        child.preferred.width.min(content.size.width),
                                        child.preferred.height.min(content.size.height),
                                    ),
                                )
                            })
                            .collect()
                    }
                }
                Widget::Modal(widget) => {
                    if !widget.open {
                        children.iter().map(|_| LogicalRect::default()).collect()
                    } else {
                        let content = widget.layout(theme, parent.constraints.max).content_bounds;
                        children
                            .iter()
                            .map(|child| {
                                LogicalRect::new(
                                    content.origin,
                                    LogicalSize::new(
                                        child.preferred.width.min(content.size.width),
                                        child.preferred.height.min(content.size.height),
                                    ),
                                )
                            })
                            .collect()
                    }
                }
                Widget::Drawer(widget) => {
                    if !widget.open {
                        children.iter().map(|_| LogicalRect::default()).collect()
                    } else {
                        let content = widget
                            .layout(theme, direction, parent.constraints.max)
                            .content_bounds;
                        children
                            .iter()
                            .map(|child| {
                                LogicalRect::new(
                                    content.origin,
                                    LogicalSize::new(
                                        child.preferred.width.min(content.size.width),
                                        child.preferred.height.min(content.size.height),
                                    ),
                                )
                            })
                            .collect()
                    }
                }
                _ => children
                    .iter()
                    .map(|child| LogicalRect::new(LogicalPoint::default(), child.preferred))
                    .collect(),
            };
            for (child, bounds) in node.children().iter().copied().zip(child_bounds) {
                placements.insert(
                    child,
                    WidgetPlacement::new(
                        LogicalPoint::new(
                            parent.origin.x + bounds.origin.x,
                            parent.origin.y + bounds.origin.y,
                        ),
                        LogicalConstraints::tight(bounds.size),
                        direction,
                    ),
                );
            }
        }
        Self::build(tree, text_system, theme, |id, _| placements[&id])
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Direction, Icon, Image, LogicalConstraints, LogicalPoint, LogicalSize, PixelFormat,
        PixelImage, SemanticColorToken, SemanticRole, Text, ThemeController, ThemeDefinition,
        UserPreferences, WidgetTree,
    };

    use super::{Widget, WidgetFrame, WidgetPlacement};

    fn visual_digest(frame: &WidgetFrame, text_system: &mut crate::TextSystem) -> u64 {
        let mut hash = 0xcbf29ce484222325_u64;
        let mut feed = |bytes: &[u8]| {
            for byte in bytes {
                hash ^= u64::from(*byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        };
        for draw in &frame.text {
            feed(draw.text.as_bytes());
            for value in draw.baseline.into_iter().chain(draw.color) {
                feed(&value.to_bits().to_le_bytes());
            }
            let line = text_system.shape_line_with_style(&draw.text, &draw.style);
            for glyph in &line.glyphs {
                feed(&glyph.start.to_le_bytes());
                feed(&glyph.end.to_le_bytes());
                feed(&glyph.x.to_bits().to_le_bytes());
                feed(&glyph.width.to_bits().to_le_bytes());
                if let Some(image) = text_system.rasterize_glyph(glyph, 1.0) {
                    feed(&image.left.to_le_bytes());
                    feed(&image.top.to_le_bytes());
                    feed(&image.width.to_le_bytes());
                    feed(&image.height.to_le_bytes());
                    feed(&image.data);
                }
            }
        }
        hash
    }

    fn image_visual_digest(frame: &WidgetFrame) -> u64 {
        let mut hash = 0xcbf29ce484222325_u64;
        let mut feed = |bytes: &[u8]| {
            for byte in bytes {
                hash ^= u64::from(*byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        };
        for draw in &frame.images {
            feed(draw.image.data());
            feed(&draw.image.width().to_le_bytes());
            feed(&draw.image.height().to_le_bytes());
            for value in [
                draw.bounds.origin.x,
                draw.bounds.origin.y,
                draw.bounds.size.width,
                draw.bounds.size.height,
                draw.clip.origin.x,
                draw.clip.origin.y,
                draw.clip.size.width,
                draw.clip.size.height,
            ] {
                feed(&value.to_bits().to_le_bytes());
            }
            feed(&[u8::from(draw.mirror_horizontal)]);
            for value in draw.tint.unwrap_or_default() {
                feed(&value.to_bits().to_le_bytes());
            }
        }
        hash
    }

    fn button_visual_digest(frame: &WidgetFrame) -> u64 {
        let mut hash = 0xcbf29ce484222325_u64;
        let mut feed = |bytes: &[u8]| {
            for byte in bytes {
                hash ^= u64::from(*byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        };
        for draw in &frame.rectangles {
            for value in draw
                .position
                .into_iter()
                .chain(draw.size)
                .chain(draw.radii)
                .chain(draw.color)
                .chain([draw.border_width])
                .chain(draw.border_color)
            {
                feed(&value.to_bits().to_le_bytes());
            }
        }
        for draw in &frame.text {
            feed(draw.text.as_bytes());
            for value in draw.baseline.into_iter().chain(draw.color) {
                feed(&value.to_bits().to_le_bytes());
            }
        }
        feed(&image_visual_digest(frame).to_le_bytes());
        hash
    }

    fn text_frame(
        content: &str,
        direction: Direction,
        width: f32,
    ) -> (WidgetFrame, crate::TextSystem) {
        let tree = WidgetTree::new(Widget::from(Text::new(content)));
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(11.5, 17.25),
                LogicalConstraints::loose(LogicalSize::new(width, 160.0)),
                direction,
            )
        });
        (frame, text_system)
    }

    #[test]
    fn retained_text_frame_freezes_matching_geometry_semantics_and_paint() {
        let mut tree = WidgetTree::new(Widget::from(Text::new("رابط Mio-GUI")));
        let id = tree.root();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(20.0, 30.0),
                LogicalConstraints::loose(LogicalSize::new(180.0, 100.0)),
                Direction::Rtl,
            )
        });

        let geometry = frame.geometry.get(id).unwrap();
        let semantics = frame.semantics.get(id).unwrap();
        assert_eq!(geometry.bounds.origin, LogicalPoint::new(20.0, 30.0));
        assert_eq!(semantics.semantics.role, SemanticRole::Text);
        assert_eq!(semantics.semantics.name.as_deref(), Some("رابط Mio-GUI"));
        assert_eq!(frame.text.len(), 1);
        assert_eq!(frame.text[0].text, "رابط Mio-GUI");
        assert_eq!(frame.text[0].color, theme.colors.text.to_array());
        assert!(frame.rectangles.is_empty());

        tree.get_mut(id).unwrap().state = Widget::from(Text::new("changed after frame"));
        assert_eq!(frame.text[0].text, "رابط Mio-GUI");
    }

    #[test]
    fn retained_tree_paint_order_is_preserved_in_text_draw_order() {
        let mut tree = WidgetTree::new(Widget::from(Text::new("first")));
        let root = tree.root();
        let second = tree
            .append(root, Widget::from(Text::new("second")))
            .unwrap();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
            WidgetPlacement::new(
                LogicalPoint::new(0.0, if id == root { 0.0 } else { 30.0 }),
                LogicalConstraints::unconstrained(),
                Direction::Ltr,
            )
        });

        assert_eq!(frame.geometry.paint_order(), &[root, second]);
        assert_eq!(frame.text[0].text, "first");
        assert_eq!(frame.text[1].text, "second");
    }

    #[test]
    fn retained_image_and_icon_frame_freezes_geometry_semantics_and_paint_order() {
        let image_source = PixelImage::new(2, 1, PixelFormat::Rgba8, vec![12_u8; 8]).unwrap();
        let icon_source = PixelImage::new(1, 2, PixelFormat::Alpha8, vec![255_u8; 2]).unwrap();
        let mut tree = WidgetTree::new(Widget::from(
            Image::new(image_source).with_alternative_text("Mio logo"),
        ));
        let root = tree.root();
        let text = tree
            .append(root, Widget::from(Text::new("between")))
            .unwrap();
        let mut icon = Icon::new(icon_source)
            .unwrap()
            .with_alternative_text("Open");
        icon.color = SemanticColorToken::Primary;
        let icon_id = tree.append(root, Widget::from(icon)).unwrap();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
            WidgetPlacement::new(
                LogicalPoint::new(if id == root { 5.0 } else { 30.0 }, 7.0),
                LogicalConstraints::tight(LogicalSize::new(20.0, 10.0)),
                Direction::Rtl,
            )
        });

        assert_eq!(frame.geometry.paint_order(), &[root, text, icon_id]);
        assert_eq!(frame.images.len(), 2);
        assert_eq!(frame.images[0].image.format(), PixelFormat::Rgba8);
        assert_eq!(frame.images[0].tint, None);
        assert_eq!(frame.images[1].image.format(), PixelFormat::Alpha8);
        assert_eq!(frame.images[1].tint, Some(theme.colors.primary.to_array()));
        assert_eq!(frame.images[0].clip.origin, LogicalPoint::new(5.0, 7.0));
        assert_eq!(frame.images[1].clip.origin, LogicalPoint::new(30.0, 7.0));
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.name.as_deref(),
            Some("Mio logo")
        );
        assert_eq!(
            frame
                .semantics
                .get(icon_id)
                .unwrap()
                .semantics
                .name
                .as_deref(),
            Some("Open")
        );

        tree.get_mut(root).unwrap().state = Widget::from(Text::new("changed after frame"));
        assert_eq!(frame.images[0].image.data(), &[12_u8; 8]);
    }

    #[test]
    fn retained_button_and_icon_button_freeze_semantics_geometry_and_paint() {
        use crate::{AdornmentPlacement, Button, IconButton, SemanticAction, VisualVariant};

        let mask =
            || Icon::new(PixelImage::new(1, 1, PixelFormat::Alpha8, vec![255]).unwrap()).unwrap();
        let mut button = Button::new("Continue").with_icon(mask(), AdornmentPlacement::InlineEnd);
        button.style.variant = VisualVariant::Solid;
        let mut tree = WidgetTree::new(Widget::from(button));
        let root = tree.root();
        let icon_button = tree
            .append(root, Widget::from(IconButton::new(mask(), "Menu")))
            .unwrap();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
            WidgetPlacement::new(
                LogicalPoint::new(if id == root { 8.0 } else { 180.0 }, 12.0),
                LogicalConstraints::unconstrained(),
                Direction::Rtl,
            )
        });

        let semantics = &frame.semantics.get(root).unwrap().semantics;
        assert_eq!(semantics.role, SemanticRole::Button);
        assert!(semantics.supports(SemanticAction::Activate));
        assert_eq!(frame.rectangles.len(), 2);
        assert_eq!(frame.text.len(), 1);
        assert_eq!(frame.text[0].text, "Continue");
        assert_eq!(frame.images.len(), 2);
        assert_eq!(
            frame.geometry.get(icon_button).unwrap().bounds.size.width,
            frame.geometry.get(icon_button).unwrap().bounds.size.height
        );

        tree.get_mut(root).unwrap().state = Widget::from(Button::new("Changed"));
        assert_eq!(frame.text[0].text, "Continue");
    }

    #[test]
    fn retained_checkbox_freezes_checked_semantics_geometry_and_paint() {
        use crate::{Checkbox, SemanticAction};

        let mut checkbox = Checkbox::new("Remember me");
        checkbox.checked = true;
        let mut tree = WidgetTree::new(Widget::from(checkbox));
        let root = tree.root();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(6.0, 9.0),
                LogicalConstraints::unconstrained(),
                Direction::Rtl,
            )
        });

        let semantics = &frame.semantics.get(root).unwrap().semantics;
        assert_eq!(semantics.role, SemanticRole::Checkbox);
        assert_eq!(semantics.state.checked, Some(true));
        assert!(semantics.supports(SemanticAction::Activate));
        assert!(frame.geometry.get(root).unwrap().bounds.size.width > 16.0);
        assert_eq!(frame.rectangles.len(), 2);
        assert_eq!(frame.text.len(), 1);
        assert_eq!(frame.text[0].text, "Remember me");

        tree.get_mut(root).unwrap().state = Widget::from(Checkbox::new("Changed"));
        assert_eq!(frame.text[0].text, "Remember me");
        assert_eq!(frame.rectangles[0].color, theme.colors.primary.to_array());
    }

    #[test]
    fn checkbox_keyboard_activation_respects_disabled_focus_policy() {
        use crate::{
            Checkbox, FocusSnapshot, Key, KeyboardEvent, SemanticAction, semantic_action_for_key,
        };

        let checkbox = Checkbox::new("Remember me");
        let tree = WidgetTree::new(Widget::from(checkbox));
        let focus = FocusSnapshot::build(&tree, |_, widget| widget.focus_policy());
        assert_eq!(focus.tab_order(), &[tree.root()]);
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::default(),
                LogicalConstraints::unconstrained(),
                Direction::Ltr,
            )
        });
        let space = KeyboardEvent::pressed(Key::Space);
        assert_eq!(
            semantic_action_for_key(&frame.semantics, tree.root(), &space, Direction::Ltr)
                .unwrap()
                .action,
            SemanticAction::Activate
        );

        let mut disabled = Checkbox::new("Disabled");
        disabled.disabled = true;
        let tree = WidgetTree::new(Widget::from(disabled));
        let focus = FocusSnapshot::build(&tree, |_, widget| widget.focus_policy());
        assert!(focus.tab_order().is_empty());
    }

    #[test]
    fn retained_checkbox_visual_outputs_match_direction_goldens() {
        use crate::Checkbox;

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let fixtures = [
            (Direction::Ltr, false, 14149347552930099619_u64),
            (Direction::Rtl, true, 7749473330257813281_u64),
        ];
        for (direction, checked, expected) in fixtures {
            let mut checkbox = Checkbox::new("Remember me");
            checkbox.checked = checked;
            let tree = WidgetTree::new(Widget::from(checkbox));
            let mut text_system = crate::TextSystem::new();
            let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                WidgetPlacement::new(
                    LogicalPoint::new(7.0, 11.0),
                    LogicalConstraints::unconstrained(),
                    direction,
                )
            });
            let actual = button_visual_digest(&frame);
            assert_eq!(actual, expected, "direction={direction:?} actual={actual}");
        }
    }

    #[test]
    fn retained_radio_supports_focus_keyboard_semantics_and_frozen_paint() {
        use crate::{
            FocusSnapshot, Key, KeyboardEvent, Radio, SemanticAction, semantic_action_for_key,
        };

        let mut radio = Radio::new("Standard delivery");
        radio.selected = true;
        let mut tree = WidgetTree::new(Widget::from(radio));
        let root = tree.root();
        let focus = FocusSnapshot::build(&tree, |_, widget| widget.focus_policy());
        assert_eq!(focus.tab_order(), &[root]);
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(4.0, 7.0),
                LogicalConstraints::unconstrained(),
                Direction::Rtl,
            )
        });
        let space = KeyboardEvent::pressed(Key::Space);
        assert_eq!(
            semantic_action_for_key(&frame.semantics, root, &space, Direction::Rtl)
                .unwrap()
                .action,
            SemanticAction::Activate
        );
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.role,
            SemanticRole::Radio
        );
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.state.checked,
            Some(true)
        );
        assert_eq!(frame.rectangles.len(), 2);
        assert_eq!(frame.text[0].text, "Standard delivery");

        tree.get_mut(root).unwrap().state = Widget::from(Radio::new("Changed"));
        assert_eq!(frame.text[0].text, "Standard delivery");
    }

    #[test]
    fn retained_radio_visual_outputs_match_direction_goldens() {
        use crate::Radio;

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let fixtures = [
            (Direction::Ltr, false, 17449127050552886139_u64),
            (Direction::Rtl, true, 537322983665427384_u64),
        ];
        for (direction, selected, expected) in fixtures {
            let mut radio = Radio::new("Standard delivery");
            radio.selected = selected;
            let tree = WidgetTree::new(Widget::from(radio));
            let mut text_system = crate::TextSystem::new();
            let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                WidgetPlacement::new(
                    LogicalPoint::new(7.0, 11.0),
                    LogicalConstraints::unconstrained(),
                    direction,
                )
            });
            let actual = button_visual_digest(&frame);
            assert_eq!(actual, expected, "direction={direction:?} actual={actual}");
        }
    }

    #[test]
    fn retained_switch_supports_keyboard_semantics_and_frozen_paint() {
        use crate::{Key, KeyboardEvent, SemanticAction, Switch, semantic_action_for_key};

        let mut switch = Switch::new("Notifications");
        switch.checked = true;
        let mut tree = WidgetTree::new(Widget::from(switch));
        let root = tree.root();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(4.0, 7.0),
                LogicalConstraints::unconstrained(),
                Direction::Rtl,
            )
        });
        let space = KeyboardEvent::pressed(Key::Space);
        assert_eq!(
            semantic_action_for_key(&frame.semantics, root, &space, Direction::Rtl)
                .unwrap()
                .action,
            SemanticAction::Activate
        );
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.role,
            SemanticRole::Switch
        );
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.state.checked,
            Some(true)
        );
        assert_eq!(frame.rectangles.len(), 2);
        assert_eq!(frame.text[0].text, "Notifications");
        tree.get_mut(root).unwrap().state = Widget::from(Switch::new("Changed"));
        assert_eq!(frame.text[0].text, "Notifications");
    }

    #[test]
    fn retained_switch_visual_outputs_match_direction_goldens() {
        use crate::Switch;

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let fixtures = [
            (Direction::Ltr, false, 1930133800622114258_u64),
            (Direction::Rtl, true, 6363977688928847054_u64),
        ];
        for (direction, checked, expected) in fixtures {
            let mut switch = Switch::new("Notifications");
            switch.checked = checked;
            let tree = WidgetTree::new(Widget::from(switch));
            let mut text_system = crate::TextSystem::new();
            let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                WidgetPlacement::new(
                    LogicalPoint::new(7.0, 11.0),
                    LogicalConstraints::unconstrained(),
                    direction,
                )
            });
            let actual = button_visual_digest(&frame);
            assert_eq!(actual, expected, "direction={direction:?} actual={actual}");
        }
    }

    #[test]
    fn retained_slider_supports_directional_keyboard_semantics_and_frozen_paint() {
        use crate::{
            ArrowKey, Key, KeyboardEvent, SemanticAction, Slider, semantic_action_for_key,
        };

        let slider = Slider::new("Volume", 0.0..=100.0, 25.0).unwrap();
        let mut tree = WidgetTree::new(Widget::from(slider));
        let root = tree.root();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(4.0, 7.0),
                LogicalConstraints::unconstrained(),
                Direction::Rtl,
            )
        });
        let right = KeyboardEvent::pressed(Key::Arrow(ArrowKey::Right));
        assert_eq!(
            semantic_action_for_key(&frame.semantics, root, &right, Direction::Rtl)
                .unwrap()
                .action,
            SemanticAction::Decrement
        );
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.role,
            SemanticRole::Slider
        );
        assert_eq!(
            frame
                .semantics
                .get(root)
                .unwrap()
                .semantics
                .value
                .as_deref(),
            Some("25")
        );
        assert_eq!(frame.rectangles.len(), 3);
        tree.get_mut(root).unwrap().state =
            Widget::from(Slider::new("Changed", 0.0..=1.0, 1.0).unwrap());
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.name.as_deref(),
            Some("Volume")
        );
    }

    #[test]
    fn retained_slider_visual_outputs_match_direction_goldens() {
        use crate::Slider;

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let fixtures = [
            (Direction::Ltr, 25.0, 9162285720261724344_u64),
            (Direction::Rtl, 75.0, 3651357042649310350_u64),
        ];
        for (direction, value, expected) in fixtures {
            let slider = Slider::new("Volume", 0.0..=100.0, value).unwrap();
            let tree = WidgetTree::new(Widget::from(slider));
            let mut text_system = crate::TextSystem::new();
            let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                WidgetPlacement::new(
                    LogicalPoint::new(7.0, 11.0),
                    LogicalConstraints::unconstrained(),
                    direction,
                )
            });
            let actual = button_visual_digest(&frame);
            assert_eq!(actual, expected, "direction={direction:?} actual={actual}");
        }
    }

    #[test]
    fn retained_text_input_freezes_form_semantics_geometry_and_paint() {
        use crate::{SemanticAction, TextInput};

        let mut input = TextInput::with_text("Name", "Reza");
        input.required = true;
        let mut tree = WidgetTree::new(Widget::from(input));
        let root = tree.root();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(6.0, 9.0),
                LogicalConstraints::unconstrained(),
                Direction::Rtl,
            )
        });
        let semantics = &frame.semantics.get(root).unwrap().semantics;
        assert_eq!(semantics.role, SemanticRole::TextField);
        assert_eq!(semantics.name.as_deref(), Some("Name"));
        assert_eq!(semantics.value.as_deref(), Some("Reza"));
        assert!(semantics.state.required);
        assert!(semantics.supports(SemanticAction::SetValue));
        assert_eq!(frame.rectangles.len(), 1);
        assert_eq!(frame.text[0].text, "Reza");
        assert!(frame.geometry.get(root).unwrap().bounds.size.width >= 160.0);

        tree.get_mut(root).unwrap().state = Widget::from(TextInput::new("Changed"));
        assert_eq!(frame.text[0].text, "Reza");
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.name.as_deref(),
            Some("Name")
        );
    }

    #[test]
    fn retained_text_input_visual_outputs_match_direction_goldens() {
        use crate::TextInput;

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let fixtures = [
            (Direction::Ltr, false, 2449172446527851015_u64),
            (Direction::Rtl, true, 13475687873425295627_u64),
        ];
        for (direction, populated, expected) in fixtures {
            let mut input = if populated {
                TextInput::with_text("Name", "رضا")
            } else {
                TextInput::new("Name")
            };
            input.set_placeholder("Enter name");
            let tree = WidgetTree::new(Widget::from(input));
            let mut text_system = crate::TextSystem::new();
            let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                WidgetPlacement::new(
                    LogicalPoint::new(7.0, 11.0),
                    LogicalConstraints::unconstrained(),
                    direction,
                )
            });
            let actual = button_visual_digest(&frame);
            assert_eq!(actual, expected, "direction={direction:?} actual={actual}");
        }
    }

    #[test]
    fn retained_text_area_freezes_multiline_semantics_geometry_and_paint() {
        use crate::TextArea;

        let mut area = TextArea::with_text("Notes", "first line\nsecond line");
        area.set_minimum_lines(4);
        let mut tree = WidgetTree::new(Widget::from(area));
        let root = tree.root();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(6.0, 9.0),
                LogicalConstraints::loose(LogicalSize::new(180.0, 300.0)),
                Direction::Ltr,
            )
        });
        let semantics = &frame.semantics.get(root).unwrap().semantics;
        assert_eq!(semantics.role, SemanticRole::MultilineTextField);
        assert_eq!(semantics.name.as_deref(), Some("Notes"));
        assert_eq!(semantics.value.as_deref(), Some("first line\nsecond line"));
        assert_eq!(frame.rectangles.len(), 1);
        assert_eq!(frame.text.len(), 2);
        assert!(
            frame.geometry.get(root).unwrap().bounds.size.height
                >= 4.0 * theme.typography.line_height
        );

        tree.get_mut(root).unwrap().state = Widget::from(TextArea::new("Changed"));
        assert_eq!(frame.text[0].text, "first line");
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.name.as_deref(),
            Some("Notes")
        );
    }

    #[test]
    fn retained_text_area_visual_outputs_match_direction_goldens() {
        use crate::TextArea;

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let fixtures = [
            (
                Direction::Ltr,
                "one two three four five",
                2594894344163121533_u64,
            ),
            (Direction::Rtl, "سطر اول\nسطر دوم", 12802436581539432392_u64),
        ];
        for (direction, value, expected) in fixtures {
            let area = TextArea::with_text("Notes", value);
            let tree = WidgetTree::new(Widget::from(area));
            let mut text_system = crate::TextSystem::new();
            let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                WidgetPlacement::new(
                    LogicalPoint::new(7.0, 11.0),
                    LogicalConstraints::loose(LogicalSize::new(120.0, 300.0)),
                    direction,
                )
            });
            let actual = button_visual_digest(&frame);
            assert_eq!(actual, expected, "direction={direction:?} actual={actual}");
        }
    }

    #[test]
    fn retained_search_input_freezes_semantics_adornment_and_paint() {
        use crate::SearchInput;

        let mut tree = WidgetTree::new(Widget::from(SearchInput::with_text("Site search", "Mio")));
        let root = tree.root();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(6.0, 9.0),
                LogicalConstraints::unconstrained(),
                Direction::Rtl,
            )
        });
        let semantics = &frame.semantics.get(root).unwrap().semantics;
        assert_eq!(semantics.role, SemanticRole::SearchField);
        assert_eq!(semantics.name.as_deref(), Some("Site search"));
        assert_eq!(semantics.value.as_deref(), Some("Mio"));
        assert_eq!(frame.rectangles.len(), 1);
        assert_eq!(frame.text[0].text, "Mio");
        assert_eq!(frame.images.len(), 1);
        assert_eq!(frame.images[0].image.format(), PixelFormat::Alpha8);

        tree.get_mut(root).unwrap().state = Widget::from(SearchInput::new("Changed"));
        assert_eq!(frame.text[0].text, "Mio");
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.name.as_deref(),
            Some("Site search")
        );
    }

    #[test]
    fn retained_search_input_visual_outputs_match_direction_goldens() {
        use crate::SearchInput;

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let fixtures = [
            (Direction::Ltr, false, 5982683478870831534_u64),
            (Direction::Rtl, true, 17272577147494924067_u64),
        ];
        for (direction, populated, expected) in fixtures {
            let search = if populated {
                SearchInput::with_text("Site search", "جستجو")
            } else {
                SearchInput::new("Site search")
            };
            let tree = WidgetTree::new(Widget::from(search));
            let mut text_system = crate::TextSystem::new();
            let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                WidgetPlacement::new(
                    LogicalPoint::new(7.0, 11.0),
                    LogicalConstraints::unconstrained(),
                    direction,
                )
            });
            let actual = button_visual_digest(&frame);
            assert_eq!(actual, expected, "direction={direction:?} actual={actual}");
        }
    }

    #[test]
    fn retained_select_freezes_value_expansion_focus_and_paint() {
        use crate::{FocusSnapshot, Select, SelectOption, SemanticAction};

        let options = vec![
            SelectOption::new("Small", "sm"),
            SelectOption::new("Large", "lg"),
        ];
        let mut select = Select::new("Size", options).unwrap();
        select.select(1).unwrap();
        select.open = true;
        let mut tree = WidgetTree::new(Widget::from(select));
        let root = tree.root();
        let focus = FocusSnapshot::build(&tree, |_, widget| widget.focus_policy());
        assert_eq!(focus.tab_order(), &[root]);
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(6.0, 9.0),
                LogicalConstraints::unconstrained(),
                Direction::Rtl,
            )
        });
        let semantics = &frame.semantics.get(root).unwrap().semantics;
        assert_eq!(semantics.role, SemanticRole::ComboBox);
        assert_eq!(semantics.name.as_deref(), Some("Size"));
        assert_eq!(semantics.value.as_deref(), Some("Large"));
        assert_eq!(semantics.state.expanded, Some(true));
        assert!(semantics.supports(SemanticAction::ShowMenu));
        assert_eq!(semantics.virtual_children().len(), 2);
        let first_option = semantics.virtual_child_bounds(0).unwrap();
        assert!(first_option.origin.y > frame.geometry.get(root).unwrap().bounds.origin.y);
        assert_eq!(
            first_option.size.width,
            frame.geometry.get(root).unwrap().bounds.size.width
        );
        assert_eq!(frame.rectangles.len(), 3);
        assert_eq!(frame.text.len(), 3);
        assert_eq!(frame.text[0].text, "Large");
        assert_eq!(frame.text[1].text, "Small");
        assert_eq!(frame.text[2].text, "Large");
        assert_eq!(frame.images.len(), 1);

        let replacement =
            Select::new("Changed", vec![SelectOption::new("Other", "other")]).unwrap();
        tree.get_mut(root).unwrap().state = Widget::from(replacement);
        assert_eq!(frame.text[0].text, "Large");
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.name.as_deref(),
            Some("Size")
        );
    }

    #[test]
    fn retained_select_visual_outputs_match_direction_goldens() {
        use crate::{Select, SelectOption};

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let fixtures = [
            (Direction::Ltr, 2573753279803471247_u64),
            (Direction::Rtl, 8102710045918514718_u64),
        ];
        for (direction, expected) in fixtures {
            let select = Select::new(
                "Size",
                vec![
                    SelectOption::new("Small", "sm"),
                    SelectOption::new("Large", "lg"),
                ],
            )
            .unwrap();
            let tree = WidgetTree::new(Widget::from(select));
            let mut text_system = crate::TextSystem::new();
            let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                WidgetPlacement::new(
                    LogicalPoint::new(7.0, 11.0),
                    LogicalConstraints::unconstrained(),
                    direction,
                )
            });
            let actual = button_visual_digest(&frame);
            assert_eq!(actual, expected, "direction={direction:?} actual={actual}");
        }
    }

    #[test]
    fn retained_menu_freezes_semantics_geometry_and_item_paint() {
        use crate::{FocusSnapshot, Menu, MenuItem};

        let items = vec![
            MenuItem::new("Open"),
            MenuItem {
                disabled: true,
                ..MenuItem::new("Rename")
            },
            MenuItem::new("Delete"),
        ];
        let mut tree = WidgetTree::new(Widget::from(Menu::new("Actions", items).unwrap()));
        let root = tree.root();
        let focus = FocusSnapshot::build(&tree, |_, widget| widget.focus_policy());
        assert_eq!(focus.tab_order(), &[root]);
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(6.0, 9.0),
                LogicalConstraints::unconstrained(),
                Direction::Rtl,
            )
        });
        let semantics = &frame.semantics.get(root).unwrap().semantics;
        assert_eq!(semantics.role, SemanticRole::Menu);
        assert_eq!(semantics.name.as_deref(), Some("Actions"));
        assert_eq!(semantics.virtual_children().len(), 3);
        assert!(semantics.virtual_children()[1].state.disabled);
        let first_item = semantics.virtual_child_bounds(0).unwrap();
        assert_eq!(first_item.origin.x, 6.0);
        assert!(first_item.origin.y > 9.0);
        assert_eq!(
            first_item.size.width,
            frame.geometry.get(root).unwrap().bounds.size.width
        );
        assert_eq!(frame.rectangles.len(), 2);
        assert_eq!(frame.text.len(), 3);
        assert!(frame.geometry.get(root).unwrap().bounds.size.height > 60.0);

        tree.get_mut(root).unwrap().state =
            Widget::from(Menu::new("Changed", vec![MenuItem::new("Other")]).unwrap());
        assert_eq!(frame.text[0].text, "Open");
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.name.as_deref(),
            Some("Actions")
        );
    }

    #[test]
    fn retained_menu_visual_outputs_match_direction_goldens() {
        use crate::{Menu, MenuItem};

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let fixtures = [
            (Direction::Ltr, 7697199954990930349_u64),
            (Direction::Rtl, 9379700057365111330_u64),
        ];
        for (direction, expected) in fixtures {
            let menu = Menu::new(
                "Actions",
                vec![
                    MenuItem::new("Open"),
                    MenuItem::new("Rename"),
                    MenuItem::new("Delete"),
                ],
            )
            .unwrap();
            let tree = WidgetTree::new(Widget::from(menu));
            let mut text_system = crate::TextSystem::new();
            let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                WidgetPlacement::new(
                    LogicalPoint::new(7.0, 11.0),
                    LogicalConstraints::unconstrained(),
                    direction,
                )
            });
            let actual = button_visual_digest(&frame);
            assert_eq!(actual, expected, "direction={direction:?} actual={actual}");
        }
    }

    #[test]
    fn retained_dropdown_freezes_expansion_geometry_and_combined_paint() {
        use crate::{Button, Dropdown, FocusSnapshot, Menu, MenuItem, SemanticAction};

        let menu = Menu::new(
            "Actions",
            vec![MenuItem::new("Open"), MenuItem::new("Delete")],
        )
        .unwrap();
        let mut dropdown = Dropdown::new(Button::new("Actions"), menu);
        dropdown.open = true;
        let mut tree = WidgetTree::new(Widget::from(dropdown));
        let root = tree.root();
        let focus = FocusSnapshot::build(&tree, |_, widget| widget.focus_policy());
        assert_eq!(focus.tab_order(), &[root]);
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(6.0, 9.0),
                LogicalConstraints::unconstrained(),
                Direction::Rtl,
            )
        });
        let semantics = &frame.semantics.get(root).unwrap().semantics;
        assert_eq!(semantics.role, SemanticRole::Button);
        assert_eq!(semantics.state.expanded, Some(true));
        assert!(semantics.supports(SemanticAction::ShowMenu));
        let first_option = semantics.virtual_child_bounds(0).unwrap();
        assert!(first_option.origin.y > frame.geometry.get(root).unwrap().bounds.origin.y);
        assert_eq!(
            first_option.size.width,
            frame.geometry.get(root).unwrap().bounds.size.width
        );
        assert_eq!(frame.rectangles.len(), 3);
        assert_eq!(frame.text.len(), 3);
        assert!(frame.geometry.get(root).unwrap().bounds.size.height > 80.0);

        let replacement = Dropdown::new(
            Button::new("Changed"),
            Menu::new("Changed", vec![MenuItem::new("Other")]).unwrap(),
        );
        tree.get_mut(root).unwrap().state = Widget::from(replacement);
        assert_eq!(frame.text[0].text, "Actions");
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.state.expanded,
            Some(true)
        );
    }

    #[test]
    fn retained_dropdown_visual_outputs_match_direction_goldens() {
        use crate::{Button, Dropdown, Menu, MenuItem};

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let fixtures = [
            (Direction::Ltr, 15150853515874983511_u64),
            (Direction::Rtl, 7577439035433286427_u64),
        ];
        for (direction, expected) in fixtures {
            let menu = Menu::new(
                "Actions",
                vec![MenuItem::new("Open"), MenuItem::new("Delete")],
            )
            .unwrap();
            let mut dropdown = Dropdown::new(Button::new("Actions"), menu);
            dropdown.open = true;
            let tree = WidgetTree::new(Widget::from(dropdown));
            let mut text_system = crate::TextSystem::new();
            let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                WidgetPlacement::new(
                    LogicalPoint::new(7.0, 11.0),
                    LogicalConstraints::unconstrained(),
                    direction,
                )
            });
            let actual = button_visual_digest(&frame);
            assert_eq!(actual, expected, "direction={direction:?} actual={actual}");
        }
    }

    #[test]
    fn retained_dropdown_visual_matrix_matches_open_state_theme_direction_and_text_scale_golden() {
        use crate::{
            Button, ColorScheme, ComponentState, Dropdown, Menu, MenuItem, ThemeMode, VisualVariant,
        };

        let states = [
            ComponentState::default(),
            ComponentState {
                hovered: true,
                ..ComponentState::default()
            },
            ComponentState {
                active: true,
                ..ComponentState::default()
            },
            ComponentState {
                focused: true,
                ..ComponentState::default()
            },
            ComponentState {
                disabled: true,
                ..ComponentState::default()
            },
        ];
        let mut golden = 0xcbf29ce484222325_u64;
        for scheme in [ColorScheme::Light, ColorScheme::Dark] {
            for direction in [Direction::Ltr, Direction::Rtl] {
                for text_scale in [1.0, 1.5] {
                    for open in [false, true] {
                        for state in states {
                            let mut controller = ThemeController::default();
                            controller.set_mode(match scheme {
                                ColorScheme::Light => ThemeMode::Light,
                                ColorScheme::Dark => ThemeMode::Dark,
                            });
                            let mut preferences = UserPreferences::default();
                            preferences.set_text_scale(text_scale);
                            let theme = ThemeDefinition::default().resolve(controller, preferences);
                            let mut trigger = Button::new("Actions");
                            trigger.style.variant = VisualVariant::Outline;
                            trigger.style.state = state;
                            let menu = Menu::new(
                                "Actions",
                                vec![MenuItem::new("Open"), MenuItem::new("Delete")],
                            )
                            .unwrap();
                            let mut dropdown = Dropdown::new(trigger, menu);
                            dropdown.open = open;
                            let tree = WidgetTree::new(Widget::from(dropdown));
                            let mut text_system = crate::TextSystem::new();
                            let frame =
                                WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                                    WidgetPlacement::new(
                                        LogicalPoint::new(7.0, 11.0),
                                        LogicalConstraints::unconstrained(),
                                        direction,
                                    )
                                });
                            golden ^= button_visual_digest(&frame);
                            golden = golden.wrapping_mul(0x100000001b3);
                        }
                    }
                }
            }
        }

        assert_eq!(golden, 8043682802775564477);
    }

    #[test]
    fn retained_context_menu_freezes_clamped_overlay_semantics_and_paint() {
        use crate::{ContextMenu, FocusSnapshot, Menu, MenuItem};

        let mut menu = ContextMenu::new(
            Menu::new(
                "Context actions",
                vec![MenuItem::new("Copy"), MenuItem::new("Delete")],
            )
            .unwrap(),
        );
        menu.open_at(LogicalPoint::new(195.0, 95.0));
        let mut tree = WidgetTree::new(Widget::from(menu));
        let root = tree.root();
        let focus = FocusSnapshot::build(&tree, |_, widget| widget.focus_policy());
        assert_eq!(focus.tab_order(), &[root]);
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(0.0, 0.0),
                LogicalConstraints::tight(LogicalSize::new(200.0, 100.0)),
                Direction::Rtl,
            )
        });
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.role,
            SemanticRole::Menu
        );
        assert_eq!(frame.rectangles.len(), 2);
        assert_eq!(frame.text.len(), 2);
        assert!(frame.rectangles[0].position[0] >= 0.0);
        assert!(frame.rectangles[0].position[1] >= 0.0);

        tree.get_mut(root).unwrap().state = Widget::from(ContextMenu::new(
            Menu::new("Changed", vec![MenuItem::new("Other")]).unwrap(),
        ));
        assert_eq!(frame.text[0].text, "Copy");
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.name.as_deref(),
            Some("Context actions")
        );
    }

    #[test]
    fn retained_context_menu_visual_outputs_match_direction_goldens() {
        use crate::{ContextMenu, Menu, MenuItem};

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let fixtures = [
            (Direction::Ltr, 4093010191304192875_u64),
            (Direction::Rtl, 3960272594419871006_u64),
        ];
        for (direction, expected) in fixtures {
            let mut context = ContextMenu::new(
                Menu::new(
                    "Context actions",
                    vec![MenuItem::new("Copy"), MenuItem::new("Delete")],
                )
                .unwrap(),
            );
            context.open_at(LogicalPoint::new(145.0, 70.0));
            let tree = WidgetTree::new(Widget::from(context));
            let mut text_system = crate::TextSystem::new();
            let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                WidgetPlacement::new(
                    LogicalPoint::default(),
                    LogicalConstraints::tight(LogicalSize::new(180.0, 120.0)),
                    direction,
                )
            });
            let actual = button_visual_digest(&frame);
            assert_eq!(actual, expected, "direction={direction:?} actual={actual}");
        }
    }

    #[test]
    fn retained_tooltip_freezes_collision_adjusted_geometry_semantics_and_paint() {
        use crate::{Tooltip, TooltipPlacement};

        let mut tooltip = Tooltip::new("More information");
        tooltip.visible = true;
        tooltip.placement = TooltipPlacement::BlockEnd;
        let mut tree = WidgetTree::new(Widget::from(tooltip));
        let root = tree.root();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(190.0, 90.0),
                LogicalConstraints::tight(LogicalSize::new(200.0, 100.0)),
                Direction::Rtl,
            )
        });
        let bounds = frame.geometry.get(root).unwrap().bounds;
        assert!(bounds.origin.x + bounds.size.width <= 200.0);
        assert!(bounds.origin.y + bounds.size.height <= 100.0);
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.name.as_deref(),
            Some("More information")
        );
        assert_eq!(frame.rectangles.len(), 1);
        assert_eq!(frame.text[0].text, "More information");

        tree.get_mut(root).unwrap().state = Widget::from(Tooltip::new("Changed"));
        assert_eq!(frame.text[0].text, "More information");
    }

    #[test]
    fn retained_tooltip_visual_outputs_match_direction_goldens() {
        use crate::{Tooltip, TooltipPlacement};

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let fixtures = [
            (Direction::Ltr, 3423733459360532881_u64),
            (Direction::Rtl, 11197503365628108745_u64),
        ];
        for (direction, expected) in fixtures {
            let mut tooltip = Tooltip::new("Keyboard shortcut");
            tooltip.visible = true;
            tooltip.placement = TooltipPlacement::InlineStart;
            let tree = WidgetTree::new(Widget::from(tooltip));
            let mut text_system = crate::TextSystem::new();
            let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                WidgetPlacement::new(
                    LogicalPoint::new(100.0, 50.0),
                    LogicalConstraints::tight(LogicalSize::new(240.0, 120.0)),
                    direction,
                )
            });
            let actual = button_visual_digest(&frame);
            assert_eq!(actual, expected, "direction={direction:?} actual={actual}");
        }
    }

    #[test]
    fn retained_popover_freezes_overlay_semantics_and_panel_paint() {
        use crate::{FocusSnapshot, Popover, TooltipPlacement};

        let mut popover = Popover::new("Formatting", LogicalSize::new(100.0, 60.0));
        popover.open = true;
        popover.placement = TooltipPlacement::InlineStart;
        let mut tree = WidgetTree::new(Widget::from(popover));
        let root = tree.root();
        let focus = FocusSnapshot::build(&tree, |_, widget| widget.focus_policy());
        assert_eq!(focus.tab_order(), &[root]);
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(130.0, 60.0),
                LogicalConstraints::tight(LogicalSize::new(240.0, 140.0)),
                Direction::Rtl,
            )
        });
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.role,
            SemanticRole::Dialog
        );
        assert_eq!(frame.rectangles.len(), 1);
        assert_eq!(
            frame.geometry.get(root).unwrap().bounds.size,
            LogicalSize::new(240.0, 140.0)
        );
        tree.get_mut(root).unwrap().state =
            Widget::from(Popover::new("Changed", LogicalSize::new(20.0, 20.0)));
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.name.as_deref(),
            Some("Formatting")
        );
    }

    #[test]
    fn retained_popover_visual_outputs_match_direction_goldens() {
        use crate::{Popover, TooltipPlacement};

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let fixtures = [
            (Direction::Ltr, 12795508767424887605_u64),
            (Direction::Rtl, 9395518406588260287_u64),
        ];
        for (direction, expected) in fixtures {
            let mut popover = Popover::new("Formatting", LogicalSize::new(100.0, 60.0));
            popover.open = true;
            popover.placement = TooltipPlacement::InlineStart;
            let tree = WidgetTree::new(Widget::from(popover));
            let mut text_system = crate::TextSystem::new();
            let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                WidgetPlacement::new(
                    LogicalPoint::new(130.0, 60.0),
                    LogicalConstraints::tight(LogicalSize::new(240.0, 140.0)),
                    direction,
                )
            });
            let actual = button_visual_digest(&frame);
            assert_eq!(actual, expected, "direction={direction:?} actual={actual}");
        }
    }

    #[test]
    fn retained_modal_freezes_dialog_semantics_and_scrim_panel_order() {
        use crate::{FocusSnapshot, Modal};

        let mut modal = Modal::new("Confirm deletion", LogicalSize::new(180.0, 80.0));
        modal.open = true;
        let mut tree = WidgetTree::new(Widget::from(modal));
        let root = tree.root();
        let focus = FocusSnapshot::build(&tree, |_, widget| widget.focus_policy());
        assert_eq!(focus.tab_order(), &[root]);
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::default(),
                LogicalConstraints::tight(LogicalSize::new(320.0, 200.0)),
                Direction::Rtl,
            )
        });
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.role,
            SemanticRole::Dialog
        );
        assert_eq!(
            frame.geometry.get(root).unwrap().bounds.size,
            LogicalSize::new(320.0, 200.0)
        );
        assert_eq!(frame.rectangles.len(), 2);
        assert_eq!(frame.rectangles[0].position, [0.0, 0.0]);
        assert!(frame.rectangles[1].position[0] > 0.0);
        tree.get_mut(root).unwrap().state =
            Widget::from(Modal::new("Changed", LogicalSize::new(20.0, 20.0)));
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.name.as_deref(),
            Some("Confirm deletion")
        );
    }

    #[test]
    fn retained_modal_visual_outputs_match_direction_goldens() {
        use crate::Modal;

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let fixtures = [
            (Direction::Ltr, 5059536824061134353_u64),
            (Direction::Rtl, 5059536824061134353_u64),
        ];
        for (direction, expected) in fixtures {
            let mut modal = Modal::new("Confirm deletion", LogicalSize::new(180.0, 80.0));
            modal.open = true;
            let tree = WidgetTree::new(Widget::from(modal));
            let mut text_system = crate::TextSystem::new();
            let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                WidgetPlacement::new(
                    LogicalPoint::default(),
                    LogicalConstraints::tight(LogicalSize::new(320.0, 200.0)),
                    direction,
                )
            });
            let actual = button_visual_digest(&frame);
            assert_eq!(actual, expected, "direction={direction:?} actual={actual}");
        }
    }

    #[test]
    fn retained_drawer_freezes_dialog_semantics_and_logical_panel_edge() {
        use crate::{Drawer, FocusSnapshot};

        let mut drawer = Drawer::new("Navigation", 120.0);
        drawer.open = true;
        let mut tree = WidgetTree::new(Widget::from(drawer));
        let root = tree.root();
        let focus = FocusSnapshot::build(&tree, |_, widget| widget.focus_policy());
        assert_eq!(focus.tab_order(), &[root]);
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::default(),
                LogicalConstraints::tight(LogicalSize::new(320.0, 200.0)),
                Direction::Rtl,
            )
        });
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.role,
            SemanticRole::Dialog
        );
        assert_eq!(frame.rectangles.len(), 2);
        assert_eq!(frame.rectangles[0].position, [0.0, 0.0]);
        assert_eq!(frame.rectangles[1].position, [200.0, 0.0]);
        tree.get_mut(root).unwrap().state = Widget::from(Drawer::new("Changed", 20.0));
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.name.as_deref(),
            Some("Navigation")
        );
    }

    #[test]
    fn retained_drawer_visual_outputs_match_direction_goldens() {
        use crate::Drawer;

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let fixtures = [
            (Direction::Ltr, 14862121890869678541_u64),
            (Direction::Rtl, 983306859756351340_u64),
        ];
        for (direction, expected) in fixtures {
            let mut drawer = Drawer::new("Navigation", 120.0);
            drawer.open = true;
            let tree = WidgetTree::new(Widget::from(drawer));
            let mut text_system = crate::TextSystem::new();
            let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                WidgetPlacement::new(
                    LogicalPoint::default(),
                    LogicalConstraints::tight(LogicalSize::new(320.0, 200.0)),
                    direction,
                )
            });
            let actual = button_visual_digest(&frame);
            assert_eq!(actual, expected, "direction={direction:?} actual={actual}");
        }
    }

    #[test]
    fn composed_overlay_children_are_placed_inside_content_bounds() {
        use crate::{Button, Drawer, Modal};

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let viewport = LogicalSize::new(320.0, 200.0);
        let mut modal = Modal::new("Dialog", LogicalSize::new(180.0, 80.0));
        modal.open = true;
        let expected_modal = modal.layout(&theme, viewport).content_bounds.origin;
        let mut modal_tree = WidgetTree::new(Widget::from(modal));
        let modal_child = modal_tree
            .append(modal_tree.root(), Widget::from(Button::new("Confirm")))
            .unwrap();
        let mut text_system = crate::TextSystem::new();
        let modal_frame = WidgetFrame::build_composed(
            &modal_tree,
            &mut text_system,
            &theme,
            WidgetPlacement::new(
                LogicalPoint::default(),
                LogicalConstraints::tight(viewport),
                Direction::Ltr,
            ),
        );
        assert_eq!(
            modal_frame.geometry.get(modal_child).unwrap().bounds.origin,
            expected_modal
        );

        let mut drawer = Drawer::new("Navigation", 160.0);
        drawer.open = true;
        let expected_drawer = drawer
            .layout(&theme, Direction::Rtl, viewport)
            .content_bounds
            .origin;
        let mut drawer_tree = WidgetTree::new(Widget::from(drawer));
        let drawer_child = drawer_tree
            .append(drawer_tree.root(), Widget::from(Button::new("Profile")))
            .unwrap();
        let drawer_frame = WidgetFrame::build_composed(
            &drawer_tree,
            &mut text_system,
            &theme,
            WidgetPlacement::new(
                LogicalPoint::default(),
                LogicalConstraints::tight(viewport),
                Direction::Rtl,
            ),
        );
        assert_eq!(
            drawer_frame
                .geometry
                .get(drawer_child)
                .unwrap()
                .bounds
                .origin,
            expected_drawer
        );
    }

    #[test]
    fn nested_overlay_uses_viewport_constraints_and_translates_paint_origin() {
        use crate::{Button, Modal};

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let viewport = LogicalSize::new(320.0, 200.0);
        let mut tree = WidgetTree::new(Widget::from(crate::Stack));
        let root = tree.root();
        tree.append(root, Widget::from(Button::new("Background")))
            .unwrap();
        let mut modal = Modal::new("Dialog", LogicalSize::new(180.0, 80.0));
        modal.open = true;
        let modal = tree.append(root, Widget::from(modal)).unwrap();
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build_composed(
            &tree,
            &mut text_system,
            &theme,
            WidgetPlacement::new(
                LogicalPoint::new(10.0, 20.0),
                LogicalConstraints::tight(viewport),
                Direction::Ltr,
            ),
        );

        assert_eq!(frame.geometry.get(modal).unwrap().bounds.size, viewport);
        assert_eq!(frame.rectangles[1].position, [10.0, 20.0]);
        assert!(frame.rectangles[2].position[0] > 10.0);
        assert!(frame.rectangles[2].position[1] > 20.0);
    }

    #[test]
    fn closed_overlay_hides_child_geometry_and_focus_subtree() {
        use crate::{Button, FocusSnapshot, Modal};

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let tree = {
            let mut tree = WidgetTree::new(Widget::from(Modal::new(
                "Dialog",
                LogicalSize::new(180.0, 80.0),
            )));
            tree.append(tree.root(), Widget::from(Button::new("Hidden action")))
                .unwrap();
            tree
        };
        let child = tree.get(tree.root()).unwrap().children()[0];
        let focus = FocusSnapshot::build(&tree, |_, widget| widget.focus_policy());
        assert!(focus.tab_order().is_empty());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build_composed(
            &tree,
            &mut text_system,
            &theme,
            WidgetPlacement::new(
                LogicalPoint::default(),
                LogicalConstraints::tight(LogicalSize::new(320.0, 200.0)),
                Direction::Ltr,
            ),
        );
        assert_eq!(
            frame.geometry.get(child).unwrap().bounds.size,
            LogicalSize::default()
        );
    }

    #[test]
    fn button_focus_and_keyboard_activation_follow_enabled_semantics() {
        use crate::{
            Button, FocusSnapshot, IconButton, Key, KeyboardEvent, SemanticAction,
            semantic_action_for_key,
        };

        let mask =
            || Icon::new(PixelImage::new(1, 1, PixelFormat::Alpha8, vec![255]).unwrap()).unwrap();
        let mut tree = WidgetTree::new(Widget::from(Button::new("Save")));
        let root = tree.root();
        let mut disabled = IconButton::new(mask(), "Menu");
        disabled.style.state.disabled = true;
        let disabled = tree.append(root, Widget::from(disabled)).unwrap();
        let focus = FocusSnapshot::build(&tree, |_, widget| widget.focus_policy());
        assert_eq!(focus.tab_order(), &[root]);

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::default(),
                LogicalConstraints::unconstrained(),
                Direction::Ltr,
            )
        });
        let enter = KeyboardEvent::pressed(Key::Enter);
        assert_eq!(
            semantic_action_for_key(&frame.semantics, root, &enter, Direction::Ltr)
                .unwrap()
                .action,
            SemanticAction::Activate
        );
        assert_eq!(
            semantic_action_for_key(&frame.semantics, disabled, &enter, Direction::Ltr),
            None
        );
    }

    #[test]
    fn retained_button_visual_outputs_match_direction_goldens() {
        use crate::{AdornmentPlacement, Button, VisualVariant};

        let mask = || {
            Icon::new(PixelImage::new(2, 1, PixelFormat::Alpha8, vec![255, 96]).unwrap()).unwrap()
        };
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let fixtures = [
            (
                Direction::Ltr,
                VisualVariant::Solid,
                11723903865523465027_u64,
            ),
            (
                Direction::Rtl,
                VisualVariant::Outline,
                14904414208503005046_u64,
            ),
        ];
        for (direction, variant, expected) in fixtures {
            let mut button =
                Button::new("Continue").with_icon(mask(), AdornmentPlacement::InlineEnd);
            button.style.variant = variant;
            let tree = WidgetTree::new(Widget::from(button));
            let mut text_system = crate::TextSystem::new();
            let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                WidgetPlacement::new(
                    LogicalPoint::new(7.0, 11.0),
                    LogicalConstraints::unconstrained(),
                    direction,
                )
            });
            let actual = button_visual_digest(&frame);
            assert_eq!(actual, expected, "direction={direction:?} actual={actual}");
        }
    }

    #[test]
    fn retained_button_visual_matrix_matches_state_theme_direction_and_text_scale_golden() {
        use crate::{Button, ColorScheme, ComponentState, ThemeMode, VisualVariant};

        let states = [
            ComponentState::default(),
            ComponentState {
                hovered: true,
                ..ComponentState::default()
            },
            ComponentState {
                active: true,
                ..ComponentState::default()
            },
            ComponentState {
                focused: true,
                ..ComponentState::default()
            },
            ComponentState {
                disabled: true,
                ..ComponentState::default()
            },
        ];
        let mut golden = 0xcbf29ce484222325_u64;
        for scheme in [ColorScheme::Light, ColorScheme::Dark] {
            for direction in [Direction::Ltr, Direction::Rtl] {
                for text_scale in [1.0, 1.5] {
                    for state in states {
                        let mut controller = ThemeController::default();
                        controller.set_mode(match scheme {
                            ColorScheme::Light => ThemeMode::Light,
                            ColorScheme::Dark => ThemeMode::Dark,
                        });
                        let mut preferences = UserPreferences::default();
                        preferences.set_text_scale(text_scale);
                        let theme = ThemeDefinition::default().resolve(controller, preferences);
                        let mut button = Button::new("Continue");
                        button.style.variant = VisualVariant::Solid;
                        button.style.state = state;
                        let tree = WidgetTree::new(Widget::from(button));
                        let mut text_system = crate::TextSystem::new();
                        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                            WidgetPlacement::new(
                                LogicalPoint::new(7.0, 11.0),
                                LogicalConstraints::unconstrained(),
                                direction,
                            )
                        });
                        golden ^= button_visual_digest(&frame);
                        golden = golden.wrapping_mul(0x100000001b3);
                    }
                }
            }
        }

        assert_eq!(golden, 18327075761226786813);
    }

    #[test]
    fn retained_spacer_and_divider_freeze_geometry_and_theme_resolved_paint() {
        use crate::{Divider, Spacer};

        let mut tree = WidgetTree::new(Widget::from(Spacer::new(LogicalSize::new(12.0, 8.0))));
        let root = tree.root();
        let _divider = tree
            .append(root, Widget::from(Divider::vertical()))
            .unwrap();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
            WidgetPlacement::new(
                LogicalPoint::new(if id == root { 4.0 } else { 20.0 }, 6.0),
                LogicalConstraints::tight(LogicalSize::new(12.0, 8.0)),
                Direction::Ltr,
            )
        });

        assert_eq!(
            frame.geometry.get(root).unwrap().bounds.size,
            LogicalSize::new(12.0, 8.0)
        );
        assert_eq!(frame.semantics.get(root).unwrap().semantics.name, None);
        assert_eq!(frame.rectangles.len(), 1);
        assert_eq!(frame.rectangles[0].position, [20.0, 6.0]);
        assert_eq!(frame.rectangles[0].color, theme.colors.border.to_array());
    }

    #[test]
    fn retained_surface_freezes_theme_resolved_rectangle() {
        use crate::Surface;
        let tree = WidgetTree::new(Widget::from(Surface::new(LogicalSize::new(20.0, 10.0))));
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(3.0, 4.0),
                LogicalConstraints::unconstrained(),
                Direction::Ltr,
            )
        });
        assert_eq!(frame.rectangles[0].position, [3.0, 4.0]);
        assert_eq!(frame.rectangles[0].color, theme.colors.surface.to_array());
    }

    #[test]
    fn retained_container_freezes_constraint_resolved_geometry_without_paint() {
        use crate::{Container, LogicalRect};
        let tree = WidgetTree::new(Widget::from(Container::new(LogicalSize::new(32.0, 18.0))));
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(5.0, 7.0),
                LogicalConstraints::tight(LogicalSize::new(20.0, 10.0)),
                Direction::Ltr,
            )
        });
        assert_eq!(
            frame.geometry.get(tree.root()).unwrap().bounds,
            LogicalRect::new(LogicalPoint::new(5.0, 7.0), LogicalSize::new(20.0, 10.0))
        );
        assert!(frame.rectangles.is_empty());
    }

    #[test]
    fn composed_row_places_retained_children_and_mirrors_geometry_in_rtl() {
        use crate::{Row, Spacer};

        let mut tree = WidgetTree::new(Widget::from(Row::default()));
        let root = tree.root();
        let first = tree
            .append(root, Widget::from(Spacer::new(LogicalSize::new(10.0, 4.0))))
            .unwrap();
        let second = tree
            .append(root, Widget::from(Spacer::new(LogicalSize::new(6.0, 8.0))))
            .unwrap();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build_composed(
            &tree,
            &mut text_system,
            &theme,
            WidgetPlacement::new(
                LogicalPoint::new(5.0, 7.0),
                LogicalConstraints::unconstrained(),
                Direction::Rtl,
            ),
        );

        assert_eq!(frame.geometry.paint_order(), &[root, first, second]);
        assert_eq!(
            frame.geometry.get(root).unwrap().bounds.size,
            LogicalSize::new(16.0, 8.0)
        );
        assert_eq!(
            frame.geometry.get(first).unwrap().bounds.origin,
            LogicalPoint::new(11.0, 7.0)
        );
        assert_eq!(
            frame.geometry.get(second).unwrap().bounds.origin,
            LogicalPoint::new(5.0, 7.0)
        );
    }

    #[test]
    fn composed_stack_overlays_children_in_stable_paint_order() {
        use crate::{Spacer, Stack};

        let mut tree = WidgetTree::new(Widget::from(Stack));
        let root = tree.root();
        let back = tree
            .append(root, Widget::from(Spacer::new(LogicalSize::new(10.0, 8.0))))
            .unwrap();
        let front = tree
            .append(root, Widget::from(Spacer::new(LogicalSize::new(4.0, 3.0))))
            .unwrap();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build_composed(
            &tree,
            &mut text_system,
            &theme,
            WidgetPlacement::new(
                LogicalPoint::new(2.0, 3.0),
                LogicalConstraints::unconstrained(),
                Direction::Ltr,
            ),
        );

        assert_eq!(frame.geometry.paint_order(), &[root, back, front]);
        assert_eq!(
            frame.geometry.get(back).unwrap().bounds.origin,
            LogicalPoint::new(2.0, 3.0)
        );
        assert_eq!(
            frame.geometry.get(front).unwrap().bounds.origin,
            LogicalPoint::new(2.0, 3.0)
        );
    }

    #[test]
    fn composed_scroll_view_offsets_content_and_clips_descendants() {
        use crate::{ClipRegion, LogicalRect, ScrollOffset, ScrollView, Spacer};

        let scroll = ScrollView {
            viewport: Some(LogicalSize::new(20.0, 10.0)),
            offset: ScrollOffset::new(5.0, 0.0),
            ..ScrollView::default()
        };
        let mut tree = WidgetTree::new(Widget::from(scroll));
        let root = tree.root();
        let content = tree
            .append(
                root,
                Widget::from(Spacer::new(LogicalSize::new(40.0, 10.0))),
            )
            .unwrap();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build_composed(
            &tree,
            &mut text_system,
            &theme,
            WidgetPlacement::new(
                LogicalPoint::new(10.0, 12.0),
                LogicalConstraints::unconstrained(),
                Direction::Ltr,
            ),
        );

        let root_bounds =
            LogicalRect::new(LogicalPoint::new(10.0, 12.0), LogicalSize::new(20.0, 10.0));
        assert_eq!(frame.geometry.get(root).unwrap().bounds, root_bounds);
        assert_eq!(
            frame.geometry.get(content).unwrap().bounds.origin,
            LogicalPoint::new(5.0, 12.0)
        );
        assert_eq!(
            frame.geometry.get(content).unwrap().clip,
            ClipRegion::Rect(root_bounds)
        );
    }

    #[test]
    fn retained_image_and_icon_visual_outputs_match_goldens() {
        let image = PixelImage::new(
            3,
            2,
            PixelFormat::Rgba8,
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255,
                255, 0, 255, 255,
            ],
        )
        .unwrap();
        let mut icon =
            Icon::new(PixelImage::new(2, 2, PixelFormat::Alpha8, vec![0, 255, 255, 0]).unwrap())
                .unwrap();
        icon.mirror_in_rtl = true;
        icon.color = SemanticColorToken::Primary;
        let mut tree = WidgetTree::new(Widget::from(Image::new(image)));
        let root = tree.root();
        tree.append(root, Widget::from(icon)).unwrap();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
            WidgetPlacement::new(
                LogicalPoint::new(if id == root { 2.0 } else { 17.0 }, 3.0),
                LogicalConstraints::tight(LogicalSize::new(10.0, 8.0)),
                Direction::Rtl,
            )
        });
        assert_eq!(image_visual_digest(&frame), 9240581001633210751);
    }

    #[test]
    fn retained_text_visual_outputs_match_bundled_font_goldens() {
        let fixtures = [
            (
                "Mio-GUI text",
                Direction::Ltr,
                240.0,
                7466596128845657332_u64,
            ),
            (
                "رابط کاربری",
                Direction::Rtl,
                240.0,
                9441082396134697323_u64,
            ),
            (
                "نسخه Mio-GUI 2",
                Direction::Rtl,
                240.0,
                9807824100324377841_u64,
            ),
            (
                "متن بلند برای شکستن سطرها",
                Direction::Rtl,
                82.0,
                17325006582298125765_u64,
            ),
        ];

        for (content, direction, width, expected) in fixtures {
            let (frame, mut text_system) = text_frame(content, direction, width);
            let actual = visual_digest(&frame, &mut text_system);
            assert_eq!(actual, expected, "content={content:?} actual={actual}");
        }
    }
}
