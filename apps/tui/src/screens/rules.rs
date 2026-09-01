//! Rules: list of automation rules with a detail pane; run or edit a rule.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use zealot_domain::rule::RuleDto;

use super::{Cx, EditRequest, Pane, Screen, Select};
use crate::msg::Load;
use crate::net::Net;

#[derive(Default)]
pub struct RulesState {
    pub rules: Load<Vec<RuleDto>>,
    pub sel: Select,
    pub started: bool,
    pub generation: u64,
}

impl RulesState {
    fn selected(&self) -> Option<&RuleDto> {
        let rules = self.rules.loaded()?;
        rules.get(self.sel.resolved(rules.len())?)
    }
}

impl Pane for RulesState {
    fn load(&mut self, net: &Net, gen_id: u64) {
        self.generation = gen_id;
        self.started = true;
        self.rules = Load::Loading;
        net.load_rules(gen_id);
    }

    fn on_key(&mut self, key: KeyEvent, cx: &mut Cx) {
        let len = self.rules.loaded().map(Vec::len).unwrap_or(0);
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => cx.back(),
            KeyCode::Char('j') | KeyCode::Down => self.sel.down(len),
            KeyCode::Char('k') | KeyCode::Up => self.sel.up(),
            KeyCode::Char('r') | KeyCode::Enter => {
                if let Some(rule) = self.selected() {
                    let (id, name) = (rule.rule_id, rule.name.clone());
                    cx.toast(format!("running '{name}'…"), false);
                    cx.net.run_rule(id, name);
                }
            }
            KeyCode::Char('e') => {
                if let Some(rule) = self.selected() {
                    cx.edit(EditRequest::RuleScript {
                        rule_id: rule.rule_id,
                        script: rule.script.clone(),
                    });
                }
            }
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let [list_area, detail_area] =
            Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
                .areas(area);

        let list_block = Block::default()
            .borders(Borders::ALL)
            .title(Screen::Rules.titled("Rules (r run · e edit script)"))
            .border_style(Style::default().fg(Color::Cyan));

        let rules = match &self.rules {
            Load::Loaded(rules) => rules,
            Load::Error(e) => {
                frame.render_widget(
                    Paragraph::new(format!("Error: {e}"))
                        .style(Style::default().fg(Color::Red))
                        .block(list_block),
                    area,
                );
                return;
            }
            _ => {
                frame.render_widget(
                    Paragraph::new("Loading…")
                        .style(Style::default().fg(Color::DarkGray))
                        .block(list_block),
                    area,
                );
                return;
            }
        };

        let items: Vec<ListItem> = rules
            .iter()
            .map(|rule| {
                let enabled = if rule.enabled {
                    Span::styled("●", Style::default().fg(Color::Green))
                } else {
                    Span::styled("○", Style::default().fg(Color::DarkGray))
                };
                let error = if rule.last_error.as_deref().is_some_and(|e| !e.is_empty()) {
                    Span::styled(" ✘", Style::default().fg(Color::Red))
                } else {
                    Span::raw("")
                };
                ListItem::new(Line::from(vec![
                    enabled,
                    Span::raw(" "),
                    Span::styled(
                        rule.name.clone(),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("  {}", rule.trigger.kind_str()),
                        Style::default().fg(Color::Yellow),
                    ),
                    error,
                ]))
            })
            .collect();
        let mut state = ListState::default().with_selected(self.sel.resolved(rules.len()));
        frame.render_stateful_widget(
            List::new(items)
                .block(list_block)
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
            list_area,
            &mut state,
        );

        let detail_block = Block::default()
            .borders(Borders::ALL)
            .title(" Detail ")
            .border_style(Style::default().fg(Color::DarkGray));
        let Some(rule) = self.selected() else {
            frame.render_widget(Paragraph::new("No rules.").block(detail_block), detail_area);
            return;
        };

        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    rule.name.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  #{}", rule.rule_id),
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            Line::from(Span::raw(rule.description.clone())),
            Line::from(vec![
                Span::styled("Trigger: ", Style::default().fg(Color::Cyan)),
                Span::raw(serde_json::to_string(&rule.trigger).unwrap_or_default()),
            ]),
            Line::from(vec![
                Span::styled("Last run: ", Style::default().fg(Color::Cyan)),
                Span::raw(
                    rule.last_run_at
                        .map(|at| at.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "never".into()),
                ),
            ]),
            Line::default(),
        ];
        if let Some(output) = rule.last_output.as_deref().filter(|o| !o.is_empty()) {
            lines.push(Line::from(Span::styled(
                "Last output:",
                Style::default().fg(Color::Green),
            )));
            for line in output.lines().take(20) {
                lines.push(Line::from(Span::raw(line.to_string())));
            }
        }
        if let Some(error) = rule.last_error.as_deref().filter(|e| !e.is_empty()) {
            lines.push(Line::from(Span::styled(
                "Last error:",
                Style::default().fg(Color::Red),
            )));
            for line in error.lines().take(20) {
                lines.push(Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Red),
                )));
            }
        }
        frame.render_widget(
            Paragraph::new(lines)
                .block(detail_block)
                .wrap(Wrap { trim: false }),
            detail_area,
        );
    }
}
