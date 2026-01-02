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
        // Serialize code - convert to serde-compatible format
        let code_serde = KeyCodeSerde::from(self.code);
        state.serialize_field("code", &code_serde)?;
        if let KeyCode::Char(c) = self.code {
            state.serialize_field("char", &c)?;
        }
        // Serialize modifiers - convert to serde-compatible format
        let modifiers_serde = KeyModifiersSerde::from(self.modifiers);
        state.serialize_field("modifiers", &modifiers_serde)?;
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
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct HotkeyVisitor;

        impl<'de> Visitor<'de> for HotkeyVisitor {
            type Value = Hotkey;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Hotkey")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Hotkey, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut code: Option<KeyCode> = None;
                let mut modifiers: Option<KeyModifiers> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "code" => {
                            if code.is_some() {
                                return Err(de::Error::duplicate_field("code"));
                            }
                            code = Some(map.next_value::<KeyCodeSerde>()?.into());
                        }
                        "modifiers" => {
                            if modifiers.is_some() {
                                return Err(de::Error::duplicate_field("modifiers"));
                            }
                            modifiers = Some(map.next_value::<KeyModifiersSerde>()?.into());
                        }
                        "char" => {
                            // Ignore char field - it's just for readability in YAML
                            let _: String = map.next_value()?;
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let code = code.ok_or_else(|| de::Error::missing_field("code"))?;
                let modifiers = modifiers.unwrap_or(KeyModifiers::empty());
                Ok(Hotkey { code, modifiers })
            }
        }

        deserializer.deserialize_map(HotkeyVisitor)
    }
}

/// Helper types for serialization of KeyCode and KeyModifiers.
///
#[derive(Serialize, Deserialize)]
enum KeyCodeSerde {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    Backtab,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Null,
    Esc,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize)]
struct KeyModifiersSerde {
    bits: u8,
}

impl From<KeyCode> for KeyCodeSerde {
    fn from(code: KeyCode) -> Self {
        match code {
            KeyCode::Backspace => KeyCodeSerde::Backspace,
            KeyCode::Enter => KeyCodeSerde::Enter,
            KeyCode::Left => KeyCodeSerde::Left,
            KeyCode::Right => KeyCodeSerde::Right,
            KeyCode::Up => KeyCodeSerde::Up,
            KeyCode::Down => KeyCodeSerde::Down,
            KeyCode::Home => KeyCodeSerde::Home,
            KeyCode::End => KeyCodeSerde::End,
            KeyCode::PageUp => KeyCodeSerde::PageUp,
            KeyCode::PageDown => KeyCodeSerde::PageDown,
            KeyCode::Tab => KeyCodeSerde::Tab,
            KeyCode::BackTab => KeyCodeSerde::Backtab,
            KeyCode::Delete => KeyCodeSerde::Delete,
            KeyCode::Insert => KeyCodeSerde::Insert,
            KeyCode::F(n) => KeyCodeSerde::F(n),
            KeyCode::Char(c) => KeyCodeSerde::Char(c),
            KeyCode::Null => KeyCodeSerde::Null,
            KeyCode::Esc => KeyCodeSerde::Esc,
            _ => KeyCodeSerde::Unknown,
        }
    }
}

impl From<KeyModifiers> for KeyModifiersSerde {
    fn from(modifiers: KeyModifiers) -> Self {
        KeyModifiersSerde {
            bits: modifiers.bits(),
        }
    }
}

impl From<KeyCodeSerde> for KeyCode {
    fn from(code: KeyCodeSerde) -> Self {
        match code {
            KeyCodeSerde::Backspace => KeyCode::Backspace,
            KeyCodeSerde::Enter => KeyCode::Enter,
            KeyCodeSerde::Left => KeyCode::Left,
            KeyCodeSerde::Right => KeyCode::Right,
            KeyCodeSerde::Up => KeyCode::Up,
            KeyCodeSerde::Down => KeyCode::Down,
            KeyCodeSerde::Home => KeyCode::Home,
            KeyCodeSerde::End => KeyCode::End,
            KeyCodeSerde::PageUp => KeyCode::PageUp,
            KeyCodeSerde::PageDown => KeyCode::PageDown,
            KeyCodeSerde::Tab => KeyCode::Tab,
            KeyCodeSerde::Backtab => KeyCode::BackTab,
            KeyCodeSerde::Delete => KeyCode::Delete,
            KeyCodeSerde::Insert => KeyCode::Insert,
            KeyCodeSerde::F(n) => KeyCode::F(n),
            KeyCodeSerde::Char(c) => KeyCode::Char(c),
            KeyCodeSerde::Null => KeyCode::Null,
            KeyCodeSerde::Esc => KeyCode::Esc,
            KeyCodeSerde::Unknown => KeyCode::Null,
        }
    }
}

