mod widgets;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use gpm::import_sgcp;
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
};
use std::{
    error::Error,
    io::{self},
};
use widgets::nested_list::TreeNode;

import_sgcp!();

// NOTE: Only for C-like enums
macro_rules! iterate_enum {
    ($enum:path) => {{
        std::iter::successors(Some(1), |&i| Some(i + 1))
            .map_while(|i| <$enum as std::convert::TryFrom<i32>>::try_from(i).ok())
            .collect::<Vec<$enum>>()
    }};
}

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> Self {
        let state = if items.is_empty() {
            ListState::default()
        } else {
            ListState::default().with_selected(Some(0))
        };

        Self { state, items }
    }

    fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            },
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            },
            None => 0,
        };
        self.state.select(Some(i));
    }
}

struct App {
    should_quit: bool,
    command_options: StatefulList<&'static str>,
    nested_command_options: Vec<TreeNode>,
}

impl App {
    fn new() -> Self {
        Self {
            should_quit: false,
            command_options: StatefulList::with_items(
                iterate_enum!(sgcp::Resource)
                    .iter()
                    .map(|resource| resource.as_str_name())
                    .collect(),
            ),
            nested_command_options: iterate_enum!(sgcp::Resource)
                .iter()
                .map(|resource| TreeNode {
                    name: resource.as_str_name().to_owned(),
                    children: get_tasks_for_resource(resource)
                        .iter()
                        .map(|task_name| TreeNode {
                            name: task_name.to_owned(),
                            children: Vec::new(),
                            is_expanded: false,
                        })
                        .collect(),
                    is_expanded: true,
                })
                .collect(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                app.should_quit = true;
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(frame: &mut Frame, app: &mut App) {
    let main_chunks = Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(frame.area());

    let left_column = main_chunks[0];
    let right_column_chunks =
        Layout::vertical([Constraint::Percentage(20), Constraint::Min(0)]).split(main_chunks[1]);

    let right_panel_top = Block::default()
        .title("Request")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    frame.render_widget(right_panel_top, right_column_chunks[0]);

    let right_panel_bottom = Block::default()
        .title("Response")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    frame.render_widget(right_panel_bottom, right_column_chunks[1]);

    let list_items: Vec<ListItem> = app
        .nested_command_options
        .iter_mut()
        .flat_map(|tree| tree.flatten_tree())
        .map(|list| ListItem::new(list))
        .collect();

    frame.render_stateful_widget(
        List::new(list_items).block(
            Block::default()
                .title("GRASP Resources")
                .borders(Borders::all())
                .border_type(BorderType::Rounded),
        ),
        left_column,
        &mut app.command_options.state,
    )
}

fn get_tasks_for_resource(resource: &sgcp::Resource) -> Vec<String> {
    match resource {
        Resource::UndefinedComponent => {
            panic!("Should not display the undefined components on the UI")
        },

        Resource::Bms => iterate_enum!(sgcp::bms::Task)
            .iter()
            .map(|task| task.as_str_name().to_owned())
            .collect(),

        Resource::Emg => iterate_enum!(sgcp::emg::Task)
            .iter()
            .map(|task| task.as_str_name().to_owned())
            .collect(),

        Resource::Maestro => iterate_enum!(sgcp::maestro::Task)
            .iter()
            .map(|task| task.as_str_name().to_owned())
            .collect(),
    }
}
