//! Hotkey configuration management.
//!
//! This module defines the hotkey system for the application, including action types,
//! hotkey bindings, and default configurations per view.

use crate::state::View;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

/// Represents all possible actions that can be bound to hotkeys.
///
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HotkeyAction {
    // Welcome view actions
    NavigateMenuNext,
    NavigateMenuPrev,
    NavigateMenuLeft,
    NavigateMenuRight,
    ToggleStar,
    EnterSearch,
    EnterDebug,
    Select,
    Cancel,
    Quit,
    OpenThemeSelector,
    OpenHotkeyEditor,

    // ProjectTasks view actions
    NavigateTaskNext,
    NavigateTaskPrev,
    NavigateColumnNext,
    NavigateColumnPrev,
    ViewTask,
    CreateTask,
    MoveTask,
    ToggleTaskComplete,
    DeleteTask,
    Back,

    // TaskDetail view actions
    SwitchPanelNext,
    SwitchPanelPrev,
    ScrollDown,
    ScrollUp,
    EditTask,
    AddComment,

    // CreateTask/EditTask view actions
    NavigateFieldNext,
    NavigateFieldPrev,
    EditField,
    SubmitForm,

    // Special mode actions (for search, debug, modals, etc.)
    SearchModeExit,
    DebugModeNavigateNext,
    DebugModeNavigatePrev,
    DebugModeCopyLog,
    DebugModeExit,
    DeleteConfirm,
    MoveTaskNavigateNext,
    MoveTaskNavigatePrev,
    MoveTaskConfirm,
    MoveTaskCancel,
    ThemeSelectorNavigateNext,
    ThemeSelectorNavigatePrev,
    ThemeSelectorSelect,
    ThemeSelectorCancel,
}

/// Represents a key combination (KeyCode + modifiers).
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hotkey {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

/// Custom serialization for Hotkey.
///
impl Serialize for Hotkey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Hotkey", 3)?;
        state.serialize_field("code", &KeyCodeSerde::from(self.code))?;
        if let KeyCode::Char(c) = self.code {
            state.serialize_field("char", &c)?;
        }
        state.serialize_field("modifiers", &KeyModifiersSerde::from(self.modifiers))?;
        state.end()
    }
}

/// Custom deserialization for Hotkey.
///
impl<'de> Deserialize<'de> for Hotkey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct HotkeyHelper {
            code: KeyCodeSerde,
            #[serde(default)]
            char: Option<char>,
            #[serde(default)]
            modifiers: KeyModifiersSerde,
        }

        let helper = HotkeyHelper::deserialize(deserializer)?;
        let code = match helper.code {
            KeyCodeSerde::Char => {
                if let Some(c) = helper.char {
                    KeyCode::Char(c)
                } else {
                    return Err(serde::de::Error::custom(
                        "Char key code requires 'char' field",
                    ));
                }
            }
            KeyCodeSerde::Esc => KeyCode::Esc,
            KeyCodeSerde::Enter => KeyCode::Enter,
            KeyCodeSerde::Backspace => KeyCode::Backspace,
            KeyCodeSerde::Up => KeyCode::Up,
            KeyCodeSerde::Down => KeyCode::Down,
            KeyCodeSerde::Left => KeyCode::Left,
            KeyCodeSerde::Right => KeyCode::Right,
        };
        Ok(Hotkey {
            code,
            modifiers: helper.modifiers.into(),
        })
    }
}

/// Helper enum for serializing KeyCode.
///
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
enum KeyCodeSerde {
    Char,
    Esc,
    Enter,
    Backspace,
    Up,
    Down,
    Left,
    Right,
}

impl From<KeyCode> for KeyCodeSerde {
    fn from(code: KeyCode) -> Self {
        match code {
            KeyCode::Char(_) => KeyCodeSerde::Char,
            KeyCode::Esc => KeyCodeSerde::Esc,
            KeyCode::Enter => KeyCodeSerde::Enter,
            KeyCode::Backspace => KeyCodeSerde::Backspace,
            KeyCode::Up => KeyCodeSerde::Up,
            KeyCode::Down => KeyCodeSerde::Down,
            KeyCode::Left => KeyCodeSerde::Left,
            KeyCode::Right => KeyCodeSerde::Right,
            _ => KeyCodeSerde::Char, // Fallback for unsupported keys
        }
    }
}

