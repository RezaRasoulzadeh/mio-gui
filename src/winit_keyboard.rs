// winit_keyboard.rs

use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key as WinitKey, ModifiersState, NamedKey};

use crate::{ArrowKey, Key, KeyModifiers, KeyState, KeyboardEvent};

pub fn keyboard_event_from_winit(event: &KeyEvent, modifiers: ModifiersState) -> KeyboardEvent {
    keyboard_event_from_parts(&event.logical_key, event.state, event.repeat, modifiers)
}

fn keyboard_event_from_parts(
    key: &WinitKey,
    state: ElementState,
    repeat: bool,
    modifiers: ModifiersState,
) -> KeyboardEvent {
    KeyboardEvent {
        key: match key {
            WinitKey::Named(NamedKey::Tab) => Key::Tab,
            WinitKey::Named(NamedKey::Enter) => Key::Enter,
            WinitKey::Named(NamedKey::Space) => Key::Space,
            WinitKey::Named(NamedKey::Escape) => Key::Escape,
            WinitKey::Named(NamedKey::ArrowLeft) => Key::Arrow(ArrowKey::Left),
            WinitKey::Named(NamedKey::ArrowRight) => Key::Arrow(ArrowKey::Right),
            WinitKey::Named(NamedKey::ArrowUp) => Key::Arrow(ArrowKey::Up),
            WinitKey::Named(NamedKey::ArrowDown) => Key::Arrow(ArrowKey::Down),
            WinitKey::Named(NamedKey::Home) => Key::Home,
            WinitKey::Named(NamedKey::End) => Key::End,
            WinitKey::Named(NamedKey::PageUp) => Key::PageUp,
            WinitKey::Named(NamedKey::PageDown) => Key::PageDown,
            WinitKey::Named(NamedKey::Backspace) => Key::Backspace,
            WinitKey::Named(NamedKey::Delete) => Key::Delete,
            WinitKey::Character(character) => Key::Character(character.to_string()),
            _ => Key::Unidentified,
        },
        state: match state {
            ElementState::Pressed => KeyState::Pressed,
            ElementState::Released => KeyState::Released,
        },
        modifiers: KeyModifiers {
            shift: modifiers.shift_key(),
            control: modifiers.control_key(),
            alt: modifiers.alt_key(),
            meta: modifiers.super_key(),
        },
        repeat,
    }
}

#[cfg(test)]
mod tests {
    use super::keyboard_event_from_parts;
    use crate::{ArrowKey, Key, KeyModifiers, KeyState};
    use winit::event::ElementState;
    use winit::keyboard::{Key as WinitKey, ModifiersState, NamedKey};

    #[test]
    fn named_navigation_keys_map_without_platform_types_in_core() {
        for (source, expected) in [
            (NamedKey::Tab, Key::Tab),
            (NamedKey::Enter, Key::Enter),
            (NamedKey::Space, Key::Space),
            (NamedKey::Escape, Key::Escape),
            (NamedKey::ArrowLeft, Key::Arrow(ArrowKey::Left)),
            (NamedKey::ArrowRight, Key::Arrow(ArrowKey::Right)),
            (NamedKey::ArrowUp, Key::Arrow(ArrowKey::Up)),
            (NamedKey::ArrowDown, Key::Arrow(ArrowKey::Down)),
        ] {
            let event = keyboard_event_from_parts(
                &WinitKey::Named(source),
                ElementState::Pressed,
                false,
                ModifiersState::empty(),
            );
            assert_eq!(event.key, expected);
            assert_eq!(event.state, KeyState::Pressed);
        }
    }

    #[test]
    fn character_release_repeat_and_modifiers_are_preserved() {
        let event = keyboard_event_from_parts(
            &WinitKey::Character("ژ".into()),
            ElementState::Released,
            true,
            ModifiersState::SHIFT | ModifiersState::CONTROL | ModifiersState::SUPER,
        );

        assert_eq!(event.key, Key::Character("ژ".to_owned()));
        assert_eq!(event.state, KeyState::Released);
        assert!(event.repeat);
        assert_eq!(
            event.modifiers,
            KeyModifiers {
                shift: true,
                control: true,
                alt: false,
                meta: true,
            }
        );
    }

    #[test]
    fn unsupported_named_keys_are_explicitly_unidentified() {
        let event = keyboard_event_from_parts(
            &WinitKey::Named(NamedKey::F1),
            ElementState::Pressed,
            false,
            ModifiersState::empty(),
        );

        assert_eq!(event.key, Key::Unidentified);
    }
}
