//! Custom widgets for the TUI installer

#![allow(dead_code)]

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Widget, Wrap},
};

/// A selectable list with highlight support
pub struct SelectableList<'a> {
    items: Vec<ListItem<'a>>,
    title: &'a str,
    state: &'a mut ListState,
}

impl<'a> SelectableList<'a> {
    pub fn new(items: Vec<(&'a str, &'a str)>, title: &'a str, state: &'a mut ListState) -> Self {
        let list_items: Vec<ListItem> = items
            .iter()
            .map(|(name, desc)| {
                let lines = vec![
                    Line::from(Span::styled(
                        *name,
                        Style::default().add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled(
                        format!("  {}", desc),
                        Style::default().fg(Color::DarkGray),
                    )),
                ];
                ListItem::new(lines)
            })
            .collect();
        Self {
            items: list_items,
            title,
            state,
        }
    }

    pub fn render(self, area: Rect, buf: &mut Buffer) {
        let list = List::new(self.items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.title)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        ratatui::widgets::StatefulWidget::render(list, area, buf, self.state);
    }
}

/// A simple selectable list (single line items)
pub struct SimpleSelectableList<'a> {
    items: Vec<&'a str>,
    title: &'a str,
    state: &'a mut ListState,
}

impl<'a> SimpleSelectableList<'a> {
    pub fn new(items: Vec<&'a str>, title: &'a str, state: &'a mut ListState) -> Self {
        Self {
            items,
            title,
            state,
        }
    }

    pub fn render(self, area: Rect, buf: &mut Buffer) {
        let list_items: Vec<ListItem> = self
            .items
            .iter()
            .map(|item| ListItem::new(Line::from(*item)))
            .collect();

        let list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.title)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        ratatui::widgets::StatefulWidget::render(list, area, buf, self.state);
    }
}

/// Text input field widget
pub struct TextInput<'a> {
    value: &'a str,
    label: &'a str,
    focused: bool,
    password: bool,
    cursor_position: usize,
}

impl<'a> TextInput<'a> {
    pub fn new(value: &'a str, label: &'a str) -> Self {
        Self {
            value,
            label,
            focused: false,
            password: false,
            cursor_position: value.len(),
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    pub fn cursor_position(mut self, pos: usize) -> Self {
        self.cursor_position = pos;
        self
    }
}

impl Widget for TextInput<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Cyan)
        };

        let display_value = if self.password {
            "*".repeat(self.value.len())
        } else {
            self.value.to_string()
        };

        // Add cursor indicator if focused
        let text = if self.focused {
            format!("{}|", display_value)
        } else {
            display_value
        };

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.label)
                    .border_style(border_style),
            )
            .style(Style::default().fg(Color::White));

        paragraph.render(area, buf);
    }
}

/// Checkbox widget
pub struct Checkbox<'a> {
    label: &'a str,
    checked: bool,
    focused: bool,
}

impl<'a> Checkbox<'a> {
    pub fn new(label: &'a str, checked: bool) -> Self {
        Self {
            label,
            checked,
            focused: false,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl Widget for Checkbox<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let checkbox = if self.checked { "[x]" } else { "[ ]" };
        let style = if self.focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let text = format!("{} {}", checkbox, self.label);
        let paragraph = Paragraph::new(text).style(style);
        paragraph.render(area, buf);
    }
}

/// Info box widget for displaying system information
pub struct InfoBox<'a> {
    title: &'a str,
    items: Vec<(&'a str, String)>,
}

impl<'a> InfoBox<'a> {
    pub fn new(title: &'a str, items: Vec<(&'a str, String)>) -> Self {
        Self { title, items }
    }
}

impl Widget for InfoBox<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines: Vec<Line> = self
            .items
            .iter()
            .map(|(label, value)| {
                Line::from(vec![
                    Span::styled(
                        format!("{}: ", label),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(value.clone(), Style::default().fg(Color::White)),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.title)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: true });

        paragraph.render(area, buf);
    }
}

/// Progress indicator widget
pub struct StepProgress {
    current: usize,
    total: usize,
    step_name: String,
}

impl StepProgress {
    pub fn new(current: usize, total: usize, step_name: &str) -> Self {
        Self {
            current,
            total,
            step_name: step_name.to_string(),
        }
    }
}

impl Widget for StepProgress {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let progress = self.current as f64 / self.total as f64;
        let filled = (area.width as f64 * progress) as u16;

        // Draw the progress bar background
        let progress_style = Style::default().fg(Color::Cyan);
        let empty_style = Style::default().fg(Color::DarkGray);

        for x in area.x..area.x + filled.min(area.width) {
            buf[(x, area.y)].set_symbol("=").set_style(progress_style);
        }
        for x in area.x + filled..area.x + area.width {
            buf[(x, area.y)].set_symbol("-").set_style(empty_style);
        }

        // Draw the step indicator below
        if area.height > 1 {
            let indicator = format!(
                "Step {}/{}: {}",
                self.current + 1,
                self.total,
                self.step_name
            );
            let indicator_line = Line::from(indicator);
            buf.set_line(area.x, area.y + 1, &indicator_line, area.width);
        }
    }
}

/// Help bar at the bottom of the screen
pub struct HelpBar<'a> {
    items: Vec<(&'a str, &'a str)>,
}

impl<'a> HelpBar<'a> {
    pub fn new(items: Vec<(&'a str, &'a str)>) -> Self {
        Self { items }
    }
}

impl Widget for HelpBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let spans: Vec<Span> = self
            .items
            .iter()
            .flat_map(|(key, desc)| {
                vec![
                    Span::styled(
                        format!(" {} ", key),
                        Style::default()
                            .bg(Color::DarkGray)
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!(" {} ", desc), Style::default().fg(Color::Gray)),
                ]
            })
            .collect();

        let line = Line::from(spans);
        buf.set_line(area.x, area.y, &line, area.width);
    }
}

/// Disk visualization widget
pub struct DiskDisplay {
    device: String,
    model: String,
    size: String,
    removable: bool,
    selected: bool,
}

impl DiskDisplay {
    pub fn new(device: &str, model: &str, size: &str, removable: bool, selected: bool) -> Self {
        Self {
            device: device.to_string(),
            model: model.to_string(),
            size: size.to_string(),
            removable,
            selected,
        }
    }
}

impl Widget for DiskDisplay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let marker = if self.selected { ">>" } else { "  " };
        let removable_tag = if self.removable { " [USB]" } else { "" };

        let style = if self.selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let line1 = Line::from(vec![
            Span::styled(marker, style),
            Span::styled(format!(" {}", self.device), style),
            Span::styled(removable_tag, Style::default().fg(Color::Cyan)),
        ]);

        let line2 = Line::from(vec![
            Span::raw("     "),
            Span::styled(&self.model, Style::default().fg(Color::Gray)),
            Span::raw(" - "),
            Span::styled(&self.size, Style::default().fg(Color::Cyan)),
        ]);

        if area.height >= 1 {
            buf.set_line(area.x, area.y, &line1, area.width);
        }
        if area.height >= 2 {
            buf.set_line(area.x, area.y + 1, &line2, area.width);
        }
    }
}