/// Helper struct for serializing KeyModifiers.
///
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct KeyModifiersSerde {
    #[serde(default)]
    control: bool,
    #[serde(default)]
    shift: bool,
    #[serde(default)]
    alt: bool,
}

impl From<KeyModifiers> for KeyModifiersSerde {
    fn from(modifiers: KeyModifiers) -> Self {
        KeyModifiersSerde {
            control: modifiers.contains(KeyModifiers::CONTROL),
            shift: modifiers.contains(KeyModifiers::SHIFT),
            alt: modifiers.contains(KeyModifiers::ALT),
        }
    }
}

impl From<KeyModifiersSerde> for KeyModifiers {
    fn from(serde: KeyModifiersSerde) -> Self {
        let mut result = KeyModifiers::empty();
        if serde.control {
            result |= KeyModifiers::CONTROL;
        }
        if serde.shift {
            result |= KeyModifiers::SHIFT;
        }
        if serde.alt {
            result |= KeyModifiers::ALT;
        }
        result
    }
}

/// Maps hotkey actions to their key bindings for a specific view.
///
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewHotkeys {
    pub welcome: HashMap<HotkeyAction, Hotkey>,
    pub project_tasks: HashMap<HotkeyAction, Hotkey>,
    pub task_detail: HashMap<HotkeyAction, Hotkey>,
    pub create_task: HashMap<HotkeyAction, Hotkey>,
    pub edit_task: HashMap<HotkeyAction, Hotkey>,
    pub search_mode: HashMap<HotkeyAction, Hotkey>,
    pub debug_mode: HashMap<HotkeyAction, Hotkey>,
    pub delete_confirmation: HashMap<HotkeyAction, Hotkey>,
    pub move_task: HashMap<HotkeyAction, Hotkey>,
    pub theme_selector: HashMap<HotkeyAction, Hotkey>,
}

impl Default for ViewHotkeys {
    fn default() -> Self {
        default_hotkeys()
    }
}

