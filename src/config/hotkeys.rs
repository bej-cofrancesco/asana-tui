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
    // Global navigation actions (work across all views)
    NavigateNext,  // j - navigate down/next in any view
    NavigatePrev,  // k - navigate up/prev in any view
    NavigateLeft,  // h - navigate left in any view
    NavigateRight, // l - navigate right in any view

    // Welcome view actions
    ToggleStar,
    EnterSearch,
    EnterDebug,
    Select,
    Cancel,
    Quit,
    OpenThemeSelector,
    OpenHotkeyEditor,

    // ProjectTasks view actions
    ViewTask,
    CreateTask,
    MoveTask,
    ToggleTaskComplete,
    DeleteTask,
    Back,
    FilterByAssignee,

    // TaskDetail view actions
    EditTask,
    AddComment,

    // CreateTask/EditTask view actions
    EditField,
    SubmitForm,

    // Special mode actions (for search, debug, modals, etc.)
    // Note: Navigation in special modes uses global NavigateNext/NavigatePrev
    SearchModeExit,
    DebugModeCopyLog,
    DebugModeExit,
    DeleteConfirm,
    MoveTaskConfirm,
    MoveTaskCancel,
    ThemeSelectorSelect,
    ThemeSelectorCancel,
    AssigneeFilterSelect,
    AssigneeFilterCancel,
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

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
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
#[serde(rename_all = "PascalCase")]
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
    #[serde(default = "HashMap::new", skip_serializing_if = "HashMap::is_empty")]
    pub assignee_filter: HashMap<HotkeyAction, Hotkey>,
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
        let global_nav = global_navigation_hotkeys();
        let global_nav_actions = [
            HotkeyAction::NavigateNext,
            HotkeyAction::NavigatePrev,
            HotkeyAction::NavigateLeft,
            HotkeyAction::NavigateRight,
        ];
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
            assignee_filter: HashMap::new(),
        };

        // Check if any global navigation action is overridden
        for action in &global_nav_actions {
            let default_hotkey = global_nav.get(action);
            let mut has_override = false;
            let mut override_hotkey = None;

            // Check all views to see if any have a different value
            for view_hotkeys in [
                &self.welcome,
                &self.project_tasks,
                &self.task_detail,
                &self.create_task,
                &self.edit_task,
                &self.debug_mode,
                &self.move_task,
                &self.theme_selector,
                &self.assignee_filter,
            ] {
                if let Some(hotkey) = view_hotkeys.get(action) {
                    if Some(hotkey) != default_hotkey {
                        has_override = true;
                        override_hotkey = Some(hotkey.clone());
                        break;
                    }
                }
            }

            // If there's an override, store it in welcome (as the canonical location)
            // merge_with_defaults will apply it globally
            if has_override {
                if let Some(hotkey) = override_hotkey {
                    overrides.welcome.insert(action.clone(), hotkey);
                }
            }
        }

        // Only include non-navigation hotkeys that differ from defaults
        for (action, hotkey) in &self.welcome {
            if !global_nav_actions.contains(action) && defaults.welcome.get(action) != Some(hotkey)
            {
                overrides.welcome.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &self.project_tasks {
            if !global_nav_actions.contains(action)
                && defaults.project_tasks.get(action) != Some(hotkey)
            {
                overrides
                    .project_tasks
                    .insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &self.task_detail {
            if !global_nav_actions.contains(action)
                && defaults.task_detail.get(action) != Some(hotkey)
            {
                overrides.task_detail.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &self.create_task {
            if !global_nav_actions.contains(action)
                && defaults.create_task.get(action) != Some(hotkey)
            {
                overrides.create_task.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &self.edit_task {
            if !global_nav_actions.contains(action)
                && defaults.edit_task.get(action) != Some(hotkey)
            {
                overrides.edit_task.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &self.search_mode {
            if defaults.search_mode.get(action) != Some(hotkey) {
                overrides.search_mode.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &self.debug_mode {
            if !global_nav_actions.contains(action)
                && defaults.debug_mode.get(action) != Some(hotkey)
            {
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
            if !global_nav_actions.contains(action)
                && defaults.move_task.get(action) != Some(hotkey)
            {
                overrides.move_task.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &self.theme_selector {
            if !global_nav_actions.contains(action)
                && defaults.theme_selector.get(action) != Some(hotkey)
            {
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
        let global_nav_actions = [
            HotkeyAction::NavigateNext,
            HotkeyAction::NavigatePrev,
            HotkeyAction::NavigateLeft,
            HotkeyAction::NavigateRight,
        ];

        // First, check if any global navigation actions are overridden
        for action in &global_nav_actions {
            // Check if this action is overridden in any view
            let override_hotkey = overrides
                .welcome
                .get(action)
                .or_else(|| overrides.project_tasks.get(action))
                .or_else(|| overrides.task_detail.get(action))
                .or_else(|| overrides.create_task.get(action))
                .or_else(|| overrides.edit_task.get(action))
                .or_else(|| overrides.debug_mode.get(action))
                .or_else(|| overrides.move_task.get(action))
                .or_else(|| overrides.theme_selector.get(action))
                .or_else(|| overrides.assignee_filter.get(action));

            if let Some(hotkey) = override_hotkey {
                // Apply the override globally to all views
                merged.welcome.insert(action.clone(), hotkey.clone());
                merged.project_tasks.insert(action.clone(), hotkey.clone());
                merged.task_detail.insert(action.clone(), hotkey.clone());
                merged.create_task.insert(action.clone(), hotkey.clone());
                merged.edit_task.insert(action.clone(), hotkey.clone());
                merged.debug_mode.insert(action.clone(), hotkey.clone());
                merged.move_task.insert(action.clone(), hotkey.clone());
                merged.theme_selector.insert(action.clone(), hotkey.clone());
                merged
                    .assignee_filter
                    .insert(action.clone(), hotkey.clone());
            }
        }

        // Apply non-navigation overrides on top of defaults
        for (action, hotkey) in &overrides.welcome {
            if !global_nav_actions.contains(action) {
                merged.welcome.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &overrides.project_tasks {
            if !global_nav_actions.contains(action) {
                merged.project_tasks.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &overrides.task_detail {
            if !global_nav_actions.contains(action) {
                merged.task_detail.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &overrides.create_task {
            if !global_nav_actions.contains(action) {
                merged.create_task.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &overrides.edit_task {
            if !global_nav_actions.contains(action) {
                merged.edit_task.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &overrides.search_mode {
            merged.search_mode.insert(action.clone(), hotkey.clone());
        }
        for (action, hotkey) in &overrides.debug_mode {
            if !global_nav_actions.contains(action) {
                merged.debug_mode.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &overrides.delete_confirmation {
            merged
                .delete_confirmation
                .insert(action.clone(), hotkey.clone());
        }
        for (action, hotkey) in &overrides.move_task {
            if !global_nav_actions.contains(action) {
                merged.move_task.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &overrides.theme_selector {
            if !global_nav_actions.contains(action) {
                merged.theme_selector.insert(action.clone(), hotkey.clone());
            }
        }
        for (action, hotkey) in &overrides.assignee_filter {
            if !global_nav_actions.contains(action) {
                merged
                    .assignee_filter
                    .insert(action.clone(), hotkey.clone());
            }
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
                HotkeyAction::NavigateNext,
                HotkeyAction::NavigatePrev,
                HotkeyAction::NavigateLeft,
                HotkeyAction::NavigateRight,
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
                HotkeyAction::FilterByAssignee,
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
            actions: vec![HotkeyAction::DebugModeCopyLog, HotkeyAction::DebugModeExit],
        },
        HotkeyGroup {
            name: "Delete Confirmation".to_string(),
            actions: vec![HotkeyAction::DeleteConfirm],
        },
        HotkeyGroup {
            name: "Move Task".to_string(),
            actions: vec![HotkeyAction::MoveTaskConfirm, HotkeyAction::MoveTaskCancel],
        },
        HotkeyGroup {
            name: "Theme Selector".to_string(),
            actions: vec![
                HotkeyAction::ThemeSelectorSelect,
                HotkeyAction::ThemeSelectorCancel,
            ],
        },
        HotkeyGroup {
            name: "Assignee Filter".to_string(),
            actions: vec![
                HotkeyAction::AssigneeFilterSelect,
                HotkeyAction::AssigneeFilterCancel,
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
                        .or_else(|| hotkeys.assignee_filter.get(action))
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
        // Global navigation actions work in all views
        HotkeyAction::NavigateNext
        | HotkeyAction::NavigatePrev
        | HotkeyAction::NavigateLeft
        | HotkeyAction::NavigateRight => {
            views.push(View::Welcome);
            views.push(View::ProjectTasks);
            views.push(View::TaskDetail);
            views.push(View::CreateTask);
            views.push(View::EditTask);
        }
        HotkeyAction::ToggleStar
        | HotkeyAction::EnterSearch
        | HotkeyAction::EnterDebug
        | HotkeyAction::Select
        | HotkeyAction::OpenThemeSelector
        | HotkeyAction::OpenHotkeyEditor => {
            views.push(View::Welcome);
        }
        HotkeyAction::ViewTask
        | HotkeyAction::CreateTask
        | HotkeyAction::MoveTask
        | HotkeyAction::ToggleTaskComplete
        | HotkeyAction::DeleteTask
        | HotkeyAction::Back
        | HotkeyAction::FilterByAssignee => {
            views.push(View::ProjectTasks);
        }
        HotkeyAction::EditTask | HotkeyAction::AddComment => {
            views.push(View::TaskDetail);
        }
        HotkeyAction::EditField | HotkeyAction::SubmitForm => {
            views.push(View::CreateTask);
            views.push(View::EditTask);
        }
        HotkeyAction::SearchModeExit => {
            // Search mode is available in multiple views
            views.push(View::Welcome);
            views.push(View::ProjectTasks);
        }
        HotkeyAction::DebugModeCopyLog | HotkeyAction::DebugModeExit => {
            // Debug mode is available from any view
            views.push(View::Welcome);
        }
        HotkeyAction::DeleteConfirm => {
            // Delete confirmation can appear in multiple views
            views.push(View::ProjectTasks);
            views.push(View::TaskDetail);
        }
        HotkeyAction::MoveTaskConfirm | HotkeyAction::MoveTaskCancel => {
            views.push(View::ProjectTasks);
        }
        HotkeyAction::ThemeSelectorSelect | HotkeyAction::ThemeSelectorCancel => {
            views.push(View::Welcome);
        }
        HotkeyAction::AssigneeFilterSelect | HotkeyAction::AssigneeFilterCancel => {
            views.push(View::ProjectTasks);
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
/// Also removes the old key binding for that action to prevent conflicts.
///
pub fn update_hotkey_for_action(hotkeys: &mut ViewHotkeys, action: &HotkeyAction, hotkey: Hotkey) {
    // Global navigation actions are applied to ALL views and special modes
    let global_nav_actions = [
        HotkeyAction::NavigateNext,
        HotkeyAction::NavigatePrev,
        HotkeyAction::NavigateLeft,
        HotkeyAction::NavigateRight,
    ];

    if global_nav_actions.contains(action) {
        // Remove old binding and apply new one globally to all views and special modes
        hotkeys.welcome.remove(action);
        hotkeys.welcome.insert(action.clone(), hotkey.clone());
        hotkeys.project_tasks.remove(action);
        hotkeys.project_tasks.insert(action.clone(), hotkey.clone());
        hotkeys.task_detail.remove(action);
        hotkeys.task_detail.insert(action.clone(), hotkey.clone());
        hotkeys.create_task.remove(action);
        hotkeys.create_task.insert(action.clone(), hotkey.clone());
        hotkeys.edit_task.remove(action);
        hotkeys.edit_task.insert(action.clone(), hotkey.clone());
        hotkeys.debug_mode.remove(action);
        hotkeys.debug_mode.insert(action.clone(), hotkey.clone());
        hotkeys.move_task.remove(action);
        hotkeys.move_task.insert(action.clone(), hotkey.clone());
        hotkeys.theme_selector.remove(action);
        hotkeys
            .theme_selector
            .insert(action.clone(), hotkey.clone());
    } else {
        // Non-global actions: update only in applicable views
        let views = find_action_view(action);
        for view in &views {
            match view {
                View::Welcome => {
                    hotkeys.welcome.remove(action);
                    hotkeys.welcome.insert(action.clone(), hotkey.clone());
                }
                View::ProjectTasks => {
                    hotkeys.project_tasks.remove(action);
                    hotkeys.project_tasks.insert(action.clone(), hotkey.clone());
                }
                View::TaskDetail => {
                    hotkeys.task_detail.remove(action);
                    hotkeys.task_detail.insert(action.clone(), hotkey.clone());
                }
                View::CreateTask => {
                    hotkeys.create_task.remove(action);
                    hotkeys.create_task.insert(action.clone(), hotkey.clone());
                }
                View::EditTask => {
                    hotkeys.edit_task.remove(action);
                    hotkeys.edit_task.insert(action.clone(), hotkey.clone());
                }
            }
        }

        // Also remove and update in special modes if applicable
        match action {
            HotkeyAction::SearchModeExit => {
                hotkeys.search_mode.remove(action);
                hotkeys.search_mode.insert(action.clone(), hotkey.clone());
            }
            HotkeyAction::DebugModeCopyLog | HotkeyAction::DebugModeExit => {
                hotkeys.debug_mode.remove(action);
                hotkeys.debug_mode.insert(action.clone(), hotkey.clone());
            }
            HotkeyAction::DeleteConfirm => {
                hotkeys.delete_confirmation.remove(action);
                hotkeys
                    .delete_confirmation
                    .insert(action.clone(), hotkey.clone());
            }
            HotkeyAction::MoveTaskConfirm | HotkeyAction::MoveTaskCancel => {
                hotkeys.move_task.remove(action);
                hotkeys.move_task.insert(action.clone(), hotkey.clone());
            }
            HotkeyAction::ThemeSelectorSelect | HotkeyAction::ThemeSelectorCancel => {
                hotkeys.theme_selector.remove(action);
                hotkeys
                    .theme_selector
                    .insert(action.clone(), hotkey.clone());
            }
            HotkeyAction::AssigneeFilterSelect | HotkeyAction::AssigneeFilterCancel => {
                hotkeys.assignee_filter.remove(action);
                hotkeys
                    .assignee_filter
                    .insert(action.clone(), hotkey.clone());
            }
            _ => {}
        }
    }

    // Also remove the new key from any other actions it might be bound to
    // This prevents one key from triggering multiple actions
    remove_key_from_all_actions(hotkeys, &hotkey, action);
}

/// Removes a key binding from all actions except the specified one.
/// This prevents key conflicts when rebinding.
///
fn remove_key_from_all_actions(
    hotkeys: &mut ViewHotkeys,
    hotkey: &Hotkey,
    except_action: &HotkeyAction,
) {
    // Helper to remove key from a map
    let remove_from_map = |map: &mut HashMap<HotkeyAction, Hotkey>| {
        let mut to_remove = Vec::new();
        for (action, existing_hotkey) in map.iter() {
            if action != except_action && matches_hotkey_static(existing_hotkey, hotkey) {
                to_remove.push(action.clone());
            }
        }
        for action in to_remove {
            map.remove(&action);
        }
    };

    remove_from_map(&mut hotkeys.welcome);
    remove_from_map(&mut hotkeys.project_tasks);
    remove_from_map(&mut hotkeys.task_detail);
    remove_from_map(&mut hotkeys.create_task);
    remove_from_map(&mut hotkeys.edit_task);
    remove_from_map(&mut hotkeys.search_mode);
    remove_from_map(&mut hotkeys.debug_mode);
    remove_from_map(&mut hotkeys.delete_confirmation);
    remove_from_map(&mut hotkeys.move_task);
    remove_from_map(&mut hotkeys.theme_selector);
    remove_from_map(&mut hotkeys.assignee_filter);
}

/// Helper to compare hotkeys without needing a KeyEvent.
///
fn matches_hotkey_static(hotkey1: &Hotkey, hotkey2: &Hotkey) -> bool {
    hotkey1.code == hotkey2.code && hotkey1.modifiers == hotkey2.modifiers
}

/// Get the global navigation hotkeys that apply to all views.
///
fn global_navigation_hotkeys() -> HashMap<HotkeyAction, Hotkey> {
    let mut nav = HashMap::new();
    nav.insert(
        HotkeyAction::NavigateNext,
        Hotkey {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::empty(),
        },
    );
    nav.insert(
        HotkeyAction::NavigatePrev,
        Hotkey {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::empty(),
        },
    );
    nav.insert(
        HotkeyAction::NavigateLeft,
        Hotkey {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::empty(),
        },
    );
    nav.insert(
        HotkeyAction::NavigateRight,
        Hotkey {
            code: KeyCode::Char('l'),
            modifiers: KeyModifiers::empty(),
        },
    );
    nav
}

/// Apply global navigation hotkeys to a view's hotkey map.
///
fn apply_global_navigation(map: &mut HashMap<HotkeyAction, Hotkey>) {
    let global_nav = global_navigation_hotkeys();
    for (action, hotkey) in global_nav {
        map.insert(action, hotkey);
    }
}

/// Default hotkey configurations.
///
pub fn default_hotkeys() -> ViewHotkeys {
    let mut welcome = HashMap::new();
    // Apply global navigation actions to all views
    apply_global_navigation(&mut welcome);
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
    // Apply global navigation actions
    apply_global_navigation(&mut project_tasks);
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
    project_tasks.insert(
        HotkeyAction::FilterByAssignee,
        Hotkey {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::empty(),
        },
    );

    let mut task_detail = HashMap::new();
    // Apply global navigation actions
    apply_global_navigation(&mut task_detail);
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
    // Apply global navigation actions
    apply_global_navigation(&mut create_task);
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
    // Apply global navigation actions
    apply_global_navigation(&mut debug_mode);
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
    // Apply global navigation actions
    apply_global_navigation(&mut move_task);
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
    // Global navigation actions work in theme selector too
    theme_selector.insert(
        HotkeyAction::NavigateNext,
        Hotkey {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::empty(),
        },
    );
    theme_selector.insert(
        HotkeyAction::NavigatePrev,
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

    let mut assignee_filter = HashMap::new();
    // Apply global navigation actions
    apply_global_navigation(&mut assignee_filter);
    assignee_filter.insert(
        HotkeyAction::AssigneeFilterSelect,
        Hotkey {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::empty(),
        },
    );
    assignee_filter.insert(
        HotkeyAction::AssigneeFilterCancel,
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
        assignee_filter,
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
        SpecialMode::AssigneeFilter => &hotkeys.assignee_filter,
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
    AssigneeFilter,
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

/// Builds hotkey instructions for move task modal.
/// Groups navigation keys together (e.g., "j/k: navigate").
///
pub fn build_move_task_instructions(hotkeys: &ViewHotkeys) -> String {
    let mut parts = Vec::new();

    // Group navigation keys
    let nav_next = hotkeys
        .move_task
        .get(&HotkeyAction::NavigateNext)
        .or_else(|| hotkeys.welcome.get(&HotkeyAction::NavigateNext));
    let nav_prev = hotkeys
        .move_task
        .get(&HotkeyAction::NavigatePrev)
        .or_else(|| hotkeys.welcome.get(&HotkeyAction::NavigatePrev));

    if let (Some(next), Some(prev)) = (nav_next, nav_prev) {
        parts.push(format!(
            "{}/{}: navigate",
            format_hotkey_display(next),
            format_hotkey_display(prev)
        ));
    }

    // Add select
    if let Some(select) = hotkeys
        .move_task
        .get(&HotkeyAction::Select)
        .or_else(|| hotkeys.welcome.get(&HotkeyAction::Select))
    {
        parts.push(format!("{}: select", format_hotkey_display(select)));
    }

    // Add cancel
    if let Some(cancel) = hotkeys
        .move_task
        .get(&HotkeyAction::Cancel)
        .or_else(|| hotkeys.welcome.get(&HotkeyAction::Cancel))
    {
        parts.push(format!("{}: cancel", format_hotkey_display(cancel)));
    }

    parts.join(", ")
}

/// Builds hotkey instructions for debug mode.
/// Groups navigation keys together (e.g., "j/k: navigate").
///
pub fn build_debug_mode_instructions(hotkeys: &ViewHotkeys) -> String {
    let mut parts = Vec::new();

    // Group navigation keys
    let nav_next = hotkeys
        .debug_mode
        .get(&HotkeyAction::NavigateNext)
        .or_else(|| hotkeys.welcome.get(&HotkeyAction::NavigateNext));
    let nav_prev = hotkeys
        .debug_mode
        .get(&HotkeyAction::NavigatePrev)
        .or_else(|| hotkeys.welcome.get(&HotkeyAction::NavigatePrev));

    if let (Some(next), Some(prev)) = (nav_next, nav_prev) {
        parts.push(format!(
            "{}/{}: navigate",
            format_hotkey_display(next),
            format_hotkey_display(prev)
        ));
    }

    // Add copy
    if let Some(copy) = hotkeys.debug_mode.get(&HotkeyAction::DebugModeCopyLog) {
        parts.push(format!("{}: copy", format_hotkey_display(copy)));
    }

    // Add exit (can be / or Esc)
    if let Some(exit) = hotkeys.debug_mode.get(&HotkeyAction::DebugModeExit) {
        parts.push(format!("{}: exit", format_hotkey_display(exit)));
    }

    parts.join(", ")
}

/// Builds hotkey instructions for hotkey editor.
/// Groups navigation keys together (e.g., "j/k: navigate").
///
pub fn build_hotkey_editor_instructions(hotkeys: &ViewHotkeys) -> String {
    let mut parts = Vec::new();

    // Group navigation keys
    let nav_next = hotkeys.welcome.get(&HotkeyAction::NavigateNext);
    let nav_prev = hotkeys.welcome.get(&HotkeyAction::NavigatePrev);

    if let (Some(next), Some(prev)) = (nav_next, nav_prev) {
        parts.push(format!(
            "{}/{}: navigate",
            format_hotkey_display(next),
            format_hotkey_display(prev)
        ));
    }

    // Add select (for edit)
    if let Some(select) = hotkeys.welcome.get(&HotkeyAction::Select) {
        parts.push(format!("{}: edit", format_hotkey_display(select)));
    }

    // Add cancel (for close)
    if let Some(cancel) = hotkeys.welcome.get(&HotkeyAction::Cancel) {
        parts.push(format!("{}: close", format_hotkey_display(cancel)));
    }

    parts.join(", ")
}

/// Builds hotkey instructions for custom field dropdowns.
/// Groups navigation keys together (e.g., "j/k: navigate").
///
pub fn build_custom_field_instructions(hotkeys: &ViewHotkeys) -> String {
    let mut parts = Vec::new();

    // Group navigation keys
    let nav_next = hotkeys
        .create_task
        .get(&HotkeyAction::NavigateNext)
        .or_else(|| hotkeys.edit_task.get(&HotkeyAction::NavigateNext))
        .or_else(|| hotkeys.welcome.get(&HotkeyAction::NavigateNext));
    let nav_prev = hotkeys
        .create_task
        .get(&HotkeyAction::NavigatePrev)
        .or_else(|| hotkeys.edit_task.get(&HotkeyAction::NavigatePrev))
        .or_else(|| hotkeys.welcome.get(&HotkeyAction::NavigatePrev));

    if let (Some(next), Some(prev)) = (nav_next, nav_prev) {
        parts.push(format!(
            "{}/{}: navigate",
            format_hotkey_display(next),
            format_hotkey_display(prev)
        ));
    }

    // Add select (for toggle)
    if let Some(select) = hotkeys
        .create_task
        .get(&HotkeyAction::Select)
        .or_else(|| hotkeys.edit_task.get(&HotkeyAction::Select))
        .or_else(|| hotkeys.welcome.get(&HotkeyAction::Select))
    {
        parts.push(format!("{}: toggle", format_hotkey_display(select)));
    }

    parts.join(", ")
}
