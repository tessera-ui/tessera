use std::io;

use anyhow::{Result, bail};
use ratatui::{
    Terminal, TerminalOptions, Viewport,
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    crossterm::{
        cursor::Show,
        event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode},
    },
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};
use unicode_width::UnicodeWidthChar;

/// Rows reserved below the shell prompt for the full selector layout.
const SELECTOR_HEIGHT: u16 = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InstallScope {
    Local,
    Global,
}

/// Destinations previewed by the interactive selector.
pub(super) struct Choices {
    pub local: String,
    pub global: String,
    /// Why the global scope cannot be chosen, if it is unavailable.
    pub global_unavailable: Option<&'static str>,
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
        (false, false) => bail!(
            "No installation scope was given and the interactive selector needs a terminal. \
             Run `cargo tessera skill --local` or `cargo tessera skill --global`."
        ),
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

/// Shows the selector inline below the existing shell output.
///
/// The selector rows are erased afterwards, so earlier output stays visible.
pub(super) fn prompt(choices: &Choices) -> Result<Option<InstallScope>> {
    with_cleanup(
        || {
            enable_raw_mode()?;
            let mut terminal = Terminal::with_options(
                CrosstermBackend::new(io::stdout()),
                TerminalOptions {
                    viewport: Viewport::Inline(SELECTOR_HEIGHT),
                },
            )?;
            let selection = run_prompt(&mut terminal, choices, || Ok(event::read()?));
            let cleared = terminal.clear();
            let selection = selection?;
            cleared?;
            Ok(selection)
        },
        || {
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), Show);
        },
    )
}

fn run_prompt<B: Backend>(
    terminal: &mut Terminal<B>,
    choices: &Choices,
    mut read_event: impl FnMut() -> Result<Event>,
) -> Result<Option<InstallScope>>
where
    B::Error: Send + Sync + 'static,
{
    let mut selected = InstallScope::Local;
    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            render(area, frame.buffer_mut(), choices, selected);
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
            KeyCode::Up
            | KeyCode::Down
            | KeyCode::Tab
            | KeyCode::BackTab
            | KeyCode::Char('j' | 'k')
                if choices.global_unavailable.is_none() =>
            {
                selected = match selected {
                    InstallScope::Local => InstallScope::Global,
                    InstallScope::Global => InstallScope::Local,
                };
            }
            KeyCode::Enter => return Ok(Some(selected)),
            _ => {}
        }
    }
}

fn render(area: Rect, buffer: &mut Buffer, choices: &Choices, selected: InstallScope) {
    Paragraph::new(lines(area.width, area.height, choices, selected)).render(area, buffer);
}

fn lines(width: u16, height: u16, choices: &Choices, selected: InstallScope) -> Vec<Line<'static>> {
    let width = usize::from(width);
    let compact = height < SELECTOR_HEIGHT;
    let accent = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let muted = Style::default().fg(Color::DarkGray);
    let bold = Style::default().add_modifier(Modifier::BOLD);

    let brand = "◆ tessera";
    let title = ["Install the tessera-good-taste skill", "Install skill"]
        .into_iter()
        .find(|title| brand.chars().count() + 2 + title.len() <= width);
    let mut header = vec![Span::styled(brand, accent)];
    if let Some(title) = title {
        header.push(Span::raw("  "));
        header.push(Span::styled(title, bold));
    }
    let mut lines = vec![Line::from(header)];
    if !compact {
        lines.push(Line::styled(
            if width >= 48 {
                "  Choose where coding agents should find it."
            } else {
                "  Choose a scope."
            },
            muted,
        ));
        lines.push(Line::default());
    }

    let global_description = choices
        .global_unavailable
        .unwrap_or("all projects for this user");
    for (scope, name, description, path, enabled) in [
        (
            InstallScope::Local,
            "Local",
            "this project only",
            &choices.local,
            true,
        ),
        (
            InstallScope::Global,
            "Global",
            global_description,
            &choices.global,
            choices.global_unavailable.is_none(),
        ),
    ] {
        let active = scope == selected;
        let (marker, label) = if active {
            ("› ", accent)
        } else if enabled {
            ("  ", bold)
        } else {
            ("  ", muted)
        };
        let mut option = vec![
            Span::styled(format!("  {marker}"), accent),
            Span::styled(format!("{name:<8}"), label),
        ];
        if width >= 40 {
            option.push(Span::styled(description.to_owned(), muted));
        }
        lines.push(Line::from(option));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(
                truncate_start(path, width.saturating_sub(4)),
                if active { Style::default() } else { muted },
            ),
        ]));
    }

    if !compact {
        lines.push(Line::default());
    }
    let hints: &[(&str, &str)] = if width >= 44 {
        &[
            ("↑↓", " select   "),
            ("enter", " install   "),
            ("esc", " cancel"),
        ]
    } else {
        &[("↑↓", " "), ("enter", " "), ("esc", "")]
    };
    let mut hint = vec![Span::raw("  ")];
    for (key, action) in hints {
        hint.push(Span::styled(*key, bold));
        hint.push(Span::styled(*action, muted));
    }
    lines.push(Line::from(hint));
    lines
}

