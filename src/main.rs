use std::{error::Error, io};

use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, LeaveAlternateScreen},
};
use sysinfo::{Pid, ProcessExt, System, SystemExt};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::Constraint,
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};

struct App {
    state: TableState,
    processes: Vec<(Pid, String)>,
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
        }
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
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Down => app.next(),
                KeyCode::Up => app.prev(),
                KeyCode::Enter => app.kill(),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();

    let rows = app.processes.iter().enumerate().map(|(i, f)| {
        let index = Cell::from(i.to_string());
        let pid = Cell::from(f.0.to_string());
        let name = Cell::from(f.1.to_string());

        Row::new([index, pid, name])
    });

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);

    let table = Table::new(rows)
        .header(Row::new([
            Cell::from("S.N."),
            Cell::from("PID"),
            Cell::from("Name"),
        ]))
        .block(Block::default().borders(Borders::ALL).title("Table"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Length(30),
            Constraint::Min(10),
        ]);

    f.render_stateful_widget(table, size, &mut app.state);
    // f.render_widget(block, size);
}
