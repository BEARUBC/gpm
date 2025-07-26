mod widgets;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    widgets::{Block, BorderType, Borders, List, ListItem},
};
use std::{
    error::Error,
    io::{self},
};
use widgets::nested_list::TreeNode;

struct App {
    should_quit: bool,
    command_list: Vec<TreeNode>,
}

impl App {
    fn new() -> Self {
        Self {
            should_quit: false,
            command_list: gpm::iterate_enum!(gpm::sgcp::Resource)
                .iter()
                .map(|resource| TreeNode {
                    name: resource.as_str_name().to_owned(),
                    children: gpm::get_tasks_for_resource(resource)
                        .iter()
                        .map(|task_name| TreeNode {
                            name: task_name.to_owned(),
                            children: Vec::new(),
                            is_expanded: false,
                        })
                        .collect(),
                    is_expanded: false,
                })
                .collect(),
        }
    }

    fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;

            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    self.should_quit = true;
                }
            }

            if self.should_quit {
                return Ok(());
            }
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let main_chunks =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(frame.area());

        let left_column = main_chunks[0];
        let right_column_chunks =
            Layout::vertical([Constraint::Percentage(20), Constraint::Min(0)])
                .split(main_chunks[1]);

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

        let list_items: Vec<ListItem> = self
            .command_list
            .iter_mut()
            .flat_map(|tree| tree.flatten_tree())
            .map(|list| ListItem::new(list))
            .collect();

        frame.render_widget(
            List::new(list_items).block(
                Block::default()
                    .title("GRASP Resources")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded),
            ),
            left_column,
        )
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = App::new().run(&mut terminal);

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
