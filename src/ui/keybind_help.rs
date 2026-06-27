use std::borrow::Cow;

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::release_notes::release_notes_close_button_rect;
use super::scrollbar::{release_notes_scrollbar_rect, render_scrollbar};
use super::widgets::{
    modal_stack_areas, panel_contrast_fg, render_action_button, render_modal_header,
    render_modal_shell,
};
use crate::app::AppState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum KeybindHelpCommand {
    Help,
    Settings,
    Detach,
    ReloadConfig,
    OpenNotificationTarget,
    WorkspacePicker,
    OpenNavigator,
    NewWorkspace,
    NewWorktree,
    OpenWorktree,
    RemoveWorktree,
    RenameWorkspace,
    CloseWorkspace,
    PreviousWorkspace,
    NextWorkspace,
    PreviousAgent,
    NextAgent,
    NewTab,
    RenameTab,
    PreviousTab,
    NextTab,
    CloseTab,
    SplitVertical,
    SplitHorizontal,
    ClosePane,
    RenamePane,
    EditScrollback,
    CopyMode,
    Zoom,
    EnterResizeMode,
    ToggleSidebar,
    FocusPaneLeft,
    FocusPaneDown,
    FocusPaneUp,
    FocusPaneRight,
    CyclePaneNext,
    CyclePanePrevious,
    LastPane,
    CustomCommand(usize),
}

