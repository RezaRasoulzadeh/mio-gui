// widget_app.rs

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Ime, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::ModifiersState;
use winit::window::{Window, WindowId};

use crate::{
    Direction, FocusIndicatorStyle, FocusManager, FocusSnapshot, Key, LogicalConstraints,
    LogicalPoint, LogicalSize, PlatformAccessibility, RectDraw, Renderer, ScaleFactor,
    SemanticAction, SystemClipboard, TextSystem, ThemeController, ThemeDefinition, UserPreferences,
    Widget, WidgetFrame, WidgetPlacement, WidgetTree, apply_focus_navigation, apply_winit_theme,
    keyboard_event_from_winit,
};

#[cfg(target_os = "macos")]
const SETTLE_REDRAWS: u8 = 3;
#[cfg(not(target_os = "macos"))]
const SETTLE_REDRAWS: u8 = 1;

pub fn run_widget_tree(tree: WidgetTree<Widget>, direction: Direction) {
    let event_loop = match EventLoop::new() {
        Ok(event_loop) => event_loop,
        Err(error) => {
            eprintln!("Mio-GUI event loop creation failed: {error}");
            return;
        }
    };
    let mut app = WidgetApp::new(tree, direction);
    if let Err(error) = event_loop.run_app(&mut app) {
        eprintln!("Mio-GUI event loop failed: {error}");
    }
}

struct WidgetApp {
    tree: WidgetTree<Widget>,
    direction: Direction,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    accessibility: Option<PlatformAccessibility>,
    frame: Option<WidgetFrame>,
    text_system: TextSystem,
    focus: FocusManager,
    modifiers: ModifiersState,
    cursor: Option<LogicalPoint>,
    pending_size: Option<PhysicalSize<u32>>,
    redraws_remaining: u8,
    theme: ThemeController,
    clipboard: Option<SystemClipboard>,
    pointer_target: Option<(crate::WidgetId, Option<usize>)>,
    active_modal: Option<crate::WidgetId>,
    focus_before_modal: Option<crate::WidgetId>,
}

impl WidgetApp {
    fn new(tree: WidgetTree<Widget>, direction: Direction) -> Self {
        Self {
            tree,
            direction,
            window: None,
            renderer: None,
            accessibility: None,
            frame: None,
            text_system: TextSystem::new(),
            focus: FocusManager::default(),
            modifiers: ModifiersState::empty(),
            cursor: None,
            pending_size: None,
            redraws_remaining: 0,
            theme: ThemeController::default(),
            clipboard: None,
            pointer_target: None,
            active_modal: None,
            focus_before_modal: None,
        }
    }

    fn rebuild(&mut self, size: PhysicalSize<u32>) {
        self.reconcile_modal_focus();
        self.synchronize_focus_state();
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        let scale = renderer.scale_factor().max(f32::EPSILON);
        let viewport = LogicalSize::new(size.width as f32 / scale, size.height as f32 / scale);
        let theme = ThemeDefinition::default().resolve(self.theme, UserPreferences::default());
        let mut frame = WidgetFrame::build_composed(
            &self.tree,
            &mut self.text_system,
            &theme,
            WidgetPlacement::new(
                LogicalPoint::new(24.0, 24.0),
                LogicalConstraints::loose(LogicalSize::new(
                    (viewport.width - 48.0).max(0.0),
                    (viewport.height - 48.0).max(0.0),
                )),
                self.direction,
            ),
        );
        if let Some(indicator) = self.focus.indicator(
            &frame.geometry,
            FocusIndicatorStyle {
                color: theme.colors.focus.to_array(),
                width: theme.borders.thick,
                offset: 1.0,
                radius: theme.radii.medium + theme.borders.thick + 1.0,
            },
        ) {
            frame.rectangles.push(RectDraw {
                position: [indicator.bounds.origin.x, indicator.bounds.origin.y],
                size: [indicator.bounds.size.width, indicator.bounds.size.height],
                radii: [indicator.radius; 4],
                color: [0.0; 4],
                border_width: indicator.width,
                border_color: indicator.color,
            });
        }
        renderer.set_widget_frame(&frame);
        renderer.set_clear_color(theme.colors.background);
        if let Some(accessibility) = self.accessibility.as_mut() {
            accessibility.update(
                &frame.semantics,
                &frame.geometry,
                ScaleFactor::new(scale).unwrap(),
                self.focus.focused(),
            );
        }
        self.frame = Some(frame);
    }

    fn handle_keyboard(&mut self, event: &winit::event::KeyEvent) {
        let event = keyboard_event_from_winit(event, self.modifiers);
        self.handle_keyboard_event(event);
    }

