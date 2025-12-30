use super::form_dropdowns;
use super::Frame;
use crate::asana::CustomField;
use crate::state::{CustomFieldValue, EditFormState, State};
use crate::ui::widgets::styling;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Render task creation form.
///
pub fn create_task(frame: &mut Frame, size: Rect, state: &mut State) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // Form fields
            Constraint::Length(1), // Footer
        ])
        .split(size);

    let title_block = Block::default()
        .borders(Borders::ALL)
        .title("Create New Task");
    let title = Paragraph::new("Create New Task")
        .block(title_block)
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    let form_state = state.get_edit_form_state().unwrap_or(EditFormState::Name);
    // Clone custom fields early to avoid borrow checker issues
    // Filter out custom_id fields (they cannot be manually edited)
    let custom_fields: Vec<CustomField> = state
        .get_project_custom_fields()
        .iter()
        .filter(|cf| {
            // Skip custom_id fields
            let is_custom_id = cf
                .representation_type
                .as_ref()
                .map(|s| s == "custom_id")
                .unwrap_or(false)
                || cf.id_prefix.is_some();
            !is_custom_id
        })
        .cloned()
        .collect();

    // Calculate which fields to show based on available height
    // Fields have different heights: Name/Assignee/DueDate/Section = 3, Notes = 5, Custom dropdowns = 7+
    let available_height = chunks[1].height;
    let is_editing = state.is_field_editing_mode();

    // Build list of all fields with their heights
    let all_fields: Vec<(usize, &str, Option<usize>, u16)> = {
        let assignee_height = if is_editing && matches!(form_state, EditFormState::Assignee) {
            10 // Search (3) + dropdown (7)
        } else {
            3
        };
        let section_height = if is_editing && matches!(form_state, EditFormState::Section) {
            10 // Search (3) + dropdown (7)
        } else {
            3
        };
        let mut fields = vec![
            (0, "Name", None, 3),
            (1, "Notes", None, 5),
            (2, "Assignee", None, assignee_height),
            (3, "DueDate", None, 3),
            (4, "Section", None, section_height),
        ];
        for (idx, cf) in custom_fields.iter().enumerate() {
            let height = match cf.resource_subtype.as_str() {
                "enum" | "multi_enum" | "people" => {
                    // Dynamic height: 10 when editing, 3 when not
                    if is_editing && matches!(form_state, EditFormState::CustomField(i) if i == idx)
                    {
                        10 // Search (3) + dropdown (7)
                    } else {
                        3
                    }
                }
                _ => 3,
            };
            fields.push((5 + idx, "CustomField", Some(idx), height));
        }
        fields
    };

    // Find the index of the currently selected field
    let current_field_idx = match form_state {
        EditFormState::Name => 0,
        EditFormState::Notes => 1,
        EditFormState::Assignee => 2,
        EditFormState::DueDate => 3,
        EditFormState::Section => 4,
        EditFormState::CustomField(cf_idx) => 5 + cf_idx,
    };

    // Calculate visible range centered around current field (like kanban board)
    // Start by trying to center the current field, then fill available space
    let mut start_idx = 0;
    let mut end_idx = 0;
    let mut cumulative_height = 0u16;

    // First, try to find a start position that centers the current field
    // We'll work backwards from the current field to find where we can start
    let mut height_before_current = 0u16;

    // Calculate height before current field
    for idx in (0..current_field_idx).rev() {
        if let Some((_, _, _, height)) = all_fields.get(idx) {
            if height_before_current + *height <= available_height / 2 {
                height_before_current += *height;
                start_idx = idx;
            } else {
                break;
            }
        }
    }

    // Now fill forward from start_idx until we run out of space
    for (idx, (_, _, _, height)) in all_fields.iter().enumerate().skip(start_idx) {
        if cumulative_height + *height > available_height.saturating_sub(2) {
            // Not enough space for this field
            break;
        }
        cumulative_height += *height;
        end_idx = idx + 1;
    }

    // Ensure current field is visible
    if current_field_idx < start_idx {
        start_idx = current_field_idx;
        // Recalculate end_idx from new start
        cumulative_height = 0;
        for (idx, (_, _, _, height)) in all_fields.iter().enumerate().skip(start_idx) {
            if cumulative_height + *height > available_height.saturating_sub(2) {
                break;
            }
            cumulative_height += *height;
            end_idx = idx + 1;
        }
    } else if current_field_idx >= end_idx {
        // Current field is after end, need to scroll forward
        // Calculate backwards from current field
        cumulative_height = 0;
        end_idx = current_field_idx + 1;
        for idx in (0..=current_field_idx).rev() {
            if let Some((_, _, _, height)) = all_fields.get(idx) {
                if cumulative_height + *height > available_height.saturating_sub(2) {
                    break;
                }
                cumulative_height += *height;
                start_idx = idx;
            }
        }
    }

    // Ensure we show at least one field
    if end_idx <= start_idx && start_idx < all_fields.len() {
        end_idx = start_idx + 1;
    }

    let visible_fields = &all_fields[start_idx..end_idx.min(all_fields.len())];

    // Build constraints for visible fields only using their actual heights
    let constraints: Vec<Constraint> = visible_fields
        .iter()
        .map(|(_, _, _, height)| Constraint::Length(*height))
        .collect();

    // Render form fields
    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(chunks[1]);

    // Render only visible fields
    for (chunk_idx, (_field_idx, field_type, custom_idx, _)) in visible_fields.iter().enumerate() {
        if chunk_idx >= form_chunks.len() {
            break;
        }

        match *field_type {
            "Name" => {
                render_field(
                    frame,
                    form_chunks[chunk_idx],
                    "Name",
                    state.get_form_name(),
                    form_state == EditFormState::Name,
                    is_editing && form_state == EditFormState::Name,
                );
            }
            "Notes" => {
                render_notes_field(
                    frame,
                    form_chunks[chunk_idx],
                    state,
                    form_state == EditFormState::Notes,
                    is_editing && form_state == EditFormState::Notes,
                );
            }
            "Assignee" => {
                // Show dropdown only when in editing mode, like custom fields
                if is_editing && form_state == EditFormState::Assignee {
                    form_dropdowns::render_assignee_dropdown(frame, form_chunks[chunk_idx], state);
                } else {
                    // Show simple field when not editing
                    let assignee_text = if let Some(assignee_gid) = state.get_form_assignee() {
                        state
                            .get_workspace_users()
                            .iter()
                            .find(|u| u.gid == *assignee_gid)
                            .map(|u| {
                                if !u.email.is_empty() {
                                    format!("{} ({})", u.name, u.email)
                                } else {
                                    u.name.clone()
                                }
                            })
                            .unwrap_or_else(|| "Unknown".to_string())
                    } else {
                        "None".to_string()
                    };
                    render_field(
                        frame,
                        form_chunks[chunk_idx],
                        "Assignee (dropdown)",
                        &assignee_text,
                        form_state == EditFormState::Assignee,
                        false,
                    );
                }
            }
            "DueDate" => {
                render_field(
                    frame,
                    form_chunks[chunk_idx],
                    "Due Date (YYYY-MM-DD)",
                    state.get_form_due_on(),
                    form_state == EditFormState::DueDate,
                    is_editing && form_state == EditFormState::DueDate,
                );
            }
            "Section" => {
                // Show dropdown only when in editing mode, like custom fields
                if is_editing && form_state == EditFormState::Section {
                    form_dropdowns::render_section_dropdown(frame, form_chunks[chunk_idx], state);
                } else {
                    // Show simple field when not editing
                    let section_text = if let Some(section_gid) = state.get_form_section() {
                        state
                            .get_sections()
                            .iter()
                            .find(|s| s.gid == *section_gid)
                            .map(|s| s.name.as_str())
                            .unwrap_or("Unknown")
                    } else {
                        "None"
                    };
                    render_field(
                        frame,
                        form_chunks[chunk_idx],
                        "Section (dropdown)",
                        section_text,
                        form_state == EditFormState::Section,
                        false,
                    );
                }
            }
            "CustomField" => {
                if let Some(cf_idx) = custom_idx {
                    if let Some(cf) = custom_fields.get(*cf_idx) {
                        let is_selected =
                            matches!(form_state, EditFormState::CustomField(i) if i == *cf_idx);
                        let is_editing_field = is_editing && is_selected;
                        let cf_gid = cf.gid.clone();
                        let value_clone = state.get_custom_field_value(&cf_gid).cloned();
                        render_custom_field_inner(
                            frame,
                            form_chunks[chunk_idx],
                            cf,
                            value_clone,
                            state,
                            is_selected,
                            is_editing_field,
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

fn render_field(
    frame: &mut Frame,
    size: Rect,
    label: &str,
    value: &str,
    is_selected: bool,
    is_editing: bool,
) {
    // Different styles for navigation vs editing
    let (border_style, title) = if is_editing {
        // EDITING: Yellow border and indicator
        (
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            format!("{} [EDITING]", label),
        )
    } else if is_selected {
        // SELECTED (Navigation mode): Cyan border
        (
            styling::active_block_border_style(), // Cyan
            format!("{} [Press Enter to edit]", label),
        )
    } else {
        // Not selected: Normal border
        (styling::normal_block_border_style(), label.to_string())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    let display_value = if value.is_empty() {
        if is_editing {
            "Type to enter value..."
        } else {
            "Empty"
        }
    } else {
        value
    };

    let text_style = if is_editing {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else if is_selected {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        styling::normal_text_style()
    };

    let text = if is_editing {
        Line::from(vec![
            Span::styled(display_value, text_style),
            Span::styled(" █", Style::default().fg(Color::Yellow)), // Editing cursor
        ])
    } else if is_selected {
        Line::from(vec![
            Span::styled("▸ ", Style::default().fg(Color::Cyan)), // Navigation indicator
            Span::styled(display_value, text_style),
        ])
    } else {
        Line::from(vec![Span::styled(display_value, text_style)])
    };

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, size);
}

fn render_notes_field(
    frame: &mut Frame,
    size: Rect,
    state: &mut State,
    is_selected: bool,
    is_editing: bool,
) {
    let (border_style, title) = if is_editing {
        // EDITING: Yellow border and indicator
        (
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            "Notes [EDITING - Esc to exit]",
        )
    } else if is_selected {
        // SELECTED (Navigation mode): Cyan border
        (
            styling::active_block_border_style(), // Cyan
            "Notes [Press Enter to edit]",
        )
    } else {
        // Not selected: Normal border
        (styling::normal_block_border_style(), "Notes")
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    // Get mutable access to textarea from state
    let textarea = state.get_form_notes_textarea();

    // Apply block styling
    textarea.set_block(block);

    // Render the textarea
    frame.render_widget(textarea.widget(), size);
}

fn render_custom_field_inner(
    frame: &mut Frame,
    size: Rect,
    cf: &CustomField,
    value: Option<CustomFieldValue>,
    state: &State,
    is_selected: bool,
    is_editing: bool,
) {
    match cf.resource_subtype.as_str() {
        "text" => {
            let text_value = match &value {
                Some(CustomFieldValue::Text(s)) => s.clone(),
                _ => String::new(),
            };
            render_field(frame, size, &cf.name, &text_value, is_selected, is_editing);
        }
        "number" => {
            let num_value = match &value {
                Some(CustomFieldValue::Number(Some(n))) => n.to_string(),
                Some(CustomFieldValue::Number(None)) => String::new(),
                _ => String::new(),
            };
            render_field(
                frame,
                size,
                &format!("{} (number)", cf.name),
                &num_value,
                is_selected,
                is_editing,
            );
        }
        "date" => {
            let date_value = match &value {
                Some(CustomFieldValue::Date(Some(d))) => d.clone(),
                Some(CustomFieldValue::Date(None)) => String::new(),
                _ => String::new(),
            };
            render_field(
                frame,
                size,
                &format!("{} (YYYY-MM-DD)", cf.name),
                &date_value,
                is_selected,
                is_editing,
            );
        }
        "enum" => {
            if is_editing {
                form_dropdowns::render_enum_dropdown(frame, size, cf, &cf.gid, state);
            } else {
                let selected_text = match &value {
                    Some(CustomFieldValue::Enum(Some(gid))) => cf
                        .enum_options
                        .iter()
                        .find(|eo| eo.gid == *gid)
                        .map(|eo| eo.name.as_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                    _ => "None".to_string(),
                };
                render_field(
                    frame,
                    size,
                    &format!("{} (dropdown)", cf.name),
                    &selected_text,
                    is_selected,
                    false,
                );
            }
        }
        "multi_enum" => {
            if is_editing {
                render_multi_enum_dropdown(frame, size, cf, &cf.gid, state);
            } else {
                let selected = match &value {
                    Some(CustomFieldValue::MultiEnum(gids)) => cf
                        .enum_options
                        .iter()
                        .filter(|eo| gids.contains(&eo.gid))
                        .map(|eo| eo.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    _ => String::new(),
                };
                render_field(
                    frame,
                    size,
                    &format!("{} (multi-select)", cf.name),
                    if selected.is_empty() {
                        "None"
                    } else {
                        &selected
                    },
                    is_selected,
                    false,
                );
            }
        }
        "people" => {
            if is_editing {
                render_people_dropdown(frame, size, cf, &cf.gid, state);
            } else {
                let selected_text = match &value {
                    Some(CustomFieldValue::People(gids)) => {
                        if gids.is_empty() {
                            "None".to_string()
                        } else {
                            format!("{} selected", gids.len())
                        }
                    }
                    _ => "None".to_string(),
                };
                render_field(
                    frame,
                    size,
                    &format!("{} (people)", cf.name),
                    &selected_text,
                    is_selected,
                    false,
                );
            }
        }
        _ => {
            render_field(
                frame,
                size,
                &cf.name,
                "Unsupported type",
                is_selected,
                false,
            );
        }
    }
}

fn render_multi_enum_dropdown(
    frame: &mut Frame,
    size: Rect,
    cf: &CustomField,
    cf_gid: &str,
    state: &State,
) {
    let search = state.get_custom_field_search(cf_gid).to_string();
    let filtered: Vec<_> = cf
        .enum_options
        .iter()
        .filter(|eo| {
            eo.enabled
                && (search.is_empty() || eo.name.to_lowercase().contains(&search.to_lowercase()))
        })
        .collect();

    let current_idx = state.get_custom_field_dropdown_index(cf_gid);
    let selected_idx = current_idx.min(filtered.len().saturating_sub(1));

    let selected_gids: Vec<String> = match state.get_custom_field_value(cf_gid) {
        Some(CustomFieldValue::MultiEnum(gids)) => gids.clone(),
        _ => vec![],
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(5),    // Dropdown list
        ])
        .split(size);

    // Search input
    let search_text = if search.is_empty() {
        "Type to search...".to_string()
    } else {
        search
    };
    render_field(
        frame,
        chunks[0],
        &format!("{} (multi-select, search)", cf.name),
        &search_text,
        true,
        true,
    );

    // Dropdown list with checkboxes
    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, eo)| {
            let is_selected = selected_gids.contains(&eo.gid);
            let prefix = if is_selected { "[✓] " } else { "[ ] " };
            let style = if i == selected_idx {
                styling::active_list_item_style()
            } else {
                styling::normal_text_style()
            };
            ListItem::new(format!("{}{}", prefix, eo.name)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Options ({} found, {} selected)",
            filtered.len(),
            selected_gids.len()
        )))
        .style(styling::normal_text_style())
        .highlight_style(styling::active_list_item_style());

    frame.render_widget(list, chunks[1]);
}

fn render_people_dropdown(
    frame: &mut Frame,
    size: Rect,
    cf: &CustomField,
    cf_gid: &str,
    state: &State,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input (same size as original field)
            Constraint::Min(7), // Dropdown list (max 5 items + borders, but allow more if space available)
        ])
        .split(size);

    // Search input - use same pattern as assignee/sections
    let search_text = state.get_custom_field_search(cf_gid);
    let search_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Search {} (people)", cf.name))
        .border_style(styling::active_block_border_style());
    let search_para = Paragraph::new(format!("> {}", search_text))
        .block(search_block)
        .style(styling::normal_text_style());
    frame.render_widget(search_para, chunks[0]);

    // Filtered users list - limit to max 5 visible items
    let users = state.get_workspace_users();
    let filtered: Vec<_> = users
        .iter()
        .filter(|u| {
            search_text.is_empty() || u.name.to_lowercase().contains(&search_text.to_lowercase())
        })
        .collect();
    let selected_index = state.get_custom_field_dropdown_index(cf_gid);

    let selected_gids: Vec<String> = match state.get_custom_field_value(cf_gid) {
        Some(CustomFieldValue::People(gids)) => gids.clone(),
        _ => vec![],
    };

    // Calculate visible range (show max 5 items, centered around selected)
    let max_visible = 5;
    let total_items = filtered.len();
    let start_index = if total_items <= max_visible {
        0
    } else {
        (selected_index as i32 - max_visible as i32 / 2)
            .max(0)
            .min((total_items - max_visible) as i32) as usize
    };
    let end_index = (start_index + max_visible).min(total_items);
    let visible_users = &filtered[start_index..end_index];
    let visible_selected = selected_index.saturating_sub(start_index);

    // Dropdown list with checkboxes
    let items: Vec<ListItem> = visible_users
        .iter()
        .map(|u| {
            let is_selected = selected_gids.contains(&u.gid);
            let prefix = if is_selected { "[✓] " } else { "[ ] " };
            let display_text = if !u.email.is_empty() {
                format!("{}{} ({})", prefix, u.name, u.email)
            } else {
                format!("{}{}", prefix, u.name)
            };
            ListItem::new(display_text)
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            "{} ({} results, {} selected, ↑ / ↓ to navigate, Enter to toggle)",
            cf.name,
            filtered.len(),
            selected_gids.len()
        ))
        .border_style(styling::active_block_border_style());

    // Use ListState for proper selection display
    let mut list_state = ratatui::widgets::ListState::default();
    if !items.is_empty() {
        list_state.select(Some(visible_selected.min(items.len().saturating_sub(1))));
    }

    let list = List::new(items)
        .block(block)
        .style(styling::normal_text_style())
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, chunks[1], &mut list_state);
}