/// Returns default hotkey mappings for all views.
///
pub fn default_hotkeys() -> ViewHotkeys {
    let mut welcome = HashMap::new();
    welcome.insert(
        HotkeyAction::NavigateMenuNext,
        Hotkey {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::empty(),
        },
    );
    welcome.insert(
        HotkeyAction::NavigateMenuPrev,
        Hotkey {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::empty(),
        },
    );
    welcome.insert(
        HotkeyAction::NavigateMenuLeft,
        Hotkey {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::empty(),
        },
    );
    welcome.insert(
        HotkeyAction::NavigateMenuRight,
        Hotkey {
            code: KeyCode::Char('l'),
            modifiers: KeyModifiers::empty(),
        },
    );
    welcome.insert(
        HotkeyAction::ToggleStar,
        Hotkey {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::empty(),
        },
    );
    welcome.insert(
        HotkeyAction::EnterSearch,
        Hotkey {
            code: KeyCode::Char('/'),
            modifiers: KeyModifiers::empty(),
        },
    );
    welcome.insert(
        HotkeyAction::EnterDebug,
        Hotkey {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::empty(),
        },
    );
    welcome.insert(
        HotkeyAction::Select,
        Hotkey {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::empty(),
        },
    );
    welcome.insert(
        HotkeyAction::Cancel,
        Hotkey {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::empty(),
        },
    );
    welcome.insert(
        HotkeyAction::Quit,
        Hotkey {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::empty(),
        },
    );
    welcome.insert(
        HotkeyAction::OpenThemeSelector,
        Hotkey {
            code: KeyCode::Char('t'),
            modifiers: KeyModifiers::empty(),
        },
    );
    welcome.insert(
        HotkeyAction::OpenHotkeyEditor,
        Hotkey {
            code: KeyCode::Char('?'),
            modifiers: KeyModifiers::empty(),
        },
    );

    let mut project_tasks = HashMap::new();
    project_tasks.insert(
        HotkeyAction::NavigateTaskNext,
        Hotkey {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::empty(),
        },
    );
    project_tasks.insert(
        HotkeyAction::NavigateTaskPrev,
        Hotkey {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::empty(),
        },
    );
    project_tasks.insert(
        HotkeyAction::NavigateColumnNext,
        Hotkey {
            code: KeyCode::Char('l'),
            modifiers: KeyModifiers::empty(),
        },
    );
    project_tasks.insert(
        HotkeyAction::NavigateColumnPrev,
        Hotkey {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::empty(),
        },
    );
    project_tasks.insert(
        HotkeyAction::ViewTask,
        Hotkey {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::empty(),
        },
    );
    project_tasks.insert(
        HotkeyAction::CreateTask,
        Hotkey {
            code: KeyCode::Char('n'),
            modifiers: KeyModifiers::empty(),
        },
    );
    project_tasks.insert(
        HotkeyAction::MoveTask,
        Hotkey {
            code: KeyCode::Char('m'),
            modifiers: KeyModifiers::empty(),
        },
    );
    project_tasks.insert(
        HotkeyAction::ToggleTaskComplete,
        Hotkey {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::empty(),
        },
    );
    project_tasks.insert(
        HotkeyAction::DeleteTask,
        Hotkey {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::empty(),
        },
    );
    project_tasks.insert(
        HotkeyAction::EnterSearch,
        Hotkey {
            code: KeyCode::Char('/'),
            modifiers: KeyModifiers::empty(),
        },
    );
    project_tasks.insert(
        HotkeyAction::Back,
        Hotkey {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::empty(),
        },
    );
    project_tasks.insert(
        HotkeyAction::Quit,
        Hotkey {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::empty(),
        },
    );

    let mut task_detail = HashMap::new();
    task_detail.insert(
        HotkeyAction::SwitchPanelNext,
        Hotkey {
            code: KeyCode::Char('l'),
            modifiers: KeyModifiers::empty(),
        },
    );
    task_detail.insert(
        HotkeyAction::SwitchPanelPrev,
        Hotkey {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::empty(),
        },
    );
    task_detail.insert(
        HotkeyAction::ScrollDown,
        Hotkey {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::empty(),
        },
    );
    task_detail.insert(
        HotkeyAction::ScrollUp,
        Hotkey {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::empty(),
        },
    );
    task_detail.insert(
        HotkeyAction::EditTask,
        Hotkey {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::empty(),
        },
    );
    task_detail.insert(
        HotkeyAction::DeleteTask,
        Hotkey {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::empty(),
        },
    );
    task_detail.insert(
        HotkeyAction::AddComment,
        Hotkey {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::empty(),
        },
    );
    task_detail.insert(
        HotkeyAction::Back,
        Hotkey {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::empty(),
        },
    );
    task_detail.insert(
        HotkeyAction::Quit,
        Hotkey {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::empty(),
        },
    );

    let mut create_task = HashMap::new();
    create_task.insert(
        HotkeyAction::NavigateFieldNext,
        Hotkey {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::empty(),
        },
    );
    create_task.insert(
        HotkeyAction::NavigateFieldPrev,
        Hotkey {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::empty(),
        },
    );
    create_task.insert(
        HotkeyAction::EditField,
        Hotkey {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::empty(),
        },
    );
    create_task.insert(
        HotkeyAction::SubmitForm,
        Hotkey {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::empty(),
        },
    );
    create_task.insert(
        HotkeyAction::Cancel,
        Hotkey {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::empty(),
        },
    );

    let edit_task = create_task.clone();

    // Special mode hotkeys
    let mut search_mode = HashMap::new();
    search_mode.insert(
        HotkeyAction::SearchModeExit,
        Hotkey {
            code: KeyCode::Char('/'),
            modifiers: KeyModifiers::empty(),
        },
    );
    search_mode.insert(
        HotkeyAction::Cancel,
        Hotkey {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::empty(),
        },
    );

    let mut debug_mode = HashMap::new();
    debug_mode.insert(
        HotkeyAction::DebugModeNavigateNext,
        Hotkey {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::empty(),
        },
    );
    debug_mode.insert(
        HotkeyAction::DebugModeNavigatePrev,
        Hotkey {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::empty(),
        },
    );
    debug_mode.insert(
        HotkeyAction::DebugModeCopyLog,
        Hotkey {
            code: KeyCode::Char('y'),
            modifiers: KeyModifiers::empty(),
        },
    );
    debug_mode.insert(
        HotkeyAction::DebugModeExit,
        Hotkey {
            code: KeyCode::Char('/'),
            modifiers: KeyModifiers::empty(),
        },
    );
    debug_mode.insert(
        HotkeyAction::Cancel,
        Hotkey {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::empty(),
        },
    );

    let mut delete_confirmation = HashMap::new();
    delete_confirmation.insert(
        HotkeyAction::DeleteConfirm,
        Hotkey {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::empty(),
        },
    );
    delete_confirmation.insert(
        HotkeyAction::Cancel,
        Hotkey {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::empty(),
        },
    );

    let mut move_task = HashMap::new();
    move_task.insert(
        HotkeyAction::MoveTaskNavigateNext,
        Hotkey {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::empty(),
        },
    );
    move_task.insert(
        HotkeyAction::MoveTaskNavigatePrev,
        Hotkey {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::empty(),
        },
    );
    move_task.insert(
        HotkeyAction::MoveTaskConfirm,
        Hotkey {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::empty(),
        },
    );
    move_task.insert(
        HotkeyAction::MoveTaskCancel,
        Hotkey {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::empty(),
        },
    );

    let mut theme_selector = HashMap::new();
    theme_selector.insert(
        HotkeyAction::ThemeSelectorNavigateNext,
        Hotkey {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::empty(),
        },
    );
    theme_selector.insert(
        HotkeyAction::ThemeSelectorNavigatePrev,
        Hotkey {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::empty(),
        },
    );
    theme_selector.insert(
        HotkeyAction::ThemeSelectorSelect,
        Hotkey {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::empty(),
        },
    );
    theme_selector.insert(
        HotkeyAction::ThemeSelectorCancel,
        Hotkey {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::empty(),
        },
    );

    ViewHotkeys {
        welcome,
        project_tasks,
        task_detail,
        create_task,
        edit_task,
        search_mode,
        debug_mode,
        delete_confirmation,
        move_task,
        theme_selector,
    }
}