    fn handle_keyboard_event(&mut self, event: crate::KeyboardEvent) {
        if event.state != crate::KeyState::Pressed {
            return;
        }
        if !event.repeat && event.key == Key::Escape && self.dismiss_top_overlay() {
            self.request_redraw();
            return;
        }
        let snapshot = self.focus_snapshot();
        if event.key == Key::Tab && !event.repeat {
            apply_focus_navigation(&mut self.focus, &snapshot, &event, self.direction);
        } else if let Some(focused) = self.focus.focused() {
            if !event.repeat
                && matches!(event.key, Key::Space | Key::Enter)
                && matches!(
                    self.tree.get(focused).map(|node| &node.state),
                    Some(Widget::Radio(_))
                )
            {
                self.tree.select_radio(focused);
                self.request_redraw();
                return;
            }
            if matches!(event.key, Key::Arrow(_))
                && matches!(
                    self.tree.get(focused).map(|node| &node.state),
                    Some(Widget::Radio(_))
                )
            {
                let forward = matches!(
                    event.key,
                    Key::Arrow(crate::ArrowKey::Down) | Key::Arrow(crate::ArrowKey::Right)
                );
                self.move_radio_group(focused, forward);
                self.request_redraw();
                return;
            }
            if !event.repeat && self.handle_text_shortcut(focused, &event) {
                self.request_redraw();
                return;
            }
            let focused_size = self
                .frame
                .as_ref()
                .and_then(|frame| frame.geometry.get(focused))
                .map(|node| node.bounds.size);
            let Some(node) = self.tree.get_mut(focused) else {
                return;
            };
            let mut editing_event = event.clone();
            if self.direction == Direction::Rtl {
                editing_event.key = match editing_event.key {
                    Key::Arrow(crate::ArrowKey::Left) => Key::Arrow(crate::ArrowKey::Right),
                    Key::Arrow(crate::ArrowKey::Right) => Key::Arrow(crate::ArrowKey::Left),
                    key => key,
                };
            }
            let status = match &mut node.state {
                Widget::TextInput(input) => {
                    input.handle_key(&editing_event);
                    None
                }
                Widget::TextArea(area) => {
                    if matches!(
                        editing_event.key,
                        Key::Arrow(crate::ArrowKey::Up | crate::ArrowKey::Down)
                            | Key::Home
                            | Key::End
                    ) {
                        let theme = ThemeDefinition::default()
                            .resolve(self.theme, UserPreferences::default());
                        let layout = area.layout(
                            &mut self.text_system,
                            &theme,
                            self.direction,
                            LogicalConstraints::tight(focused_size.unwrap_or_default()),
                        );
                        area.handle_key_with_layout(&editing_event, &layout);
                    } else {
                        area.handle_key(&editing_event);
                    }
                    None
                }
                Widget::SearchInput(search) => {
                    if !event.repeat || !matches!(event.key, Key::Enter | Key::Escape) {
                        search.handle_key(&editing_event);
                    }
                    None
                }
                Widget::Checkbox(checkbox)
                    if !event.repeat && matches!(event.key, Key::Space | Key::Enter) =>
                {
                    checkbox.activate();
                    Some(format!(
                        "Receive updates: {}",
                        if checkbox.checked {
                            "checked"
                        } else {
                            "not checked"
                        }
                    ))
                }
                Widget::Switch(switch)
                    if !event.repeat && matches!(event.key, Key::Space | Key::Enter) =>
                {
                    switch.activate();
                    Some(format!(
                        "{}: {}",
                        switch.label(),
                        if switch.checked { "on" } else { "off" }
                    ))
                }
                Widget::Slider(slider) => {
                    let changed = slider.handle_key(&event, self.direction);
                    changed.then(|| format!("{}: {}", slider.label(), slider.value()))
                }
                Widget::Button(button)
                    if !event.repeat && matches!(event.key, Key::Space | Key::Enter) =>
                {
                    eprintln!("Mio-GUI activated {}", button.label());
                    Some(format!("{} activated", button.label()))
                }
                Widget::IconButton(button)
                    if !event.repeat && matches!(event.key, Key::Space | Key::Enter) =>
                {
                    eprintln!("Mio-GUI activated {}", button.label());
                    Some(format!("{} activated", button.label()))
                }
                Widget::Select(select) => {
                    let action = select.handle_key(&event);
                    (action != crate::SelectAction::None)
                        .then(|| format!("{}: {}", select.label(), select.selected().label))
                }
                Widget::Menu(menu) => {
                    let action = menu.handle_key(&event);
                    (action != crate::MenuAction::None)
                        .then(|| format!("{}: item {}", menu.label(), menu.active_index() + 1))
                }
                Widget::Dropdown(dropdown) => {
                    let action = dropdown.handle_key(&event);
                    (action != crate::DropdownAction::None).then(|| {
                        format!(
                            "{}: {}",
                            dropdown.trigger.label(),
                            if dropdown.open { "open" } else { "closed" }
                        )
                    })
                }
                Widget::ContextMenu(menu) => {
                    menu.handle_key(&event);
                    None
                }
                Widget::Popover(popover) => {
                    popover.handle_key(&event);
                    None
                }
                Widget::Modal(modal) => {
                    modal.handle_key(&event);
                    None
                }
                Widget::Drawer(drawer) => {
                    drawer.handle_key(&event);
                    None
                }
                _ => None,
            };
            if let Some(status) = status {
                self.set_status(&status);
            }
        }
        self.request_redraw();
    }