#[derive(Clone)]
pub(super) struct HelpEntry {
    pub(super) key: String,
    pub(super) label: Cow<'static, str>,
    pub(super) command: Option<KeybindHelpCommand>,
}
pub(super) type HelpGroup = (&'static str, Vec<HelpEntry>);

fn help_entry(key: impl Into<String>, label: &'static str) -> HelpEntry {
    HelpEntry {
        key: key.into(),
        label: Cow::Borrowed(label),
        command: None,
    }
}

fn command_entry(
    key: impl Into<String>,
    label: &'static str,
    command: KeybindHelpCommand,
) -> HelpEntry {
    HelpEntry {
        key: key.into(),
        label: Cow::Borrowed(label),
        command: Some(command),
    }
}

fn keybind_label(bindings: &crate::config::ActionKeybinds) -> String {
    bindings.label().unwrap_or_else(|| "unset".to_string())
}

fn indexed_label(bindings: &[crate::config::IndexedKeybind]) -> String {
    if bindings.is_empty() {
        return "unset".to_string();
    }

    let mut parts = Vec::new();
    let mut index = 0;
    while index < bindings.len() {
        if let Some(prefix) = indexed_range_prefix(&bindings[index..]) {
            parts.push(format!("{prefix}1..9"));
            index += 9;
        } else {
            parts.push(bindings[index].label.clone());
            index += 1;
        }
    }

    parts.join(" / ")
}

fn indexed_range_prefix(bindings: &[crate::config::IndexedKeybind]) -> Option<&str> {
    let run = bindings.get(..9)?;
    let prefix = run[0].label.strip_suffix('1')?;
    for (offset, binding) in run.iter().enumerate() {
        let digit = char::from(b'1' + offset as u8);
        if binding.label.strip_suffix(digit) != Some(prefix) {
            return None;
        }
    }
    Some(prefix)
}

fn keybind_help_matches_query(query: &str, text: &str) -> bool {
    let haystack = text.to_lowercase();
    query
        .to_lowercase()
        .split_whitespace()
        .all(|needle| haystack.contains(needle) || ordered_subsequence_matches(needle, &haystack))
}

fn ordered_subsequence_matches(needle: &str, haystack: &str) -> bool {
    let mut needle_chars = needle.chars();
    let Some(mut current) = needle_chars.next() else {
        return true;
    };

    for haystack_char in haystack.chars() {
        if haystack_char == current {
            let Some(next) = needle_chars.next() else {
                return true;
            };
            current = next;
        }
    }

    false
}

pub(super) fn keybind_help_groups(app: &AppState) -> Vec<HelpGroup> {
    let kb = &app.keybinds;
    let mut groups = Vec::new();

    groups.push((
        "global",
        vec![
            command_entry(
                keybind_label(&kb.help),
                "keybinds",
                KeybindHelpCommand::Help,
            ),
            command_entry(
                keybind_label(&kb.settings),
                "settings",
                KeybindHelpCommand::Settings,
            ),
            command_entry(
                keybind_label(&kb.detach),
                "detach",
                KeybindHelpCommand::Detach,
            ),
            command_entry(
                keybind_label(&kb.reload_config),
                "reload config",
                KeybindHelpCommand::ReloadConfig,
            ),
            command_entry(
                keybind_label(&kb.open_notification_target),
                "open notification target",
                KeybindHelpCommand::OpenNotificationTarget,
            ),
        ],
    ));

    groups.push((
        "navigation",
        vec![
            help_entry("esc", "back"),
            help_entry(
                format!(
                    "{} / {}",
                    keybind_label(&kb.navigate.workspace_up),
                    keybind_label(&kb.navigate.workspace_down)
                ),
                "workspace list",
            ),
            help_entry(
                format!(
                    "{} / {} / {} / {} / left / right",
                    keybind_label(&kb.navigate.pane_left),
                    keybind_label(&kb.navigate.pane_down),
                    keybind_label(&kb.navigate.pane_up),
                    keybind_label(&kb.navigate.pane_right)
                ),
                "move focus",
            ),
            help_entry("tab / shift+tab", "cycle pane"),
            help_entry("enter", "open workspace"),
            help_entry("1..9", "switch workspace"),
        ],
    ));

    let workspace_tab = vec![
        command_entry(
            keybind_label(&kb.workspace_picker),
            "workspace navigation",
            KeybindHelpCommand::WorkspacePicker,
        ),
        command_entry(
            keybind_label(&kb.goto),
            "session navigator",
            KeybindHelpCommand::OpenNavigator,
        ),
        command_entry(
            keybind_label(&kb.new_workspace),
            "new workspace",
            KeybindHelpCommand::NewWorkspace,
        ),
        command_entry(
            keybind_label(&kb.new_worktree),
            "new worktree",
            KeybindHelpCommand::NewWorktree,
        ),
        command_entry(
            keybind_label(&kb.open_worktree),
            "open worktree",
            KeybindHelpCommand::OpenWorktree,
        ),
        command_entry(
            keybind_label(&kb.remove_worktree),
            "delete worktree checkout",
            KeybindHelpCommand::RemoveWorktree,
        ),
        command_entry(
            keybind_label(&kb.rename_workspace),
            "rename workspace",
            KeybindHelpCommand::RenameWorkspace,
        ),
        command_entry(
            keybind_label(&kb.close_workspace),
            "close workspace",
            KeybindHelpCommand::CloseWorkspace,
        ),
        command_entry(
            keybind_label(&kb.previous_workspace),
            "previous workspace",
            KeybindHelpCommand::PreviousWorkspace,
        ),
        command_entry(
            keybind_label(&kb.next_workspace),
            "next workspace",
            KeybindHelpCommand::NextWorkspace,
        ),
        help_entry(indexed_label(&kb.switch_workspace), "switch workspace 1-9"),
        command_entry(
            keybind_label(&kb.previous_agent),
            "previous agent",
            KeybindHelpCommand::PreviousAgent,
        ),
        command_entry(
            keybind_label(&kb.next_agent),
            "next agent",
            KeybindHelpCommand::NextAgent,
        ),
        help_entry(indexed_label(&kb.focus_agent), "focus agent 1-9"),
        command_entry(
            keybind_label(&kb.new_tab),
            "new tab",
            KeybindHelpCommand::NewTab,
        ),
        command_entry(
            keybind_label(&kb.rename_tab),
            "rename tab",
            KeybindHelpCommand::RenameTab,
        ),
        command_entry(
            keybind_label(&kb.previous_tab),
            "previous tab",
            KeybindHelpCommand::PreviousTab,
        ),
        command_entry(
            keybind_label(&kb.next_tab),
            "next tab",
            KeybindHelpCommand::NextTab,
        ),
        help_entry(keybind_label(&kb.move_tab_previous), "move tab left"),
        help_entry(keybind_label(&kb.move_tab_next), "move tab right"),
        help_entry(indexed_label(&kb.switch_tab), "switch tab 1-9"),
        command_entry(
            keybind_label(&kb.close_tab),
            "close tab",
            KeybindHelpCommand::CloseTab,
        ),
    ];
    groups.push(("workspaces / tabs", workspace_tab));

    let panes = vec![
        command_entry(
            keybind_label(&kb.split_vertical),
            "split vertical",
            KeybindHelpCommand::SplitVertical,
        ),
        command_entry(
            keybind_label(&kb.split_horizontal),
            "split horizontal",
            KeybindHelpCommand::SplitHorizontal,
        ),
        command_entry(
            keybind_label(&kb.close_pane),
            "close pane",
            KeybindHelpCommand::ClosePane,
        ),
        command_entry(
            keybind_label(&kb.rename_pane),
            "rename pane",
            KeybindHelpCommand::RenamePane,
        ),
        command_entry(
            keybind_label(&kb.edit_scrollback),
            "edit scrollback",
            KeybindHelpCommand::EditScrollback,
        ),
        command_entry(
            keybind_label(&kb.copy_mode),
            "copy mode",
            KeybindHelpCommand::CopyMode,
        ),
        command_entry(
            keybind_label(&kb.zoom),
            "zoom pane",
            KeybindHelpCommand::Zoom,
        ),
        command_entry(
            keybind_label(&kb.resize_mode),
            "resize mode",
            KeybindHelpCommand::EnterResizeMode,
        ),
        help_entry(keybind_label(&kb.resize_pane_left), "resize pane left"),
        help_entry(keybind_label(&kb.resize_pane_down), "resize pane down"),
        help_entry(keybind_label(&kb.resize_pane_up), "resize pane up"),
        help_entry(keybind_label(&kb.resize_pane_right), "resize pane right"),
        command_entry(
            keybind_label(&kb.toggle_sidebar),
            "toggle sidebar",
            KeybindHelpCommand::ToggleSidebar,
        ),
        command_entry(
            keybind_label(&kb.focus_pane_left),
            "focus pane left",
            KeybindHelpCommand::FocusPaneLeft,
        ),
        command_entry(
            keybind_label(&kb.focus_pane_down),
            "focus pane down",
            KeybindHelpCommand::FocusPaneDown,
        ),
        command_entry(
            keybind_label(&kb.focus_pane_up),
            "focus pane up",
            KeybindHelpCommand::FocusPaneUp,
        ),
        command_entry(
            keybind_label(&kb.focus_pane_right),
            "focus pane right",
            KeybindHelpCommand::FocusPaneRight,
        ),
        command_entry(
            keybind_label(&kb.cycle_pane_next),
            "cycle pane next",
            KeybindHelpCommand::CyclePaneNext,
        ),
        command_entry(
            keybind_label(&kb.cycle_pane_previous),
            "cycle pane previous",
            KeybindHelpCommand::CyclePanePrevious,
        ),
        command_entry(
            keybind_label(&kb.last_pane),
            "last pane",
            KeybindHelpCommand::LastPane,
        ),
    ];
    groups.push(("panes", panes));

    if !kb.custom_commands.is_empty() {
        groups.push((
            "custom",
            kb.custom_commands
                .iter()
                .enumerate()
                .map(|(index, binding)| HelpEntry {
                    key: binding.label.clone(),
                    label: binding
                        .description
                        .clone()
                        .map(Cow::Owned)
                        .unwrap_or(Cow::Borrowed("custom command")),
                    command: Some(KeybindHelpCommand::CustomCommand(index)),
                })
                .collect(),
        ));
    }

    groups
}

fn filtered_keybind_help_groups(app: &AppState) -> Vec<HelpGroup> {
    let query = app.keybind_help.query.trim();
    let groups = keybind_help_groups(app);
    if query.is_empty() {
        return groups;
    }

    let mut command_groups = Vec::new();
    let mut references = Vec::new();
    for (group, entries) in groups {
        let mut commands = Vec::new();
        for entry in entries {
            if !keybind_help_matches_query(query, &format!("{} {}", entry.key, entry.label)) {
                continue;
            }
            if entry.command.is_some() {
                commands.push(entry);
            } else {
                references.push(entry);
            }
        }
        if !commands.is_empty() {
            command_groups.push((group, commands));
        }
    }
    if !references.is_empty() {
        command_groups.push(("reference only", references));
    }
    command_groups
}

fn keybind_help_row_commands(app: &AppState) -> Vec<Option<KeybindHelpCommand>> {
    filtered_keybind_help_groups(app)
        .into_iter()
        .flat_map(|(_, entries)| entries.into_iter().map(|entry| entry.command))
        .collect()
}

pub(crate) fn keybind_help_selectable_count(app: &AppState) -> usize {
    keybind_help_row_commands(app)
        .into_iter()
        .filter(Option::is_some)
        .count()
}

pub(crate) fn keybind_help_selected_command(app: &AppState) -> Option<KeybindHelpCommand> {
    keybind_help_row_commands(app)
        .into_iter()
        .flatten()
        .nth(app.keybind_help.selected)
}

pub(crate) fn keybind_help_selectable_ordinal_for_row(
    app: &AppState,
    target_row_index: usize,
) -> Option<usize> {
    let rows = keybind_help_row_commands(app);
    rows.get(target_row_index)?.as_ref()?;
    Some(
        rows[..target_row_index]
            .iter()
            .filter(|command| command.is_some())
            .count(),
    )
}

pub(crate) fn keybind_help_row_index_at_line(
    app: &AppState,
    target_line_index: usize,
) -> Option<usize> {
    let mut line_index = 0usize;
    let mut row_index = 0usize;
    for (_, entries) in filtered_keybind_help_groups(app) {
        if line_index == target_line_index {
            return None;
        }
        line_index += 1;
        for _ in entries {
            if line_index == target_line_index {
                return Some(row_index);
            }
            row_index += 1;
            line_index += 1;
        }
        line_index += 1;
    }
    None
}

pub(crate) fn keybind_help_selected_line_index(app: &AppState) -> Option<usize> {
    let selected = app.keybind_help.selected;
    let mut line_index = 0usize;
    let mut selectable_index = 0usize;
    for (_, entries) in filtered_keybind_help_groups(app) {
        line_index += 1;
        for entry in entries {
            if entry.command.is_some() {
                if selectable_index == selected {
                    return Some(line_index);
                }
                selectable_index += 1;
            }
            line_index += 1;
        }
        line_index += 1;
    }
    None
}

pub(crate) fn keybind_help_lines(app: &AppState) -> Vec<(usize, Line<'static>)> {
    let heading_style = Style::default()
        .fg(app.palette.accent)
        .add_modifier(Modifier::BOLD);
    let key_style = Style::default()
        .fg(app.palette.mauve)
        .add_modifier(Modifier::BOLD);
    let label_style = Style::default().fg(app.palette.text);

    let selected_command_style = Style::default()
        .fg(panel_contrast_fg(&app.palette))
        .bg(app.palette.accent)
        .add_modifier(Modifier::BOLD);
    let groups = filtered_keybind_help_groups(app);
    let key_width = groups
        .iter()
        .flat_map(|(_, entries)| entries.iter().map(|entry| entry.key.chars().count()))
        .max()
        .unwrap_or(8);

    let mut lines = Vec::new();
    let mut selectable_index = 0usize;

    for (group, entries) in groups {
        lines.push((
            group.len() + 1,
            Line::from(vec![Span::styled(format!(" {group}"), heading_style)]),
        ));
        for entry in entries {
            let executable = entry.command.is_some();
            let selected = executable && selectable_index == app.keybind_help.selected;
            let reference_style = Style::default().fg(app.palette.overlay1);
            let key_style = if selected {
                selected_command_style
            } else if executable {
                key_style
            } else {
                reference_style
            };
            let label_style = if selected {
                selected_command_style
            } else if executable {
                label_style
            } else {
                reference_style
            };
            let marker = if selected { ">" } else { " " };
            let padded_key = format!("{marker}{:<width$} ", entry.key, width = key_width);
            let width = padded_key.chars().count() + entry.label.chars().count();
            lines.push((
                width,
                Line::from(vec![
                    Span::styled(padded_key, key_style),
                    Span::styled(entry.label.into_owned(), label_style),
                ]),
            ));
            if executable {
                selectable_index += 1;
            }
        }
        lines.push((0, Line::raw("")));
    }

    if lines.is_empty() {
        lines.push((
            18,
            Line::from(vec![Span::styled(
                " no commands match",
                Style::default().fg(app.palette.overlay1),
            )]),
        ));
    }

    lines
}

fn render_keybind_help_search(app: &AppState, frame: &mut Frame, area: Rect) {
    let query = app.keybind_help.query.trim();
    let shown = filtered_keybind_help_groups(app)
        .iter()
        .map(|(_, entries)| entries.len())
        .sum::<usize>();
    let summary = if query.is_empty() {
        format!("{shown} items")
    } else {
        format!("{shown} matches")
    };
    let mut spans = Vec::new();
    if !app.keybind_help.search_focused {
        spans.push(Span::styled(
            format!(" press / to filter from {shown} items"),
            Style::default().fg(app.palette.overlay0),
        ));
    } else {
        spans.push(Span::styled(
            format!(" / {query}"),
            Style::default().fg(app.palette.text),
        ));
        spans.push(Span::styled(
            "  ",
            Style::default().fg(app.palette.overlay0),
        ));
        spans.push(Span::styled(
            summary,
            Style::default().fg(app.palette.overlay0),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

pub(super) fn render_keybind_help_overlay(app: &AppState, frame: &mut Frame) {
    super::dim_background(frame, frame.area());

    let Some(inner) = render_modal_shell(frame, frame.area(), 76, 22, &app.palette) else {
        return;
    };
    if inner.height < 6 || inner.width < 20 {
        return;
    }

    let stack = modal_stack_areas(inner, 3, 1, 0, 1);
    let header_rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas::<3>(stack.header);

    render_modal_header(frame, header_rows[0], "keybinds", &app.palette);
    render_action_button(
        frame,
        release_notes_close_button_rect(header_rows[0]),
        Some("esc"),
        if app.keybind_help.search_focused {
            "back"
        } else {
            "close"
        },
        Style::default()
            .fg(panel_contrast_fg(&app.palette))
            .bg(app.palette.accent)
            .add_modifier(Modifier::BOLD),
    );
    render_keybind_help_search(app, frame, header_rows[1]);
    frame.render_widget(
        Paragraph::new(" available commands and configured shortcuts")
            .style(Style::default().fg(app.palette.overlay1)),
        header_rows[2],
    );

    let body_area = stack.content;
    let metrics = crate::pane::ScrollMetrics {
        offset_from_bottom: app
            .keybind_help_max_scroll()
            .saturating_sub(app.keybind_help.scroll) as usize,
        max_offset_from_bottom: app.keybind_help_max_scroll() as usize,
        viewport_rows: body_area.height.max(1) as usize,
    };
    let track = release_notes_scrollbar_rect(body_area, metrics);
    let text_area = track
        .map(|_| {
            Rect::new(
                body_area.x,
                body_area.y,
                body_area.width.saturating_sub(1),
                body_area.height,
            )
        })
        .unwrap_or(body_area);

    let body = Paragraph::new(
        keybind_help_lines(app)
            .into_iter()
            .map(|(_, line)| line)
            .collect::<Vec<_>>(),
    )
    .scroll((app.keybind_help.scroll, 0));
    frame.render_widget(body, text_area);
    if let Some(track) = track {
        render_scrollbar(
            frame,
            metrics,
            track,
            app.palette.overlay0,
            app.palette.overlay1,
            "▐",
        );
    }

    let footer = if app.keybind_help.search_focused {
        Line::from(vec![
            Span::styled(" filter ", Style::default().fg(app.palette.overlay0)),
            Span::styled("type/backspace", Style::default().fg(app.palette.text)),
            Span::styled(" · ", Style::default().fg(app.palette.overlay0)),
            Span::styled("select ", Style::default().fg(app.palette.overlay0)),
            Span::styled("↑↓", Style::default().fg(app.palette.text)),
            Span::styled(" · ", Style::default().fg(app.palette.overlay0)),
            Span::styled("run ", Style::default().fg(app.palette.overlay0)),
            Span::styled("enter", Style::default().fg(app.palette.text)),
            Span::styled(" · ", Style::default().fg(app.palette.overlay0)),
            Span::styled("back ", Style::default().fg(app.palette.overlay0)),
            Span::styled("esc", Style::default().fg(app.palette.text)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" search ", Style::default().fg(app.palette.overlay0)),
            Span::styled("/", Style::default().fg(app.palette.text)),
            Span::styled(" · ", Style::default().fg(app.palette.overlay0)),
            Span::styled("select ", Style::default().fg(app.palette.overlay0)),
            Span::styled("j/k/↑↓", Style::default().fg(app.palette.text)),
            Span::styled(" · ", Style::default().fg(app.palette.overlay0)),
            Span::styled("run ", Style::default().fg(app.palette.overlay0)),
            Span::styled("enter", Style::default().fg(app.palette.text)),
            Span::styled(" · ", Style::default().fg(app.palette.overlay0)),
            Span::styled("close ", Style::default().fg(app.palette.overlay0)),
            Span::styled("esc", Style::default().fg(app.palette.text)),
        ])
    };
    frame.render_widget(Paragraph::new(footer), stack.footer.unwrap_or_default());
}
