use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};

use crate::app::{App, Screen};

pub mod screens;

pub fn draw(frame: &mut Frame, app: &App) {
    let [main, status] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(frame.area());

    match app.screen {
        Screen::Today => screens::today::draw(frame, main, app),
        Screen::Browse => screens::browse::draw(frame, main, app),
        Screen::Search => screens::search::draw(frame, main, app),
        Screen::Viewer => screens::viewer::draw(frame, main, app),
        Screen::Habits => screens::habits::draw(frame, main, app),
        Screen::Rules => screens::rules::draw(frame, main, app),
    }

    draw_status_bar(frame, status, app);

    if app.palette.is_some() {
        draw_palette(frame, app);
    }
    if app.show_help {
        draw_help(frame, app);
    }
}

fn draw_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let mut spans = vec![
        Span::styled(
            format!(" {} ", app.screen.title()),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(&app.profile_label, Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
    ];

    if let Some((text, is_error, _)) = &app.toast {
        spans.push(Span::styled(
            text.clone(),
            Style::default().fg(if *is_error { Color::Red } else { Color::Green }),
        ));
    } else {
        spans.push(Span::styled(
            "1-5 screens  / search  : palette  ? help  q quit",
            Style::default().fg(Color::DarkGray),
        ));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// Center a popup of the given size within `area`.
pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

fn draw_palette(frame: &mut Frame, app: &App) {
    let Some(palette) = &app.palette else { return };
    let actions = app.palette_actions();
    let height = (actions.len() as u16 + 4).min(16);
    let area = centered_rect(60, height, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Command palette ")
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [input_area, list_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).areas(inner);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Cyan)),
            Span::raw(&palette.query),
            Span::styled("▏", Style::default().fg(Color::Cyan)),
        ])),
        input_area,
    );

    let items: Vec<ListItem> = actions
        .iter()
        .map(|action| ListItem::new(action.label()))
        .collect();
    let mut state = ListState::default().with_selected(Some(palette.selected));
    frame.render_stateful_widget(
        List::new(items).highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        list_area,
        &mut state,
    );
}

fn draw_help(frame: &mut Frame, _app: &App) {
    const HELP: &[(&str, &str)] = &[
        ("Global", ""),
        ("  1-5", "switch screen (Today, Browse, Search, Habits, Rules)"),
        ("  /", "jump to search"),
        ("  : or Ctrl-P", "command palette"),
        ("  R", "refresh current screen"),
        ("  q / Esc", "back or quit · Ctrl-C force quit"),
        ("Today", ""),
        ("  [ ]", "previous / next day · t today"),
        ("  j k", "select habit · Space cycle status · Enter open"),
        ("  i", "journal entry (Enter submit, Esc cancel)"),
        ("Browse", ""),
        ("  j k", "move · l expand · h collapse · Enter open · e edit"),
        ("Search", ""),
        ("  type", "live search · Tab scope · Ctrl-R regex"),
        ("  ↑ ↓", "select · Enter open"),
        ("Item viewer", ""),
        ("  j k Ctrl-D Ctrl-U g", "scroll"),
        ("  e", "edit in $EDITOR · b backlinks · r related · c children"),
        ("  Backspace", "back to previous item"),
        ("Habits", ""),
        ("  h j k l", "move in grid · Space cycle · [ ] shift week · t today"),
        ("Rules", ""),
        ("  j k", "select · r/Enter run · e edit script"),
    ];

    let lines: Vec<Line> = HELP
        .iter()
        .map(|(key, description)| {
            if description.is_empty() {
                Line::from(Span::styled(
                    *key,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(vec![
                    Span::styled(format!("{key:<22}"), Style::default().fg(Color::Yellow)),
                    Span::raw(*description),
                ])
            }
        })
        .collect();

    let height = (lines.len() as u16 + 2).min(frame.area().height);
    let area = centered_rect(72, height, frame.area());
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Keys (any key to close) ")
                .border_style(Style::default().fg(Color::Cyan)),
        ),
        area,
    );
}
