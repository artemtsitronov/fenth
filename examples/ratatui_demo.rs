use fenth::Theme;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph},
};

struct App {
    themes: Vec<Theme>,
    state: ListState,
    quit: bool,
}

impl App {
    fn new(themes: Vec<Theme>) -> Self {
        Self {
            themes,
            state: ListState::default().with_selected(Some(0)),
            quit: false,
        }
    }

    fn selected(&self) -> usize {
        self.state.selected().unwrap_or(0)
    }

    fn step(&mut self, delta: isize) {
        let len = self.themes.len() as isize;
        let next = (self.selected() as isize + delta).rem_euclid(len);
        self.state.select(Some(next as usize));
    }

    fn on_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
            KeyCode::Down | KeyCode::Right | KeyCode::Char('j' | 'l') | KeyCode::Tab => {
                self.step(1)
            }
            KeyCode::Up | KeyCode::Left | KeyCode::Char('k' | 'h') | KeyCode::BackTab => {
                self.step(-1)
            }
            KeyCode::Home | KeyCode::Char('g') => self.state.select(Some(0)),
            KeyCode::End | KeyCode::Char('G') => self.state.select(Some(self.themes.len() - 1)),
            _ => {}
        }
    }
}

fn rgb(theme: &Theme, role: &str) -> Color {
    let color = theme[role];
    Color::Rgb(color.r, color.g, color.b)
}

fn fg(theme: &Theme, role: &str) -> Style {
    Style::new().fg(rgb(theme, role))
}

fn on(theme: &Theme, fg_role: &str, bg_role: &str) -> Style {
    Style::new().fg(rgb(theme, fg_role)).bg(rgb(theme, bg_role))
}

fn panel(theme: &Theme, title: &str, border_role: &str, bg_role: &str) -> Block<'static> {
    Block::bordered()
        .border_style(fg(theme, border_role))
        .title(Span::styled(
            format!(" {title} "),
            fg(theme, "accent.secondary"),
        ))
        .style(on(theme, "foreground", bg_role))
}