impl From<KeyModifiersSerde> for KeyModifiers {
    fn from(modifiers: KeyModifiersSerde) -> Self {
        KeyModifiers::from_bits_truncate(modifiers.bits)
    }
}

/// Hotkey configuration grouped by view.
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewHotkeys {
    #[serde(default = "HashMap::new", skip_serializing_if = "HashMap::is_empty")]
    pub welcome: HashMap<HotkeyAction, Hotkey>,
    #[serde(default = "HashMap::new", skip_serializing_if = "HashMap::is_empty")]
    pub project_tasks: HashMap<HotkeyAction, Hotkey>,
    #[serde(default = "HashMap::new", skip_serializing_if = "HashMap::is_empty")]
    pub task_detail: HashMap<HotkeyAction, Hotkey>,
    #[serde(default = "HashMap::new", skip_serializing_if = "HashMap::is_empty")]
    pub create_task: HashMap<HotkeyAction, Hotkey>,
    #[serde(default = "HashMap::new", skip_serializing_if = "HashMap::is_empty")]
    pub edit_task: HashMap<HotkeyAction, Hotkey>,
    #[serde(default = "HashMap::new", skip_serializing_if = "HashMap::is_empty")]
    pub search_mode: HashMap<HotkeyAction, Hotkey>,
    #[serde(default = "HashMap::new", skip_serializing_if = "HashMap::is_empty")]
    pub debug_mode: HashMap<HotkeyAction, Hotkey>,
    #[serde(default = "HashMap::new", skip_serializing_if = "HashMap::is_empty")]
    pub delete_confirmation: HashMap<HotkeyAction, Hotkey>,
    #[serde(default = "HashMap::new", skip_serializing_if = "HashMap::is_empty")]
    pub move_task: HashMap<HotkeyAction, Hotkey>,
    #[serde(default = "HashMap::new", skip_serializing_if = "HashMap::is_empty")]
    pub theme_selector: HashMap<HotkeyAction, Hotkey>,
}

impl Default for ViewHotkeys {
    fn default() -> Self {
        default_hotkeys()
    }
}