/// Checks if a KeyEvent matches a Hotkey.
///
pub fn matches_hotkey(event: &KeyEvent, hotkey: &Hotkey) -> bool {
    event.code == hotkey.code && event.modifiers == hotkey.modifiers
}

/// Gets the action for a KeyEvent in a specific view.
///
pub fn get_action_for_event(
    event: &KeyEvent,
    view: &View,
    hotkeys: &ViewHotkeys,
) -> Option<HotkeyAction> {
    let view_hotkeys = match view {
        View::Welcome => &hotkeys.welcome,
        View::ProjectTasks => &hotkeys.project_tasks,
        View::TaskDetail => &hotkeys.task_detail,
        View::CreateTask => &hotkeys.create_task,
        View::EditTask => &hotkeys.edit_task,
    };

    view_hotkeys
        .iter()
        .find(|(_, hotkey)| matches_hotkey(event, hotkey))
        .map(|(action, _)| action.clone())
}

/// Gets the action for a KeyEvent in a special mode.
///
pub fn get_action_for_special_mode(
    event: &KeyEvent,
    mode: SpecialMode,
    hotkeys: &ViewHotkeys,
) -> Option<HotkeyAction> {
    let mode_hotkeys = match mode {
        SpecialMode::Search => &hotkeys.search_mode,
        SpecialMode::Debug => &hotkeys.debug_mode,
        SpecialMode::DeleteConfirmation => &hotkeys.delete_confirmation,
        SpecialMode::MoveTask => &hotkeys.move_task,
        SpecialMode::ThemeSelector => &hotkeys.theme_selector,
    };

    mode_hotkeys
        .iter()
        .find(|(_, hotkey)| matches_hotkey(event, hotkey))
        .map(|(action, _)| action.clone())
}