fn draw(frame: &mut Frame, app: &mut App) {
    let theme = app.themes[app.selected()].clone();
    let theme = &theme;
    let count = app.themes.len();
    let position = app.selected() + 1;
    let area = frame.area();

    frame.render_widget(
        Block::new().style(on(theme, "foreground", "background")),
        area,
    );

    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(area);
    let [sidebar, main] =
        Layout::horizontal([Constraint::Length(24), Constraint::Min(0)]).areas(body);
    let [list_area, palette_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(6)]).areas(sidebar);
    let [text_area, code_area, status_area] = Layout::vertical([
        Constraint::Length(5),
        Constraint::Min(9),
        Constraint::Length(6),
    ])
    .areas(main);

    let title = Line::from(vec![
        Span::styled(
            " fenth ",
            on(theme, "foreground.inverse", "accent.primary").add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            theme.name().to_string(),
            fg(theme, "foreground").add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {:?}  {position}/{count}", theme.variant()).to_lowercase(),
            fg(theme, "foreground.muted"),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(title).block(
            Block::bordered()
                .border_style(fg(theme, "border"))
                .style(on(theme, "foreground", "surface")),
        ),
        header,
    );

    let items: Vec<ListItem> = app
        .themes
        .iter()
        .map(|t| ListItem::new(t.name().to_string()))
        .collect();
    let list = List::new(items)
        .block(panel(theme, "Themes", "border.focus", "surface"))
        .style(on(theme, "foreground", "surface"))
        .highlight_style(on(theme, "foreground", "selection").add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(list, list_area, &mut app.state);

    let hues = [
        "red", "orange", "yellow", "green", "cyan", "blue", "purple", "pink",
    ];
    let swatches: Vec<Line> = hues
        .chunks(2)
        .map(|pair| {
            Line::from(
                pair.iter()
                    .flat_map(|role| {
                        [
                            Span::styled("██ ", fg(theme, role)),
                            Span::styled(format!("{role:<7}"), fg(theme, "foreground.muted")),
                        ]
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    frame.render_widget(
        Paragraph::new(swatches).block(panel(theme, "Palette", "border", "surface")),
        palette_area,
    );

    let button = |role: &str| {
        Span::styled(
            format!(" {role} "),
            on(theme, "foreground.inverse", role).add_modifier(Modifier::BOLD),
        )
    };
    let text = vec![
        Line::styled(
            "The quick brown fox jumps over the lazy dog",
            fg(theme, "foreground"),
        ),
        Line::from(vec![
            Span::styled("muted ", fg(theme, "foreground.muted")),
            Span::styled("subtle ", fg(theme, "foreground.subtle")),
            Span::styled("disabled ", fg(theme, "foreground.disabled")),
            Span::styled("link", fg(theme, "link").add_modifier(Modifier::UNDERLINED)),
        ]),
        Line::from(vec![
            button("accent.primary"),
            Span::raw(" "),
            button("accent.secondary"),
            Span::raw(" "),
            button("accent.tertiary"),
        ]),
    ];
    frame.render_widget(
        Paragraph::new(text).block(panel(theme, "Text", "border", "surface")),
        text_area,
    );

    let t = |role: &'static str, text: &'static str| Span::styled(text, fg(theme, role));
    let rows: Vec<Vec<Span>> = vec![
        vec![
            t("syntax.keyword", "use "),
            t("syntax.variable", "fenth"),
            t("syntax.punctuation", "::"),
            t("syntax.type", "Theme"),
            t("syntax.punctuation", ";"),
        ],
        vec![t("syntax.comment", "// switch themes with the arrow keys")],
        vec![
            t("syntax.keyword", "fn "),
            t("syntax.function", "main"),
            t("syntax.punctuation", "() {"),
        ],
        vec![
            t("syntax.keyword", "    let "),
            t("syntax.variable", "theme "),
            t("syntax.punctuation", "= "),
            t("syntax.variable", "fenth"),
            t("syntax.punctuation", "::"),
            t("syntax.function", "get"),
            t("syntax.punctuation", "("),
            t("syntax.string", "\"nord\""),
            t("syntax.punctuation", ").unwrap();"),
        ],
        vec![
            t("syntax.keyword", "    let "),
            t("syntax.variable", "size "),
            t("syntax.punctuation", "= "),
            t("syntax.number", "42"),
            t("syntax.punctuation", ";"),
        ],
        vec![
            t("syntax.macro", "    println!"),
            t("syntax.punctuation", "("),
            t("syntax.string", "\"{}\""),
            t("syntax.punctuation", ", "),
            t("syntax.variable", "theme"),
            t("syntax.punctuation", "["),
            t("syntax.string", "\"accent.primary\""),
            t("syntax.punctuation", "]);"),
        ],
        vec![t("syntax.punctuation", "}")],
    ];
    let active = 3;
    let code: Vec<Line> = rows
        .into_iter()
        .enumerate()
        .map(|(i, spans)| {
            let gutter = if i == active {
                "line.number.active"
            } else {
                "line.number"
            };
            let mut all = vec![Span::styled(format!("{:>3}  ", i + 1), fg(theme, gutter))];
            all.extend(spans);
            let line = Line::from(all);
            if i == active {
                line.style(Style::new().bg(rgb(theme, "line.highlight")))
            } else {
                line
            }
        })
        .collect();
    frame.render_widget(
        Paragraph::new(code).block(panel(theme, "Code", "border", "background.alt")),
        code_area,
    );

    let width = status_area.width.saturating_sub(2) as usize;
    let status = |label: &str, role: &str, message: &str| {
        Line::styled(
            format!(" {label:<9}{message:<w$}", w = width.saturating_sub(10)),
            on(theme, role, &format!("{role}.background")),
        )
    };
    let messages = vec![
        status("error", "error", "Could not read config.toml"),
        status("warning", "warning", "The cache is out of date"),
        status("success", "success", "Saved 3 files"),
        status("info", "info", &format!("{count} themes available")),
    ];
    frame.render_widget(
        Paragraph::new(messages).block(panel(theme, "Status", "border", "surface")),
        status_area,
    );

    let key =
        |text: &'static str| Span::styled(text, on(theme, "foreground.inverse", "accent.primary"));
    let label = |text: &'static str| Span::styled(text, fg(theme, "foreground.muted"));
    let help = Line::from(vec![
        key(" ↑/↓ j/k "),
        label(" switch theme  "),
        key(" g/G "),
        label(" first/last  "),
        key(" q "),
        label(" quit"),
    ]);
    frame.render_widget(
        Paragraph::new(help).style(on(theme, "foreground.muted", "background.alt")),
        footer,
    );
}

fn run(terminal: &mut DefaultTerminal, mut app: App) -> std::io::Result<()> {
    while !app.quit {
        terminal.draw(|frame| draw(frame, &mut app))?;
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.on_key(key.code);
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut themes = fenth::all().to_vec();
    for path in std::env::args().skip(1) {
        match Theme::from_file(&path) {
            Ok(theme) => themes.push(theme),
            Err(error) => {
                eprintln!("{path}: {error}");
                std::process::exit(1);
            }
        }
    }
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, App::new(themes));
    ratatui::restore();
    result
}