/// Keeps the end of a path, which identifies the destination best.
fn truncate_start(text: &str, width: usize) -> String {
    let total: usize = text.chars().filter_map(UnicodeWidthChar::width).sum();
    if total <= width {
        return text.to_owned();
    }
    if width == 0 {
        return String::new();
    }
    let mut kept = Vec::new();
    let mut used = 1;
    for character in text.chars().rev() {
        let character_width = character.width().unwrap_or(0);
        if used + character_width > width {
            break;
        }
        used += character_width;
        kept.push(character);
    }
    std::iter::once('…').chain(kept.into_iter().rev()).collect()
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use ratatui::{backend::TestBackend, crossterm::event::KeyEvent};

    use super::*;

    fn choices(global_unavailable: Option<&'static str>) -> Choices {
        Choices {
            local: r"C:\work\demo-app\.agents\skills\tessera-good-taste".into(),
            global: r"C:\Users\ada\.agents\skills\tessera-good-taste".into(),
            global_unavailable,
        }
    }

    fn text(terminal: &Terminal<TestBackend>) -> String {
        let buffer = terminal.backend().buffer();
        (0..buffer.area.height)
            .map(|y| {
                let mut row = String::new();
                for x in 0..buffer.area.width {
                    row.push_str(buffer[(x, y)].symbol());
                }
                row.trim_end().to_owned()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn screen(width: u16, height: u16, choices: &Choices, selected: InstallScope) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render(area, frame.buffer_mut(), choices, selected);
            })
            .unwrap();
        let screen = text(&terminal);
        if std::env::var_os("TESSERA_SKILL_SCREENS").is_some() {
            eprintln!("--- {width}x{height} {selected:?} ---\n{screen}");
        }
        screen
    }

    #[test]
    fn explicit_scopes_bypass_prompt_even_without_a_terminal() {
        for (global, local, expected) in [
            (true, false, InstallScope::Global),
            (false, true, InstallScope::Local),
        ] {
            assert_eq!(
                resolve(global, local, false, || panic!(
                    "explicit flags must not start the selector"
                ))
                .unwrap(),
                Some(expected)
            );
        }
    }

    #[test]
    fn missing_scope_without_a_terminal_and_conflicting_flags_never_prompt() {
        let error = resolve(false, false, false, || {
            panic!("must not prompt without a terminal")
        })
        .unwrap_err();
        assert!(error.to_string().contains("cargo tessera skill --local"));
        assert!(resolve(true, true, true, || panic!("conflict must not prompt")).is_err());
    }

    #[test]
    fn selector_renders_compact_borderless_choices_with_destinations() {
        let expected = [
            "◆ tessera  Install the tessera-good-taste skill",
            "  Choose where coding agents should find it.",
            "",
            "  › Local   this project only",
            r"    C:\work\demo-app\.agents\skills\tessera-good-taste",
            "    Global  all projects for this user",
            r"    C:\Users\ada\.agents\skills\tessera-good-taste",
            "",
            "  ↑↓ select   enter install   esc cancel",
        ]
        .join("\n");
        let local = screen(72, 9, &choices(None), InstallScope::Local);
        assert_eq!(local, expected);
        assert!(!local.chars().any(|c| matches!(c, '│' | '─' | '┌' | '┐')));

        let global = screen(72, 9, &choices(None), InstallScope::Global);
        assert!(global.contains("\n    Local   this project only\n"));
        assert!(global.contains("\n  › Global  all projects for this user\n"));

        let mut terminal = Terminal::new(TestBackend::new(72, 9)).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render(
                    area,
                    frame.buffer_mut(),
                    &choices(None),
                    InstallScope::Local,
                );
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(2, 3)].fg, Color::Cyan);
        assert_eq!(buffer[(4, 3)].fg, Color::Cyan);
        assert_eq!(buffer[(4, 5)].fg, Color::Reset);
    }

    #[test]
    fn selector_degrades_for_narrow_short_and_tiny_terminals() {
        let narrow = screen(32, 9, &choices(None), InstallScope::Local);
        let lines: Vec<_> = narrow.lines().collect();
        assert_eq!(lines[0], "◆ tessera  Install skill");
        assert_eq!(lines[3], "  › Local");
        assert!(lines[4].starts_with("    …") && lines[4].ends_with(r"\tessera-good-taste"));
        assert_eq!(lines[8], "  ↑↓ enter esc");
        assert!(lines.iter().all(|line| line.chars().count() <= 32));

        let short = screen(60, 6, &choices(None), InstallScope::Global);
        let lines: Vec<_> = short.lines().collect();
        assert_eq!(lines.len(), 6);
        assert!(lines[3].starts_with("  › Global"));
        assert!(lines[5].contains("enter install"));

        for (width, height) in [(1, 1), (5, 2), (12, 4), (200, 3)] {
            let tiny = screen(width, height, &choices(None), InstallScope::Local);
            let header = tiny.lines().next().unwrap_or_default();
            assert!(
                [
                    "◆ tessera",
                    "◆ tessera  Install skill",
                    "◆ tessera  Install the tessera-good-taste skill",
                ]
                .contains(&header)
                    || "◆ tessera".starts_with(header),
                "titles must not be cut mid-word: {header:?}"
            );
        }
        assert!(screen(12, 4, &choices(None), InstallScope::Local).starts_with("◆ tessera\n"));
    }

    #[test]
    fn redraw_after_resize_uses_the_new_size() {
        let mut terminal = Terminal::new(TestBackend::new(72, 9)).unwrap();
        let draw = |terminal: &mut Terminal<TestBackend>| {
            terminal
                .draw(|frame| {
                    let area = frame.area();
                    render(
                        area,
                        frame.buffer_mut(),
                        &choices(None),
                        InstallScope::Local,
                    );
                })
                .unwrap();
        };
        draw(&mut terminal);
        terminal.backend_mut().resize(30, 6);
        draw(&mut terminal);
        let resized = text(&terminal);
        assert_eq!(resized.lines().next(), Some("◆ tessera  Install skill"));
        assert!(resized.lines().all(|line| line.chars().count() <= 30));
    }

    fn choose(choices: &Choices, keys: &[KeyEvent]) -> Option<InstallScope> {
        let mut terminal = Terminal::new(TestBackend::new(72, 9)).unwrap();
        let mut events = keys.iter();
        run_prompt(&mut terminal, choices, || {
            Ok(Event::Key(*events.next().expect("prompt must finish")))
        })
        .unwrap()
    }

    #[test]
    fn path_override_keeps_global_visible_but_unselectable() {
        let blocked = choices(Some("unavailable with --path"));
        assert!(
            screen(72, 9, &blocked, InstallScope::Local)
                .contains("\n    Global  unavailable with --path\n")
        );
        assert_eq!(
            choose(
                &blocked,
                &[
                    KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
                    KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                ]
            ),
            Some(InstallScope::Local)
        );
    }

    #[test]
    fn selector_defaults_to_local_and_navigation_can_select_global() {
        let choices = choices(None);
        assert_eq!(
            choose(
                &choices,
                &[KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)]
            ),
            Some(InstallScope::Local)
        );
        assert_eq!(
            choose(
                &choices,
                &[
                    KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
                    KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                ]
            ),
            Some(InstallScope::Global)
        );
    }

    #[test]
    fn cancel_keys_return_no_installation_scope() {
        let choices = choices(None);
        for key in [
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        ] {
            assert_eq!(choose(&choices, &[key]), None);
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
