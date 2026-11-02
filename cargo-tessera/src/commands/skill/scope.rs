use std::io;

use anyhow::{Result, bail};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    crossterm::{
        cursor::Show,
        event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InstallScope {
    Local,
    Global,
}

pub(super) fn resolve(
    global: bool,
    local: bool,
    interactive: bool,
    prompt: impl FnOnce() -> Result<Option<InstallScope>>,
) -> Result<Option<InstallScope>> {
    match (global, local) {
        (true, true) => bail!("--global and --local cannot be used together"),
        (true, false) => Ok(Some(InstallScope::Global)),
        (false, true) => Ok(Some(InstallScope::Local)),
        (false, false) if interactive => prompt(),
        (false, false) => {
            bail!("Scope selection requires an interactive terminal. Specify --local or --global.")
        }
    }
}

struct Cleanup<F: FnOnce()>(Option<F>);

impl<F: FnOnce()> Drop for Cleanup<F> {
    fn drop(&mut self) {
        if let Some(cleanup) = self.0.take() {
            cleanup();
        }
    }
}

fn with_cleanup<T>(run: impl FnOnce() -> Result<T>, cleanup: impl FnOnce()) -> Result<T> {
    let _cleanup = Cleanup(Some(cleanup));
    run()
}

pub(super) fn prompt() -> Result<Option<InstallScope>> {
    with_cleanup(
        || {
            enable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen)?;
            let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
            run_prompt(&mut terminal, || Ok(event::read()?))
        },
        || {
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
        },
    )
}

fn run_prompt<B: Backend>(
    terminal: &mut Terminal<B>,
    mut read_event: impl FnMut() -> Result<Event>,
) -> Result<Option<InstallScope>> {
    let mut selection = ListState::default().with_selected(Some(0));
    loop {
        terminal.draw(|frame| {
            let [choices, help] = Layout::vertical([Constraint::Min(4), Constraint::Length(4)])
                .areas(frame.area());
            let list = List::new([
                ListItem::new("Local  — <project>/.agents/skills/tessera-good-taste"),
                ListItem::new("Global — ~/.agents/skills/tessera-good-taste"),
            ])
            .block(
                Block::default()
                    .title(" Install Tessera skill ")
                    .borders(Borders::ALL),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
            frame.render_stateful_widget(list, choices, &mut selection);
            frame.render_widget(
                Paragraph::new(
                    "Up/Down or Tab: select • Enter: install • Esc/q/Ctrl-C: cancel\nLocal is the default. --path applies only to local installation.",
                )
                .wrap(Wrap { trim: false }),
                help,
            );
        })?;
        let Event::Key(key) = read_event()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => return Ok(None),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(None),
            KeyCode::Up | KeyCode::Down | KeyCode::Tab | KeyCode::BackTab => {
                selection.select(Some(1 - selection.selected().unwrap_or(0)));
            }
            KeyCode::Enter => {
                return Ok(Some(if selection.selected() == Some(1) {
                    InstallScope::Global
                } else {
                    InstallScope::Local
                }));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use ratatui::{backend::TestBackend, crossterm::event::KeyEvent};

    use super::*;

    #[test]
    fn explicit_scopes_bypass_prompt_even_without_a_terminal() {
        for (global, local, expected) in [
            (true, false, InstallScope::Global),
            (false, true, InstallScope::Local),
        ] {
            assert_eq!(
                resolve(global, local, false, || panic!(
                    "explicit flags must not start the TUI"
                ))
                .unwrap(),
                Some(expected)
            );
        }
    }

    #[test]
    fn missing_scope_without_a_terminal_and_conflicting_flags_never_prompt() {
        let error = resolve(false, false, false, || {
            panic!("must not prompt without TTY")
        })
        .unwrap_err();
        assert!(error.to_string().contains("Specify --local or --global"));
        assert!(resolve(true, true, true, || panic!("conflict must not prompt")).is_err());
    }

    fn choose(keys: &[KeyEvent]) -> Option<InstallScope> {
        let mut terminal = Terminal::new(TestBackend::new(90, 10)).unwrap();
        let mut events = keys.iter();
        run_prompt(&mut terminal, || {
            Ok(Event::Key(*events.next().expect("prompt must finish")))
        })
        .unwrap()
    }

    #[test]
    fn tui_defaults_to_local_and_navigation_can_select_global() {
        assert_eq!(
            choose(&[KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)]),
            Some(InstallScope::Local)
        );
        assert_eq!(
            choose(&[
                KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            ]),
            Some(InstallScope::Global)
        );
    }

    #[test]
    fn tui_cancel_keys_return_no_installation_scope() {
        for key in [
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        ] {
            assert_eq!(choose(&[key]), None);
        }
    }

    #[test]
    fn terminal_cleanup_runs_on_cancellation_setup_error_and_unwind() {
        let restored = Cell::new(false);
        let result: Result<Option<InstallScope>> = with_cleanup(|| Ok(None), || restored.set(true));
        assert_eq!(result.unwrap(), None);
        assert!(restored.replace(false));

        let result: Result<()> = with_cleanup(|| bail!("setup failed"), || restored.set(true));
        assert!(result.is_err());
        assert!(restored.replace(false));

        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _: Result<()> = with_cleanup(|| panic!("draw failed"), || restored.set(true));
        }));
        assert!(panic.is_err());
        assert!(restored.get());
    }
}
