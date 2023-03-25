use std::{error::Error, io};

use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, LeaveAlternateScreen},
};
use sysinfo::{Pid, ProcessExt, System, SystemExt};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};

enum SortByNameOptions {
    ASC,
    DESC,
    NONE,
}

struct App {
    state: TableState,
    processes: Vec<(Pid, String)>,
    sort_by_name_option: SortByNameOptions,
}

impl App {
    fn new() -> Self {
        let mut processes = vec![];
        let s = System::new_all();
        for process in s.processes() {
            processes.push((*process.0, process.1.name().to_string()));
        }
        processes.sort_by(|a, b| a.1.cmp(&b.1));

        Self {
            processes: processes,
            state: TableState::default(),
            sort_by_name_option: SortByNameOptions::NONE,
        }
    }

    pub fn switch_sort(&mut self) {
        // if sort option is NONE then set to ASC otherwise toggle ASC and DESC
        self.sort_by_name_option = match self.sort_by_name_option {
            SortByNameOptions::ASC => SortByNameOptions::DESC,
            SortByNameOptions::DESC => SortByNameOptions::ASC,
            SortByNameOptions::NONE => SortByNameOptions::ASC,
        };

        match self.sort_by_name_option {
            SortByNameOptions::ASC => {
                self.processes.sort_by(|a, b| a.1.cmp(&b.1));
            }
            SortByNameOptions::DESC => {
                self.processes.sort_by(|a, b| b.1.cmp(&a.1));
            }
            _ => {}
        };
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.processes.len() {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.state.select(Some(i))
    }

    pub fn prev(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i <= 0 {
                    self.processes.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        self.state.select(Some(i))
    }

    pub fn kill(&mut self) {
        let process = self.processes[self.state.selected().unwrap()].clone();
        let s = System::new_all();
        s.process(process.0).unwrap().kill();

        self.processes.clear();
        for process in s.processes() {
            self.processes
                .push((*process.0, process.1.name().to_string()));
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode().expect("Could not enable raw mode");
    let mut stdout = io::stdout();
    execute!(
        stdout,
        LeaveAlternateScreen,
        DisableMouseCapture,
        Clear(crossterm::terminal::ClearType::All)
    )
    .unwrap();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);
    // terminal.draw(f)

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Down => app.next(),
                KeyCode::Up => app.prev(),
                KeyCode::Enter => app.kill(),
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('n') => app.switch_sort(),
                KeyCode::Char('j') => app.next(),
                KeyCode::Char('k') => app.prev(),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();

    let main_block = Block::default()
        .borders(Borders::ALL)
        .title("Process Killer By @himanshurajora, The Vedik Dev")
        .title_alignment(tui::layout::Alignment::Right);

    let instruction_block = Block::default()
        .borders(Borders::ALL)
        .title("Instructions")
        .style(Style::default().fg(Color::Green));

    let rows = app.processes.iter().enumerate().map(|(i, f)| {
        let index = Cell::from(i.to_string());
        let pid = Cell::from(f.0.to_string());
        let name = Cell::from(f.1.to_string());

        Row::new([index, pid, name])
    });

    let selected_style = Style::default().bg(Color::Red);

    let table = Table::new(rows)
        .header(Row::new([
            Cell::from("S.N."),
            Cell::from("PID"),
            Cell::from("Name"),
        ]))
        .block(main_block)
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Length(30),
            Constraint::Min(10),
        ]);

    let chunks = Layout::default()
        .direction(tui::layout::Direction::Vertical)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)])
        .split(size);

    let instructions = vec![
        Span::from("1. Press"),
        Span::styled(" Enter ", Style::default().add_modifier(Modifier::BOLD)),
        Span::from("to kill a process, 2. Press"),
        Span::styled(" N ", Style::default().add_modifier(Modifier::BOLD)),
        Span::from("to toggle sort by Name, 3. Press"),
        Span::styled(" Q ", Style::default().add_modifier(Modifier::BOLD)),
        Span::from("to quit"),
        Span::from(", Press J and K or Up and Down array for navigating through the list"),
    ];

    let sort_name_text = Text::from(Spans::from(instructions));
    let paragraph = Paragraph::new(sort_name_text).block(instruction_block);

    f.render_stateful_widget(table, chunks[0], &mut app.state);
    // f.render_widget(instruction_block, chunks[1]);
    f.render_widget(paragraph, chunks[1]);
}
