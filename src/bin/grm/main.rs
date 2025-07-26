mod client;
mod widgets;

use client::GpmClient;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem},
};
use std::{
    error::Error,
    io::{self},
};
use widgets::{
    nested_list::{FlattenedListNode, NestedListNode},
    stateful_list::StatefulList,
};

struct App {
    should_quit: bool,
    command_tree: Vec<NestedListNode>,
    command_list: StatefulList<FlattenedListNode>,
    gpm_client: GpmClient,
}

impl App {
    fn new() -> Self {
        let command_tree: Vec<NestedListNode> = gpm::enum_values!(gpm::sgcp::Resource)
            .iter()
            .map(|resource| {
                NestedListNode::with_children(
                    resource.as_str_name(),
                    gpm::get_tasks_for_resource(resource)
                        .iter()
                        .map(|task_name| NestedListNode::new(task_name))
                        .collect(),
                )
            })
            .collect();

        let command_list = StatefulList::with_items(NestedListNode::flatten(&command_tree));

        Self {
            should_quit: false,
            command_tree,
            command_list,
            gpm_client: GpmClient::new(),
        }
    }

    fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;

            if let Event::Key(key) = event::read()? {
                self.handle_key_events(key);
            }

            if self.should_quit {
                return Ok(());
            }
        }
    }

    fn handle_key_events(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Down | KeyCode::Char('j') => self.command_list.next(),
            KeyCode::Up | KeyCode::Char('k') => self.command_list.previous(),
            KeyCode::Enter => self.command_list.state.selected().map_or((), |index| {
                let node = self.command_list.items.get_mut(index).unwrap();

                if node.is_root() {
                    node.toggle_expansion(&mut self.command_tree);

                    self.command_list =
                        StatefulList::with_items(NestedListNode::flatten(&self.command_tree));

                    self.command_list
                        .state
                        .select(Some(FlattenedListNode::clamp_selection(
                            self.command_list.items.len(),
                            index,
                        )))
                } else {
                    let root = node.path.get(0).unwrap();
                    let root_node = self.command_tree.get(*root).unwrap();
                    let task_code = node.path.get(1).unwrap();

                    self.gpm_client
                        .send(
                            gpm::sgcp::Resource::from_str_name(root_node.name.as_str()).unwrap(),
                            *task_code as i32 + 1, // +1 since we don't render task 0
                        )
                        .unwrap();
                }
            }),
            _ => {},
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

        frame.render_stateful_widget(
            List::new(self.command_list.items.iter().map(|v| ListItem::new(v)))
                .block(
                    Block::default()
                        .title("GRASP Resources")
                        .borders(Borders::all())
                        .border_type(BorderType::Rounded),
                )
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> "),
            left_column,
            &mut self.command_list.state,
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