impl ViewHotkeys {
    /// Get only the hotkeys that differ from defaults (user overrides).
    ///
    pub fn get_overrides(&self) -> ViewHotkeys {
        let defaults = default_hotkeys();
        let mut overrides = ViewHotkeys {
            welcome: HashMap::new(),
            project_tasks: HashMap::new(),
            task_detail: HashMap::new(),
            create_task: HashMap::new(),
            edit_task: HashMap::new(),
            search_mode: HashMap::new(),
            debug_mode: HashMap::new(),
            delete_confirmation: HashMap::new(),
            move_task: HashMap::new(),
            theme_selector: HashMap::new(),
        };

        // Only include hotkeys that differ from defaults
        for (action, hotkey) in &self.welcome {
            if defaults.welcome.get(action) != Some(hotkey) {
                overrides.welcome.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &self.project_tasks {
            if defaults.project_tasks.get(action) != Some(hotkey) {
                overrides
                    .project_tasks
                    .insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &self.task_detail {
            if defaults.task_detail.get(action) != Some(hotkey) {
                overrides.task_detail.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &self.create_task {
            if defaults.create_task.get(action) != Some(hotkey) {
                overrides.create_task.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &self.edit_task {
            if defaults.edit_task.get(action) != Some(hotkey) {
                overrides.edit_task.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &self.search_mode {
            if defaults.search_mode.get(action) != Some(hotkey) {
                overrides.search_mode.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &self.debug_mode {
            if defaults.debug_mode.get(action) != Some(hotkey) {
                overrides.debug_mode.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &self.delete_confirmation {
            if defaults.delete_confirmation.get(action) != Some(hotkey) {
                overrides
                    .delete_confirmation
                    .insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &self.move_task {
            if defaults.move_task.get(action) != Some(hotkey) {
                overrides.move_task.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &self.theme_selector {
            if defaults.theme_selector.get(action) != Some(hotkey) {
                overrides
                    .theme_selector
                    .insert(action.clone(), hotkey.clone());
            }
        }

        overrides
    }

    /// Merge user overrides with defaults.
    ///
    pub fn merge_with_defaults(overrides: &ViewHotkeys) -> ViewHotkeys {
        let mut merged = default_hotkeys();

        // Apply overrides on top of defaults
        for (action, hotkey) in &overrides.welcome {
            merged.welcome.insert(action.clone(), hotkey.clone());
        }
        for (action, hotkey) in &overrides.project_tasks {
            merged.project_tasks.insert(action.clone(), hotkey.clone());
        }
        for (action, hotkey) in &overrides.task_detail {
            merged.task_detail.insert(action.clone(), hotkey.clone());
        }
        for (action, hotkey) in &overrides.create_task {
            merged.create_task.insert(action.clone(), hotkey.clone());
        }
        for (action, hotkey) in &overrides.edit_task {
            merged.edit_task.insert(action.clone(), hotkey.clone());
        }
        for (action, hotkey) in &overrides.search_mode {
            merged.search_mode.insert(action.clone(), hotkey.clone());
        }
        for (action, hotkey) in &overrides.debug_mode {
            merged.debug_mode.insert(action.clone(), hotkey.clone());
        }
        for (action, hotkey) in &overrides.delete_confirmation {
            merged
                .delete_confirmation
                .insert(action.clone(), hotkey.clone());
        }
        for (action, hotkey) in &overrides.move_task {
            merged.move_task.insert(action.clone(), hotkey.clone());
        }
        for (action, hotkey) in &overrides.theme_selector {
            merged.theme_selector.insert(action.clone(), hotkey.clone());
        }

        merged
    }
}

/// Hotkey group for organizing hotkeys in the editor.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotkeyGroup {
    pub name: String,
    pub actions: Vec<HotkeyAction>,
}

/// Get all hotkey groups for the editor.
///
pub fn get_hotkey_groups() -> Vec<HotkeyGroup> {
    vec![
        HotkeyGroup {
            name: "Navigation".to_string(),
            actions: vec![
                HotkeyAction::NavigateMenuNext,
                HotkeyAction::NavigateMenuPrev,
                HotkeyAction::NavigateMenuLeft,
                HotkeyAction::NavigateMenuRight,
                HotkeyAction::NavigateTaskNext,
                HotkeyAction::NavigateTaskPrev,
                HotkeyAction::NavigateColumnNext,
                HotkeyAction::NavigateColumnPrev,
                HotkeyAction::SwitchPanelNext,
                HotkeyAction::SwitchPanelPrev,
                HotkeyAction::NavigateFieldNext,
                HotkeyAction::NavigateFieldPrev,
                HotkeyAction::ScrollDown,
                HotkeyAction::ScrollUp,
            ],
        },
        HotkeyGroup {
            name: "Actions".to_string(),
            actions: vec![
                HotkeyAction::Select,
                HotkeyAction::ViewTask,
                HotkeyAction::CreateTask,
                HotkeyAction::EditTask,
                HotkeyAction::DeleteTask,
                HotkeyAction::MoveTask,
                HotkeyAction::ToggleTaskComplete,
                HotkeyAction::ToggleStar,
                HotkeyAction::AddComment,
                HotkeyAction::EditField,
                HotkeyAction::SubmitForm,
            ],
        },
        HotkeyGroup {
            name: "System".to_string(),
            actions: vec![
                HotkeyAction::EnterSearch,
                HotkeyAction::EnterDebug,
                HotkeyAction::OpenThemeSelector,
                HotkeyAction::OpenHotkeyEditor,
                HotkeyAction::Cancel,
                HotkeyAction::Back,
                HotkeyAction::Quit,
            ],
        },
        HotkeyGroup {
            name: "Search Mode".to_string(),
            actions: vec![HotkeyAction::SearchModeExit],
        },
        HotkeyGroup {
            name: "Debug Mode".to_string(),
            actions: vec![
                HotkeyAction::DebugModeNavigateNext,
                HotkeyAction::DebugModeNavigatePrev,
                HotkeyAction::DebugModeCopyLog,
                HotkeyAction::DebugModeExit,
            ],
        },
        HotkeyGroup {
            name: "Delete Confirmation".to_string(),
            actions: vec![HotkeyAction::DeleteConfirm],
        },
        HotkeyGroup {
            name: "Move Task".to_string(),
            actions: vec![
                HotkeyAction::MoveTaskNavigateNext,
                HotkeyAction::MoveTaskNavigatePrev,
                HotkeyAction::MoveTaskConfirm,
                HotkeyAction::MoveTaskCancel,
            ],
        },
        HotkeyGroup {
            name: "Theme Selector".to_string(),
            actions: vec![
                HotkeyAction::ThemeSelectorNavigateNext,
                HotkeyAction::ThemeSelectorNavigatePrev,
                HotkeyAction::ThemeSelectorSelect,
                HotkeyAction::ThemeSelectorCancel,
            ],
        },
    ]
}

/// Type alias for grouped hotkeys to reduce complexity.
///
type GroupedHotkeys = Vec<(HotkeyGroup, Vec<(HotkeyAction, Option<Hotkey>)>)>;

/// Get all hotkeys from all views as a flat list grouped by category.
///
pub fn get_all_hotkeys_grouped(hotkeys: &ViewHotkeys) -> GroupedHotkeys {
    get_hotkey_groups()
        .into_iter()
        .map(|group| {
            let group_hotkeys: Vec<(HotkeyAction, Option<Hotkey>)> = group
                .actions
                .iter()
                .map(|action| {
                    // Find the hotkey in any view
                    let hotkey = hotkeys
                        .welcome
                        .get(action)
                        .or_else(|| hotkeys.project_tasks.get(action))
                        .or_else(|| hotkeys.task_detail.get(action))
                        .or_else(|| hotkeys.create_task.get(action))
                        .or_else(|| hotkeys.edit_task.get(action))
                        .or_else(|| hotkeys.search_mode.get(action))
                        .or_else(|| hotkeys.debug_mode.get(action))
                        .or_else(|| hotkeys.delete_confirmation.get(action))
                        .or_else(|| hotkeys.move_task.get(action))
                        .or_else(|| hotkeys.theme_selector.get(action))
                        .cloned();
                    (action.clone(), hotkey)
                })
                .collect();
            (group, group_hotkeys)
        })
        .collect()
}

/// Find which view a hotkey action belongs to.
///
pub fn find_action_view(action: &HotkeyAction) -> Vec<View> {
    let mut views = Vec::new();

    // Check each view's typical actions
    match action {
        HotkeyAction::NavigateMenuNext
        | HotkeyAction::NavigateMenuPrev
        | HotkeyAction::NavigateMenuLeft
        | HotkeyAction::NavigateMenuRight
        | HotkeyAction::ToggleStar
        | HotkeyAction::EnterSearch
        | HotkeyAction::EnterDebug
        | HotkeyAction::Select
        | HotkeyAction::OpenThemeSelector
        | HotkeyAction::OpenHotkeyEditor => {
            views.push(View::Welcome);
        }
        HotkeyAction::NavigateTaskNext
        | HotkeyAction::NavigateTaskPrev
        | HotkeyAction::NavigateColumnNext
        | HotkeyAction::NavigateColumnPrev
        | HotkeyAction::ViewTask
        | HotkeyAction::CreateTask
        | HotkeyAction::MoveTask
        | HotkeyAction::ToggleTaskComplete
        | HotkeyAction::DeleteTask
        | HotkeyAction::Back => {
            views.push(View::ProjectTasks);
        }
        HotkeyAction::SwitchPanelNext
        | HotkeyAction::SwitchPanelPrev
        | HotkeyAction::ScrollDown
        | HotkeyAction::ScrollUp
        | HotkeyAction::EditTask
        | HotkeyAction::AddComment => {
            views.push(View::TaskDetail);
        }
        HotkeyAction::NavigateFieldNext
        | HotkeyAction::NavigateFieldPrev
        | HotkeyAction::EditField
        | HotkeyAction::SubmitForm => {
            views.push(View::CreateTask);
            views.push(View::EditTask);
        }
        HotkeyAction::SearchModeExit => {
            // Search mode is available in multiple views
            views.push(View::Welcome);
            views.push(View::ProjectTasks);
        }
        HotkeyAction::DebugModeNavigateNext
        | HotkeyAction::DebugModeNavigatePrev
        | HotkeyAction::DebugModeCopyLog
        | HotkeyAction::DebugModeExit => {
            // Debug mode is available from any view
            views.push(View::Welcome);
        }
        HotkeyAction::DeleteConfirm => {
            // Delete confirmation can appear in multiple views
            views.push(View::ProjectTasks);
            views.push(View::TaskDetail);
        }
        HotkeyAction::MoveTaskNavigateNext
        | HotkeyAction::MoveTaskNavigatePrev
        | HotkeyAction::MoveTaskConfirm
        | HotkeyAction::MoveTaskCancel => {
            views.push(View::ProjectTasks);
        }
        HotkeyAction::ThemeSelectorNavigateNext
        | HotkeyAction::ThemeSelectorNavigatePrev
        | HotkeyAction::ThemeSelectorSelect
        | HotkeyAction::ThemeSelectorCancel => {
            views.push(View::Welcome);
        }
        HotkeyAction::Cancel | HotkeyAction::Quit => {
            // Available in all views
            views.push(View::Welcome);
            views.push(View::ProjectTasks);
            views.push(View::TaskDetail);
            views.push(View::CreateTask);
            views.push(View::EditTask);
        }
    }

    views
}

/// Update a hotkey for an action across all applicable views.
///
pub fn update_hotkey_for_action(hotkeys: &mut ViewHotkeys, action: &HotkeyAction, hotkey: Hotkey) {
    let views = find_action_view(action);
    for view in views {
        match view {
            View::Welcome => {
                hotkeys.welcome.insert(action.clone(), hotkey.clone());
            }
            View::ProjectTasks => {
                hotkeys.project_tasks.insert(action.clone(), hotkey.clone());
            }
            View::TaskDetail => {
                hotkeys.task_detail.insert(action.clone(), hotkey.clone());
            }
            View::CreateTask => {
                hotkeys.create_task.insert(action.clone(), hotkey.clone());
            }
            View::EditTask => {
                hotkeys.edit_task.insert(action.clone(), hotkey.clone());
            }
        }
    }

    // Also update special modes if applicable
    match action {
        HotkeyAction::SearchModeExit => {
            hotkeys.search_mode.insert(action.clone(), hotkey);
        }
        HotkeyAction::DebugModeNavigateNext
        | HotkeyAction::DebugModeNavigatePrev
        | HotkeyAction::DebugModeCopyLog
        | HotkeyAction::DebugModeExit => {
            hotkeys.debug_mode.insert(action.clone(), hotkey.clone());
        }
        HotkeyAction::DeleteConfirm => {
            hotkeys
                .delete_confirmation
                .insert(action.clone(), hotkey.clone());
        }
        HotkeyAction::MoveTaskNavigateNext
        | HotkeyAction::MoveTaskNavigatePrev
        | HotkeyAction::MoveTaskConfirm
        | HotkeyAction::MoveTaskCancel => {
            hotkeys.move_task.insert(action.clone(), hotkey.clone());
        }
        HotkeyAction::ThemeSelectorNavigateNext
        | HotkeyAction::ThemeSelectorNavigatePrev
        | HotkeyAction::ThemeSelectorSelect
        | HotkeyAction::ThemeSelectorCancel => {
            hotkeys.theme_selector.insert(action.clone(), hotkey);
        }
        _ => {}
    }
}

/// Default hotkey configurations.
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
            code: KeyCode::Char('c'),
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
            code: KeyCode::Char('x'),
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
        HotkeyAction::Back,
        Hotkey {
            code: KeyCode::Esc,
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
        HotkeyAction::Cancel,
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
        HotkeyAction::AddComment,
        Hotkey {
            code: KeyCode::Char('c'),
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
        HotkeyAction::Back,
        Hotkey {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::empty(),
        },
    );
    task_detail.insert(
        HotkeyAction::Cancel,
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
            code: KeyCode::Tab,
            modifiers: KeyModifiers::empty(),
        },
    );
    create_task.insert(
        HotkeyAction::NavigateFieldPrev,
        Hotkey {
            code: KeyCode::BackTab,
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

    let mut search_mode = HashMap::new();
    search_mode.insert(
        HotkeyAction::SearchModeExit,
        Hotkey {
            code: KeyCode::Esc,
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
            code: KeyCode::Esc,
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
        crossterm::event::KeyCode::Tab => "Tab".to_string(),
        crossterm::event::KeyCode::BackTab => "Shift+Tab".to_string(),
        crossterm::event::KeyCode::F(n) => format!("F{}", n),
        _ => "Unknown".to_string(),
    };

    if parts.is_empty() {
        key_str
    } else {
        format!("{}+{}", parts.join("+"), key_str)
    }
}
