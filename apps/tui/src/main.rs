mod app;
mod editor;
mod msg;
mod net;
mod ui;

use std::io::stdout;

use anyhow::{Context, Result};
use crossterm::event::{Event as CtEvent, EventStream, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use futures::StreamExt;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio::sync::mpsc;
use zealot_client::{Config, ZealotClient};

use crate::app::{App, EditRequest};
use crate::msg::Msg;
use crate::net::Net;

fn connect() -> Result<(ZealotClient, String, Option<String>)> {
    let config_path = Config::default_path()?;
    let config = Config::load(&config_path)?;
    let resolved = config
        .resolve(None)
        .context("no credentials — run `zealot login` first (or set ZEALOT_URL/ZEALOT_API_KEY)")?;
    let journal_item = resolved
        .profile
        .as_ref()
        .and_then(|name| config.profiles.get(name))
        .and_then(|profile| profile.journal_item.clone());
    let label = match &resolved.profile {
        Some(profile) => format!("{profile} · {}", resolved.server_url),
        None => resolved.server_url.clone(),
    };
    Ok((
        ZealotClient::new(resolved.server_url.clone(), resolved.api_key.clone()),
        label,
        journal_item,
    ))
}

fn enter_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    Ok(terminal)
}

fn leave_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(stdout(), LeaveAlternateScreen);
}

#[tokio::main]
async fn main() -> Result<()> {
    let (client, label, journal_item) = connect()?;

    // The TUI parses the renderer's ANSI back into spans; OSC 8 links would
    // confuse that parser.
    zealot_zscript::set_color_enabled(true);
    zealot_zscript::style::set_hyperlinks_enabled(false);

    // Restore the terminal on panic so the shell isn't left in raw mode.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        leave_terminal();
        default_hook(info);
    }));

    let (tx, mut rx) = mpsc::unbounded_channel::<Msg>();
    let net = Net {
        client,
        tx: tx.clone(),
    };
    let mut app = App::new(net, label, journal_item);

    let mut terminal = enter_terminal()?;
    let mut input = EventStream::new();
    let mut tick = tokio::time::interval(std::time::Duration::from_millis(150));

    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        tokio::select! {
            maybe_input = input.next() => {
                match maybe_input {
                    Some(Ok(CtEvent::Key(key))) if key.kind != KeyEventKind::Release => {
                        app.on_key(key);
                    }
                    Some(Ok(CtEvent::Resize(_, _))) => {
                        let _ = terminal.clear();
                    }
                    Some(Ok(_)) => {}
                    Some(Err(e)) => {
                        leave_terminal();
                        return Err(e.into());
                    }
                    None => break,
                }
            }
            Some(msg) = rx.recv() => app.on_msg(msg),
            _ = tick.tick() => app.on_tick(),
        }

        // $EDITOR round-trips suspend the TUI.
        if let Some(request) = app.pending_edit.take() {
            terminal.clear()?;
            leave_terminal();
            let outcome = match &request {
                EditRequest::ItemContent { content, .. } => editor::edit_text(content, "md"),
                EditRequest::RuleScript { script, .. } => editor::edit_text(script, "lua"),
            };
            terminal = enter_terminal()?;
            terminal.clear()?;
            match outcome {
                Ok(Some(edited)) => match request {
                    EditRequest::ItemContent { item_id, .. } => {
                        app.net.save_item_content(item_id, edited);
                    }
                    EditRequest::RuleScript { rule_id, .. } => {
                        app.net.save_rule_script(rule_id, edited);
                    }
                },
                Ok(None) => app.toast("no changes", false),
                Err(e) => {
                    let _ = tx.send(Msg::Done(Err(e.to_string())));
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    leave_terminal();
    Ok(())
}