/// Represents special modes that have their own hotkey configurations.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialMode {
    Search,
    Debug,
    DeleteConfirmation,
    MoveTask,
    ThemeSelector,
}

/// Builds a footer text string from hotkey configurations.
/// Takes a list of tuples: (action, description, optional_second_action_for_paired_keys)
///
pub fn build_footer_text(
    hotkeys: &HashMap<HotkeyAction, Hotkey>,
    actions: &[(HotkeyAction, &str, Option<HotkeyAction>)],
) -> String {
    let mut parts = Vec::new();

    for (action, description, paired_action) in actions {
        if let Some(hotkey) = hotkeys.get(action) {
            if let Some(paired) = paired_action {
                if let Some(paired_hotkey) = hotkeys.get(paired) {
                    parts.push(format!(
                        " {}/{}: {}",
                        format_hotkey_display(hotkey),
                        format_hotkey_display(paired_hotkey),
                        description
                    ));
                } else {
                    parts.push(format!(
                        " {}: {}",
                        format_hotkey_display(hotkey),
                        description
                    ));
                }
            } else {
                parts.push(format!(
                    " {}: {}",
                    format_hotkey_display(hotkey),
                    description
                ));
            }
        }
    }

    if parts.is_empty() {
        String::new()
    } else {
        parts.join(",")
    }
}

/// Formats a hotkey for display in the footer.
///
pub fn format_hotkey_display(hotkey: &Hotkey) -> String {
    let mut parts = Vec::new();
    if hotkey
        .modifiers
        .contains(crossterm::event::KeyModifiers::CONTROL)
    {
        parts.push("Ctrl");
    }
    if hotkey
        .modifiers
        .contains(crossterm::event::KeyModifiers::SHIFT)
    {
        parts.push("Shift");
    }
    if hotkey
        .modifiers
        .contains(crossterm::event::KeyModifiers::ALT)
    {
        parts.push("Alt");
    }

    let key_str = match &hotkey.code {
        crossterm::event::KeyCode::Char(c) => {
            if *c == ' ' {
                "Space".to_string()
            } else {
                c.to_string()
            }
        }
        crossterm::event::KeyCode::Esc => "Esc".to_string(),
        crossterm::event::KeyCode::Enter => "Enter".to_string(),
        crossterm::event::KeyCode::Backspace => "Backspace".to_string(),
        crossterm::event::KeyCode::Up => "Up".to_string(),
        crossterm::event::KeyCode::Down => "Down".to_string(),
        crossterm::event::KeyCode::Left => "Left".to_string(),
        crossterm::event::KeyCode::Right => "Right".to_string(),
        _ => "Unknown".to_string(),
    };

    if parts.is_empty() {
        key_str
    } else {
        format!("{}+{}", parts.join("+"), key_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEventKind;

    #[test]
    fn test_matches_hotkey() {
        let hotkey = Hotkey {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::empty(),
        };
        let event = KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        assert!(matches_hotkey(&event, &hotkey));

        let event2 = KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        assert!(!matches_hotkey(&event2, &hotkey));
    }

    #[test]
    fn test_get_action_for_event() {
        let hotkeys = default_hotkeys();
        let event = KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };

        let action = get_action_for_event(&event, &View::Welcome, &hotkeys);
        assert_eq!(action, Some(HotkeyAction::NavigateMenuNext));

        let action2 = get_action_for_event(&event, &View::ProjectTasks, &hotkeys);
        assert_eq!(action2, Some(HotkeyAction::NavigateTaskNext));
    }

    #[test]
    fn test_default_hotkeys() {
        let hotkeys = default_hotkeys();
        assert!(!hotkeys.welcome.is_empty());
        assert!(!hotkeys.project_tasks.is_empty());
        assert!(!hotkeys.task_detail.is_empty());
        assert!(!hotkeys.create_task.is_empty());
        assert!(!hotkeys.edit_task.is_empty());
    }

    #[test]
    fn test_hotkey_serialization() {
        let hotkey = Hotkey {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::empty(),
        };
        let serialized = serde_yaml::to_string(&hotkey).unwrap();
        assert!(serialized.contains("j"));
        let deserialized: Hotkey = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(hotkey, deserialized);
    }
}