    fn request_redraw(&mut self) {
        self.redraws_remaining = SETTLE_REDRAWS;
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn active_modal(&self) -> Option<crate::WidgetId> {
        self.tree
            .depth_first(self.tree.root())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .find(|id| match &self.tree.get(*id).unwrap().state {
                Widget::Modal(modal) => modal.open,
                Widget::Drawer(drawer) => drawer.open,
                _ => false,
            })
    }

    fn top_open_overlay(&self) -> Option<crate::WidgetId> {
        self.tree
            .depth_first(self.tree.root())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .find(|id| match &self.tree.get(*id).unwrap().state {
                Widget::ContextMenu(menu) => menu.open,
                Widget::Popover(popover) => popover.open,
                Widget::Modal(modal) => modal.open,
                Widget::Drawer(drawer) => drawer.open,
                _ => false,
            })
    }

    fn dismiss_top_overlay(&mut self) -> bool {
        let Some(overlay) = self.top_open_overlay() else {
            return false;
        };
        match &mut self.tree.get_mut(overlay).unwrap().state {
            Widget::ContextMenu(menu) => menu.dismiss(),
            Widget::Popover(popover) => popover.dismiss() == crate::PopoverAction::Dismissed,
            Widget::Modal(modal) => modal.dismiss() == crate::ModalAction::Dismissed,
            Widget::Drawer(drawer) => drawer.dismiss() == crate::DrawerAction::Dismissed,
            _ => false,
        }
    }

    fn focus_snapshot(&self) -> FocusSnapshot {
        let modal = self.active_modal();
        FocusSnapshot::build(&self.tree, |id, widget| {
            let mut policy = widget.focus_policy();
            if let Widget::Radio(radio) = widget {
                if let Some(group) = radio.group() {
                    policy.skip_tab_order = self.tree.radio_tab_stop(group) != Some(id);
                }
            }
            if let Some(modal) = modal {
                let related = id == modal
                    || self.tree.ancestors(id).any(|ancestor| ancestor == modal)
                    || self.tree.ancestors(modal).any(|ancestor| ancestor == id);
                policy.inert |= !related;
            }
            policy
        })
    }

    fn reconcile_modal_focus(&mut self) {
        let next = self.active_modal();
        if self.active_modal.is_none() && next.is_some() {
            self.focus_before_modal = self.focus.focused();
            let snapshot = self.focus_snapshot();
            self.focus
                .traverse(&snapshot, crate::FocusTraversal::Forward);
        } else if self.active_modal.is_some() && next.is_none() {
            let snapshot = self.focus_snapshot();
            if let Some(previous) = self.focus_before_modal.take() {
                if !self.focus.focus(&snapshot, previous) {
                    self.focus.reconcile(&snapshot);
                }
            } else {
                self.focus.reconcile(&snapshot);
            }
        }
        self.active_modal = next;
    }

    fn set_status(&self, status: &str) {
        if let Some(window) = &self.window {
            window.set_title(&format!("Mio-GUI representative form — {status}"));
        }
    }

    fn handle_text_shortcut(
        &mut self,
        focused: crate::WidgetId,
        event: &crate::KeyboardEvent,
    ) -> bool {
        let primary = if cfg!(target_os = "macos") {
            event.modifiers.meta
        } else {
            event.modifiers.control
        };
        if !primary {
            return false;
        }
        let Key::Character(character) = &event.key else {
            return false;
        };
        let Some(node) = self.tree.get_mut(focused) else {
            return false;
        };
        let input = match &mut node.state {
            Widget::TextInput(input) => input,
            Widget::TextArea(area) => &mut area.input,
            Widget::SearchInput(search) => &mut search.input,
            _ => return false,
        };
        if character.eq_ignore_ascii_case("a") {
            input.edit.select_all();
            return true;
        }
        let Some(clipboard) = self.clipboard.as_mut() else {
            return false;
        };
        let result = if character.eq_ignore_ascii_case("c") {
            input.edit.copy_to(clipboard)
        } else if character.eq_ignore_ascii_case("x") {
            input.edit.cut_to(clipboard)
        } else if character.eq_ignore_ascii_case("v") {
            input.edit.paste_from(clipboard)
        } else {
            return false;
        };
        if let Err(error) = result {
            eprintln!("Mio-GUI clipboard error: {error}");
        }
        true
    }

    fn handle_ime(&mut self, ime: Ime) {
        let Some(focused) = self.focus.focused() else {
            return;
        };
        let Some(node) = self.tree.get_mut(focused) else {
            return;
        };
        let input = match &mut node.state {
            Widget::TextInput(input) => input,
            Widget::TextArea(area) => &mut area.input,
            Widget::SearchInput(search) => &mut search.input,
            _ => return,
        };
        match ime {
            Ime::Enabled => {}
            Ime::Preedit(text, selection) => input
                .edit
                .update_composition_with_selection(&text, selection.map(|(start, end)| start..end)),
            Ime::Commit(text) => {
                if input.edit.composition_range().is_some() {
                    input.edit.update_composition(&text);
                    input.edit.commit_composition();
                } else {
                    input.edit.paste(&text);
                }
            }
            Ime::Disabled => input.edit.commit_composition(),
        }
        self.request_redraw();
    }

    fn synchronize_focus_state(&mut self) {
        let focused = self.focus.focused();
        let ids = self.tree.depth_first(self.tree.root()).collect::<Vec<_>>();
        for id in ids {
            let Some(node) = self.tree.get_mut(id) else {
                continue;
            };
            match &mut node.state {
                Widget::TextInput(input) => input.focused = focused == Some(id),
                Widget::TextArea(area) => area.input.focused = focused == Some(id),
                Widget::SearchInput(search) => search.input.focused = focused == Some(id),
                Widget::Button(button) => button.style.state.focused = focused == Some(id),
                Widget::IconButton(button) => button.style.state.focused = focused == Some(id),
                Widget::Select(select) if focused.is_some() && focused != Some(id) => {
                    select.open = false;
                }
                Widget::Dropdown(dropdown) if focused.is_some() && focused != Some(id) => {
                    dropdown.open = false;
                }
                _ => {}
            }
        }
    }

    fn activate_target(&mut self, target: crate::WidgetId) -> bool {
        if matches!(
            self.tree.get(target).map(|node| &node.state),
            Some(Widget::Radio(_))
        ) {
            return self.tree.select_radio(target);
        }
        let Some(node) = self.tree.get_mut(target) else {
            return false;
        };
        let (activated, status) = match &mut node.state {
            Widget::Checkbox(checkbox) => {
                let activated = checkbox.activate();
                (
                    activated,
                    activated.then(|| {
                        format!(
                            "{}: {}",
                            checkbox.label(),
                            if checkbox.checked {
                                "checked"
                            } else {
                                "not checked"
                            }
                        )
                    }),
                )
            }
            Widget::Switch(switch) => {
                let activated = switch.activate();
                (
                    activated,
                    activated.then(|| {
                        format!(
                            "{}: {}",
                            switch.label(),
                            if switch.checked { "on" } else { "off" }
                        )
                    }),
                )
            }
            Widget::Button(button) if !button.style.state.disabled => {
                eprintln!("Mio-GUI activated {}", button.label());
                (true, Some(format!("{} activated", button.label())))
            }
            Widget::IconButton(button) if !button.style.state.disabled => {
                eprintln!("Mio-GUI activated {}", button.label());
                (true, Some(format!("{} activated", button.label())))
            }
            Widget::Select(select) => {
                let activated = select.handle_key(&crate::KeyboardEvent::pressed(Key::Space))
                    != crate::SelectAction::None;
                (
                    activated,
                    activated.then(|| format!("{}: open", select.label())),
                )
            }
            Widget::Dropdown(dropdown) => {
                let activated = dropdown.handle_key(&crate::KeyboardEvent::pressed(Key::Space))
                    != crate::DropdownAction::None;
                (
                    activated,
                    activated.then(|| format!("{}: open", dropdown.trigger.label())),
                )
            }
            _ => (false, None),
        };
        if let Some(status) = status {
            self.set_status(&status);
        }
        activated
    }

    fn move_radio_group(&mut self, current: crate::WidgetId, forward: bool) -> bool {
        let Some(next) = self.tree.adjacent_radio(current, forward) else {
            return false;
        };
        self.tree.select_radio(next);
        let snapshot = self.focus_snapshot();
        self.focus.focus(&snapshot, next)
    }

    fn handle_pointer_press(&mut self) {
        let Some(point) = self.cursor else {
            return;
        };
        let Some(target) = self
            .frame
            .as_ref()
            .and_then(|frame| frame.geometry.hit_test(point))
        else {
            return;
        };
        if self.dismiss_overlay_scrim(target, point) {
            self.request_redraw();
            return;
        }
        let snapshot = self.focus_snapshot();
        if self.focus.focus(&snapshot, target) {
            let activate_after_update = !matches!(
                &self.tree.get(target).unwrap().state,
                Widget::Dropdown(dropdown) if dropdown.open
            ) && !matches!(
                &self.tree.get(target).unwrap().state,
                Widget::Select(select) if select.open
            );
            let anchor = self.update_pointer_target(target, None, point);
            self.pointer_target = Some((target, anchor));
            if activate_after_update {
                self.activate_target(target);
            }
            self.request_redraw();
        }
    }

    fn dismiss_overlay_scrim(&mut self, target: crate::WidgetId, point: LogicalPoint) -> bool {
        let Some(bounds) = self
            .frame
            .as_ref()
            .and_then(|frame| frame.geometry.get(target))
            .map(|node| node.bounds)
        else {
            return false;
        };
        let theme = ThemeDefinition::default().resolve(self.theme, UserPreferences::default());
        let state = &mut self.tree.get_mut(target).unwrap().state;
        match state {
            Widget::Modal(modal) if modal.open => {
                let layout = modal.layout(&theme, bounds.size);
                let panel = crate::LogicalRect::new(
                    LogicalPoint::new(
                        bounds.origin.x + layout.panel_origin.x,
                        bounds.origin.y + layout.panel_origin.y,
                    ),
                    layout.panel_size,
                );
                !panel.contains(point) && modal.dismiss() == crate::ModalAction::Dismissed
            }
            Widget::Drawer(drawer) if drawer.open => {
                let layout = drawer.layout(&theme, self.direction, bounds.size);
                let panel = crate::LogicalRect::new(
                    LogicalPoint::new(
                        bounds.origin.x + layout.panel_origin.x,
                        bounds.origin.y + layout.panel_origin.y,
                    ),
                    layout.panel_size,
                );
                !panel.contains(point) && drawer.dismiss() == crate::DrawerAction::Dismissed
            }
            Widget::Popover(popover) if popover.open => {
                let layout = popover.layout(
                    &theme,
                    self.direction,
                    crate::LogicalRect::new(bounds.origin, LogicalSize::default()),
                    bounds.size,
                );
                let panel = crate::LogicalRect::new(layout.panel_origin, layout.panel_size);
                !panel.contains(point) && popover.dismiss() == crate::PopoverAction::Dismissed
            }
            Widget::ContextMenu(menu) if menu.open => {
                let layout = menu.layout(
                    &mut self.text_system,
                    &theme,
                    self.direction,
                    LogicalConstraints::tight(bounds.size),
                );
                let panel = crate::LogicalRect::new(
                    LogicalPoint::new(
                        bounds.origin.x + layout.menu_origin.x,
                        bounds.origin.y + layout.menu_origin.y,
                    ),
                    layout.menu.size,
                );
                !panel.contains(point) && menu.dismiss()
            }
            _ => false,
        }
    }

    fn handle_pointer_drag(&mut self) {
        let (Some((target, anchor)), Some(point)) = (self.pointer_target, self.cursor) else {
            return;
        };
        self.update_pointer_target(target, anchor, point);
        self.request_redraw();
    }

    fn update_pointer_target(
        &mut self,
        target: crate::WidgetId,
        anchor: Option<usize>,
        point: LogicalPoint,
    ) -> Option<usize> {
        let bounds = self
            .frame
            .as_ref()?
            .geometry
            .get(target)
            .map(|node| node.bounds)?;
        let window = self.window.clone();
        let theme = ThemeDefinition::default().resolve(self.theme, UserPreferences::default());
        let node = self.tree.get_mut(target)?;
        match &mut node.state {
            Widget::TextInput(input) => {
                let layout = input.layout(
                    &mut self.text_system,
                    &theme,
                    self.direction,
                    LogicalConstraints::tight(bounds.size),
                );
                let caret = layout.hit_test(input, bounds.origin, point);
                if let Some(anchor) = anchor {
                    input.edit.set_selection_from_anchor(anchor, caret);
                } else {
                    input.edit.set_caret(caret);
                }
                Some(anchor.unwrap_or(caret))
            }
            Widget::TextArea(area) => {
                let layout = area.layout(
                    &mut self.text_system,
                    &theme,
                    self.direction,
                    LogicalConstraints::tight(bounds.size),
                );
                let caret = layout.hit_test(&area.input, bounds.origin, point);
                if let Some(anchor) = anchor {
                    area.input.edit.set_selection_from_anchor(anchor, caret);
                } else {
                    area.input.edit.set_caret(caret);
                }
                Some(anchor.unwrap_or(caret))
            }
            Widget::SearchInput(search) => {
                let layout = search.layout(
                    &mut self.text_system,
                    &theme,
                    self.direction,
                    LogicalConstraints::tight(bounds.size),
                );
                let input_origin = LogicalPoint::new(
                    bounds.origin.x + layout.input_offset.x,
                    bounds.origin.y + layout.input_offset.y,
                );
                let caret = layout.input.hit_test(&search.input, input_origin, point);
                if let Some(anchor) = anchor {
                    search.input.edit.set_selection_from_anchor(anchor, caret);
                } else {
                    search.input.edit.set_caret(caret);
                }
                Some(anchor.unwrap_or(caret))
            }
            Widget::Select(select) if select.open => {
                let layout = select.layout(
                    &mut self.text_system,
                    &theme,
                    self.direction,
                    LogicalConstraints::tight(bounds.size),
                );
                let menu_origin = LogicalPoint::new(
                    bounds.origin.x + layout.menu_origin.x,
                    bounds.origin.y + layout.menu_origin.y,
                );
                if let Some(index) = layout.menu.hit_test(menu_origin, point) {
                    if select.select(index).is_ok() {
                        select.open = false;
                    }
                }
                None
            }
            Widget::Menu(menu) => {
                let layout = menu.layout(
                    &mut self.text_system,
                    &theme,
                    self.direction,
                    LogicalConstraints::tight(bounds.size),
                );
                if let Some(index) = layout.hit_test(bounds.origin, point) {
                    menu.activate(index);
                }
                None
            }
            Widget::Dropdown(dropdown) if dropdown.open => {
                let layout = dropdown.layout(
                    &mut self.text_system,
                    &theme,
                    self.direction,
                    LogicalConstraints::tight(bounds.size),
                );
                let menu_origin = LogicalPoint::new(
                    bounds.origin.x + layout.menu_origin.x,
                    bounds.origin.y + layout.menu_origin.y,
                );
                if let Some(index) = layout.menu.hit_test(menu_origin, point) {
                    if dropdown.menu.activate(index) == crate::MenuAction::Activated(index) {
                        dropdown.open = false;
                    }
                }
                None
            }
            Widget::ContextMenu(menu) if menu.open => {
                let layout = menu.layout(
                    &mut self.text_system,
                    &theme,
                    self.direction,
                    LogicalConstraints::tight(bounds.size),
                );
                let menu_origin = LogicalPoint::new(
                    bounds.origin.x + layout.menu_origin.x,
                    bounds.origin.y + layout.menu_origin.y,
                );
                if let Some(index) = layout.menu.hit_test(menu_origin, point) {
                    if menu.menu.activate(index) == crate::MenuAction::Activated(index) {
                        menu.open = false;
                    }
                }
                None
            }
            Widget::Slider(slider) => {
                let mut fraction =
                    ((point.x - bounds.origin.x) / bounds.size.width).clamp(0.0, 1.0);
                if self.direction == Direction::Rtl {
                    fraction = 1.0 - fraction;
                }
                let range = slider.range();
                let value = *range.start() + (*range.end() - *range.start()) * fraction;
                let _ = slider.set_value(value);
                if let Some(window) = window {
                    window.set_title(&format!(
                        "Mio-GUI representative form — {}: {}",
                        slider.label(),
                        slider.value()
                    ));
                }
                None
            }
            _ => None,
        }
    }

    fn handle_accessibility_actions(&mut self) {
        let Some(frame) = self.frame.as_ref() else {
            return;
        };
        let actions = self
            .accessibility
            .as_ref()
            .map(|adapter| adapter.drain_actions(&frame.semantics))
            .unwrap_or_default();
        for request in actions {
            self.apply_semantic_action(request);
        }
    }

    fn apply_semantic_action(&mut self, request: crate::SemanticActionRequest) -> bool {
        match request.action {
            SemanticAction::Focus => {
                let snapshot = self.focus_snapshot();
                self.focus.focus(&snapshot, request.target)
            }
            SemanticAction::Blur if self.focus.focused() == Some(request.target) => {
                self.focus.clear();
                true
            }
            SemanticAction::Blur => false,
            SemanticAction::Activate => match request.value {
                Some(crate::SemanticActionValue::Index(index)) => self
                    .tree
                    .get_mut(request.target)
                    .is_some_and(|node| match &mut node.state {
                        Widget::Menu(menu) => {
                            menu.activate(index) == crate::MenuAction::Activated(index)
                        }
                        Widget::Select(select) => {
                            let activated = select.select(index).is_ok();
                            if activated {
                                select.open = false;
                            }
                            activated
                        }
                        Widget::Dropdown(dropdown) => {
                            let activated = dropdown.menu.activate(index)
                                == crate::MenuAction::Activated(index);
                            if activated {
                                dropdown.open = false;
                            }
                            activated
                        }
                        Widget::ContextMenu(menu) => {
                            let activated =
                                menu.menu.activate(index) == crate::MenuAction::Activated(index);
                            if activated {
                                menu.open = false;
                            }
                            activated
                        }
                        _ => false,
                    }),
                _ => self.activate_target(request.target),
            },
            SemanticAction::ShowMenu => self.activate_target(request.target),
            SemanticAction::Increment | SemanticAction::Decrement => self
                .tree
                .get_mut(request.target)
                .and_then(|node| match &mut node.state {
                    Widget::Slider(slider) if request.action == SemanticAction::Increment => {
                        Some(slider.increment())
                    }
                    Widget::Slider(slider) => Some(slider.decrement()),
                    _ => None,
                })
                .unwrap_or(false),
            SemanticAction::SetValue => {
                let Some(node) = self.tree.get_mut(request.target) else {
                    return false;
                };
                match (&mut node.state, request.value) {
                    (Widget::TextInput(input), Some(crate::SemanticActionValue::Text(value))) => {
                        input.set_text(value)
                    }
                    (Widget::TextArea(area), Some(crate::SemanticActionValue::Text(value))) => {
                        area.input.set_text(value)
                    }
                    (
                        Widget::SearchInput(search),
                        Some(crate::SemanticActionValue::Text(value)),
                    ) => search.input.set_text(value),
                    (Widget::Slider(slider), Some(crate::SemanticActionValue::Number(value))) => {
                        slider.set_value(value as f32).unwrap_or(false)
                    }
                    _ => false,
                }
            }
            SemanticAction::SetTextSelection => {
                let Some(crate::SemanticActionValue::TextSelection { anchor, caret }) =
                    request.value
                else {
                    return false;
                };
                let Some(node) = self.tree.get_mut(request.target) else {
                    return false;
                };
                let input = match &mut node.state {
                    Widget::TextInput(input) => input,
                    Widget::TextArea(area) => &mut area.input,
                    Widget::SearchInput(search) => &mut search.input,
                    _ => return false,
                };
                if input.disabled || input.read_only {
                    return false;
                }
                let before = (input.edit.selection_anchor(), input.edit.caret());
                input.edit.set_selection_from_anchor(anchor, caret);
                before != (input.edit.selection_anchor(), input.edit.caret())
            }
            SemanticAction::ScrollIntoView => false,
        }
    }
}

impl ApplicationHandler for WidgetApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
            return;
        }
        let window = match event_loop
            .create_window(Window::default_attributes().with_title("Mio-GUI representative form"))
        {
            Ok(window) => Arc::new(window),
            Err(error) => {
                eprintln!("Mio-GUI window creation failed: {error}");
                event_loop.exit();
                return;
            }
        };
        let renderer = match pollster::block_on(Renderer::new(window.clone())) {
            Ok(renderer) => renderer,
            Err(error) => {
                eprintln!("Mio-GUI renderer initialization failed: {error}");
                event_loop.exit();
                return;
            }
        };
        let size = window.inner_size();
        window.set_ime_allowed(true);
        if let Some(theme) = window.theme() {
            apply_winit_theme(&mut self.theme, theme);
        }
        self.window = Some(window.clone());
        self.renderer = Some(renderer);
        self.clipboard = match SystemClipboard::new() {
            Ok(clipboard) => Some(clipboard),
            Err(error) => {
                eprintln!("Mio-GUI clipboard initialization failed: {error}");
                None
            }
        };
        self.rebuild(size);
        if let Some(frame) = self.frame.as_ref() {
            self.accessibility = PlatformAccessibility::new(
                event_loop,
                &window,
                &frame.semantics,
                &frame.geometry,
                ScaleFactor::new(window.scale_factor() as f32).unwrap(),
                self.focus.focused(),
            );
        }
        self.redraws_remaining = SETTLE_REDRAWS;
        window.request_redraw();
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.accessibility = None;
        self.renderer = None;
        self.window = None;
        self.frame = None;
        self.pending_size = None;
        self.clipboard = None;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let (Some(adapter), Some(window)) = (self.accessibility.as_mut(), self.window.as_ref()) {
            adapter.process_event(window, &event);
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.pending_size = Some(size);
                self.redraws_remaining = SETTLE_REDRAWS;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.scale_factor_changed(scale_factor);
                }
                self.redraws_remaining = SETTLE_REDRAWS;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::ThemeChanged(theme) => {
                if apply_winit_theme(&mut self.theme, theme) {
                    self.redraws_remaining = SETTLE_REDRAWS;
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::Ime(ime) => self.handle_ime(ime),
            WindowEvent::CursorMoved { position, .. } => {
                let scale = self
                    .renderer
                    .as_ref()
                    .map(Renderer::scale_factor)
                    .unwrap_or(1.0)
                    .max(f32::EPSILON);
                self.cursor = Some(LogicalPoint::new(
                    position.x as f32 / scale,
                    position.y as f32 / scale,
                ));
                self.handle_pointer_drag();
            }
            WindowEvent::CursorLeft { .. } => self.cursor = None,
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => self.handle_pointer_press(),
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => self.pointer_target = None,
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                self.handle_keyboard(&event);
            }
            WindowEvent::RedrawRequested => {
                self.handle_accessibility_actions();
                let size = self
                    .pending_size
                    .take()
                    .or_else(|| self.window.as_ref().map(|window| window.inner_size()));
                if let Some(size) = size {
                    if let Some(renderer) = self.renderer.as_mut() {
                        renderer.resize(size);
                    }
                    self.rebuild(size);
                }
                if let Some(renderer) = self.renderer.as_mut() {
                    if let Err(error) = renderer.render() {
                        eprintln!("Mio-GUI render error: {error}");
                    }
                }
                self.redraws_remaining = self.redraws_remaining.saturating_sub(1);
                if self.redraws_remaining > 0 {
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WidgetApp;
    use crate::{
        Button, Checkbox, Column, Direction, FocusSnapshot, Key, KeyModifiers, KeyboardEvent,
        LogicalSize, Menu, MenuItem, Modal, Radio, Select, SelectOption, SemanticAction, Switch,
        TextInput, Widget, WidgetTree,
    };

    #[test]
    fn focus_projection_and_activation_update_retained_widgets() {
        let mut tree = WidgetTree::new(Widget::from(Column::default()));
        let root = tree.root();
        let input = tree
            .append(root, Widget::from(TextInput::new("Name")))
            .unwrap();
        let checkbox = tree
            .append(root, Widget::from(Checkbox::new("Updates")))
            .unwrap();
        let button = tree
            .append(root, Widget::from(Button::new("Submit")))
            .unwrap();
        let radio = tree
            .append(root, Widget::from(Radio::new("Choice")))
            .unwrap();
        let switch = tree
            .append(root, Widget::from(Switch::new("Alerts")))
            .unwrap();
        let mut app = WidgetApp::new(tree, Direction::Rtl);
        let snapshot = FocusSnapshot::build(&app.tree, |_, widget| widget.focus_policy());

        assert!(app.focus.focus(&snapshot, input));
        app.synchronize_focus_state();
        let Widget::TextInput(input) = &app.tree.get(input).unwrap().state else {
            unreachable!()
        };
        assert!(input.focused);

        assert!(app.focus.focus(&snapshot, button));
        app.synchronize_focus_state();
        let Widget::Button(button) = &app.tree.get(button).unwrap().state else {
            unreachable!()
        };
        assert!(button.style.state.focused);

        app.activate_target(checkbox);
        let Widget::Checkbox(checkbox) = &app.tree.get(checkbox).unwrap().state else {
            unreachable!()
        };
        assert!(checkbox.checked);

        app.activate_target(radio);
        let Widget::Radio(radio) = &app.tree.get(radio).unwrap().state else {
            unreachable!()
        };
        assert!(radio.selected);

        app.activate_target(switch);
        let Widget::Switch(switch) = &app.tree.get(switch).unwrap().state else {
            unreachable!()
        };
        assert!(switch.checked);
    }

    #[test]
    fn activating_grouped_radio_exclusively_selects_it() {
        let mut tree = WidgetTree::new(Widget::from(Column::default()));
        let root = tree.root();
        let first = tree
            .append(
                root,
                Widget::from(Radio::new("Standard").with_group("delivery", "standard")),
            )
            .unwrap();
        let second = tree
            .append(
                root,
                Widget::from(Radio::new("Express").with_group("delivery", "express")),
            )
            .unwrap();
        let mut app = WidgetApp::new(tree, Direction::Ltr);

        app.activate_target(first);
        app.activate_target(second);

        let Widget::Radio(first) = &app.tree.get(first).unwrap().state else {
            unreachable!()
        };
        let Widget::Radio(second) = &app.tree.get(second).unwrap().state else {
            unreachable!()
        };
        assert!(!first.selected);
        assert!(second.selected);
    }

    #[test]
    fn radio_group_movement_skips_disabled_options_and_wraps() {
        let mut tree = WidgetTree::new(Widget::from(Column::default()));
        let root = tree.root();
        let first = tree
            .append(
                root,
                Widget::from(Radio::new("First").with_group("choice", "first")),
            )
            .unwrap();
        let mut disabled = Radio::new("Disabled").with_group("choice", "disabled");
        disabled.disabled = true;
        tree.append(root, Widget::from(disabled)).unwrap();
        let last = tree
            .append(
                root,
                Widget::from(Radio::new("Last").with_group("choice", "last")),
            )
            .unwrap();
        let mut app = WidgetApp::new(tree, Direction::Ltr);
        let snapshot = app.focus_snapshot();
        assert_eq!(snapshot.tab_order(), &[first]);
        assert!(snapshot.accepts_focus(last));
        assert!(app.focus.focus(&snapshot, first));

        assert!(app.move_radio_group(first, true));
        assert_eq!(app.focus.focused(), Some(last));
        assert_eq!(app.focus_snapshot().tab_order(), &[last]);
        assert!(app.move_radio_group(last, true));
        assert_eq!(app.focus.focused(), Some(first));
    }

    #[test]
    fn key_release_does_not_repeat_control_activation() {
        let tree = WidgetTree::new(Widget::from(Checkbox::new("Updates")));
        let checkbox = tree.root();
        let mut app = WidgetApp::new(tree, Direction::Ltr);
        let snapshot = app.focus_snapshot();
        assert!(app.focus.focus(&snapshot, checkbox));

        app.handle_keyboard_event(KeyboardEvent::pressed(Key::Space));
        app.handle_keyboard_event(KeyboardEvent {
            state: crate::KeyState::Released,
            ..KeyboardEvent::pressed(Key::Space)
        });

        let Widget::Checkbox(checkbox) = &app.tree.get(checkbox).unwrap().state else {
            unreachable!()
        };
        assert!(checkbox.checked);
    }

    #[test]
    fn held_slider_key_repeats_but_release_does_not_adjust() {
        let tree = WidgetTree::new(Widget::from(
            crate::Slider::new("Amount", 0.0..=10.0, 0.0).unwrap(),
        ));
        let slider = tree.root();
        let mut app = WidgetApp::new(tree, Direction::Ltr);
        let snapshot = app.focus_snapshot();
        assert!(app.focus.focus(&snapshot, slider));
        let key = Key::Arrow(crate::ArrowKey::Right);

        app.handle_keyboard_event(KeyboardEvent::pressed(key.clone()));
        app.handle_keyboard_event(KeyboardEvent {
            repeat: true,
            ..KeyboardEvent::pressed(key.clone())
        });
        app.handle_keyboard_event(KeyboardEvent {
            state: crate::KeyState::Released,
            ..KeyboardEvent::pressed(key)
        });

        let Widget::Slider(slider) = &app.tree.get(slider).unwrap().state else {
            unreachable!()
        };
        assert!((slider.value() - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn icon_button_focus_and_activation_match_text_buttons() {
        let icon = crate::Icon::new(
            crate::PixelImage::new(1, 1, crate::PixelFormat::Alpha8, vec![255]).unwrap(),
        )
        .unwrap();
        let tree = WidgetTree::new(Widget::from(crate::IconButton::new(icon, "Settings")));
        let button = tree.root();
        let mut app = WidgetApp::new(tree, Direction::Ltr);
        let snapshot = app.focus_snapshot();
        assert!(app.focus.focus(&snapshot, button));

        app.synchronize_focus_state();
        assert!(app.activate_target(button));

        let Widget::IconButton(button) = &app.tree.get(button).unwrap().state else {
            unreachable!()
        };
        assert!(button.style.state.focused);
    }

    #[test]
    fn primary_select_all_shortcut_updates_focused_text_input() {
        let tree = WidgetTree::new(Widget::from(TextInput::with_text("Name", "Mio GUI")));
        let input = tree.root();
        let mut app = WidgetApp::new(tree, Direction::Ltr);
        let snapshot = FocusSnapshot::build(&app.tree, |_, widget| widget.focus_policy());
        assert!(app.focus.focus(&snapshot, input));
        let mut event = KeyboardEvent::pressed(Key::Character("a".into()));
        event.modifiers = if cfg!(target_os = "macos") {
            KeyModifiers {
                meta: true,
                ..KeyModifiers::default()
            }
        } else {
            KeyModifiers {
                control: true,
                ..KeyModifiers::default()
            }
        };

        assert!(app.handle_text_shortcut(input, &event));
        let Widget::TextInput(input) = &app.tree.get(input).unwrap().state else {
            unreachable!()
        };
        assert_eq!(input.edit.selected_text(), "Mio GUI");
    }

    #[test]
    fn moving_focus_closes_select_and_dropdown_popups() {
        let mut tree = WidgetTree::new(Widget::from(Column::default()));
        let root = tree.root();
        let select = tree
            .append(
                root,
                Widget::from(
                    Select::new(
                        "Country",
                        vec![
                            SelectOption::new("Iran", "ir"),
                            SelectOption::new("Japan", "jp"),
                        ],
                    )
                    .unwrap(),
                ),
            )
            .unwrap();
        let dropdown = tree
            .append(
                root,
                Widget::from(crate::Dropdown::new(
                    Button::new("Actions"),
                    Menu::new("Actions", vec![MenuItem::new("Open")]).unwrap(),
                )),
            )
            .unwrap();
        let next = tree
            .append(root, Widget::from(Button::new("Next")))
            .unwrap();
        let mut app = WidgetApp::new(tree, Direction::Ltr);
        let snapshot = app.focus_snapshot();

        assert!(app.focus.focus(&snapshot, select));
        assert!(app.activate_target(select));
        assert!(app.focus.focus(&snapshot, dropdown));
        app.synchronize_focus_state();
        let Widget::Select(select) = &app.tree.get(select).unwrap().state else {
            unreachable!()
        };
        assert!(!select.open);

        assert!(app.activate_target(dropdown));
        assert!(app.focus.focus(&snapshot, next));
        app.synchronize_focus_state();
        let Widget::Dropdown(dropdown) = &app.tree.get(dropdown).unwrap().state else {
            unreachable!()
        };
        assert!(!dropdown.open);
    }

    #[test]
    fn semantic_set_value_updates_text_and_slider_controls() {
        let mut tree = WidgetTree::new(Widget::from(Column::default()));
        let root = tree.root();
        let input = tree
            .append(root, Widget::from(TextInput::with_text("Name", "Old")))
            .unwrap();
        let slider = tree
            .append(
                root,
                Widget::from(crate::Slider::new("Amount", 0.0..=100.0, 20.0).unwrap()),
            )
            .unwrap();
        let mut app = WidgetApp::new(tree, Direction::Ltr);

        assert!(app.apply_semantic_action(crate::SemanticActionRequest {
            target: input,
            action: SemanticAction::SetValue,
            value: Some(crate::SemanticActionValue::Text("New value".into())),
        }));
        assert!(app.apply_semantic_action(crate::SemanticActionRequest {
            target: slider,
            action: SemanticAction::SetValue,
            value: Some(crate::SemanticActionValue::Number(72.5)),
        }));
        assert!(app.apply_semantic_action(crate::SemanticActionRequest {
            target: input,
            action: SemanticAction::SetTextSelection,
            value: Some(crate::SemanticActionValue::TextSelection {
                anchor: 8,
                caret: 0,
            }),
        }));

        let Widget::TextInput(input) = &app.tree.get(input).unwrap().state else {
            unreachable!()
        };
        let Widget::Slider(slider) = &app.tree.get(slider).unwrap().state else {
            unreachable!()
        };
        assert_eq!(input.text(), "New value");
        assert_eq!(input.edit.selection_anchor(), 8);
        assert_eq!(input.edit.caret(), 0);
        assert_eq!(slider.value(), 72.5);
    }

    #[test]
    fn semantic_menu_item_activation_targets_the_indexed_item() {
        let tree = WidgetTree::new(Widget::from(
            Menu::new(
                "Actions",
                vec![MenuItem::new("Open"), MenuItem::new("Delete")],
            )
            .unwrap(),
        ));
        let target = tree.root();
        let mut app = WidgetApp::new(tree, Direction::Ltr);

        assert!(app.apply_semantic_action(crate::SemanticActionRequest {
            target,
            action: SemanticAction::Activate,
            value: Some(crate::SemanticActionValue::Index(1)),
        }));
        let Widget::Menu(menu) = &app.tree.get(target).unwrap().state else {
            unreachable!()
        };
        assert_eq!(menu.active_index(), 1);
    }

    #[test]
    fn semantic_popup_item_activation_updates_and_closes_each_control() {
        let mut tree = WidgetTree::new(Widget::from(Column::default()));
        let root = tree.root();
        let mut select = Select::new(
            "Country",
            vec![
                SelectOption::new("Iran", "ir"),
                SelectOption::new("Japan", "jp"),
            ],
        )
        .unwrap();
        select.open = true;
        let select = tree.append(root, Widget::from(select)).unwrap();
        let mut dropdown = crate::Dropdown::new(
            Button::new("Actions"),
            Menu::new(
                "Actions",
                vec![MenuItem::new("Open"), MenuItem::new("Save")],
            )
            .unwrap(),
        );
        dropdown.open = true;
        let dropdown = tree.append(root, Widget::from(dropdown)).unwrap();
        let mut context = crate::ContextMenu::new(
            Menu::new(
                "Context",
                vec![MenuItem::new("Copy"), MenuItem::new("Delete")],
            )
            .unwrap(),
        );
        context.open_at(crate::LogicalPoint::default());
        let context = tree.append(root, Widget::from(context)).unwrap();
        let mut app = WidgetApp::new(tree, Direction::Ltr);

        for target in [select, dropdown, context] {
            assert!(app.apply_semantic_action(crate::SemanticActionRequest {
                target,
                action: SemanticAction::Activate,
                value: Some(crate::SemanticActionValue::Index(1)),
            }));
        }
        let Widget::Select(select) = &app.tree.get(select).unwrap().state else {
            unreachable!()
        };
        let Widget::Dropdown(dropdown) = &app.tree.get(dropdown).unwrap().state else {
            unreachable!()
        };
        let Widget::ContextMenu(context) = &app.tree.get(context).unwrap().state else {
            unreachable!()
        };
        assert_eq!(select.selected_index(), 1);
        assert!(!select.open);
        assert_eq!(dropdown.menu.active_index(), 1);
        assert!(!dropdown.open);
        assert_eq!(context.menu.active_index(), 1);
        assert!(!context.open);
    }

    #[test]
    fn modal_focus_is_trapped_and_restored_after_dismissal() {
        let mut tree = WidgetTree::new(Widget::from(Column::default()));
        let root = tree.root();
        let outside = tree
            .append(root, Widget::from(Button::new("Outside")))
            .unwrap();
        let mut modal = Modal::new("Dialog", LogicalSize::new(200.0, 120.0));
        modal.open = true;
        let modal = tree.append(root, Widget::from(modal)).unwrap();
        let inside = tree
            .append(modal, Widget::from(Button::new("Inside")))
            .unwrap();
        let mut app = WidgetApp::new(tree, Direction::Ltr);
        let initial = FocusSnapshot::build(&app.tree, |_, widget| widget.focus_policy());
        assert!(app.focus.focus(&initial, outside));

        app.reconcile_modal_focus();
        let trapped = app.focus_snapshot();
        assert_eq!(trapped.tab_order(), &[modal, inside]);
        assert_eq!(app.focus.focused(), Some(modal));

        assert!(app.dismiss_top_overlay());
        app.reconcile_modal_focus();
        assert_eq!(app.focus.focused(), Some(outside));
    }
}
