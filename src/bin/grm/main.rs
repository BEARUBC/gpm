//! Grasp Remote Module
//! A TUI interface to the commands exposed by GPM for testing.

mod client;
mod utils;
mod widgets;

use client::{GpmClient, GpmResponse};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout, Margin},
    style::{Color, Modifier, Style, Styled, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, List, ListItem, Padding, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
};
use std::{
    error::Error,
    io::{self},
};
use widgets::{
    nested_list::{FlattenedListNode, NestedListNode},
    stateful_list::StatefulList,
};

enum GrmBlock {
    CommandList,
    History,
}

struct App {
    should_quit: bool,
    command_tree: Vec<NestedListNode>,
    command_list: StatefulList<FlattenedListNode>,
    gpm_client: GpmClient,
    responses: StatefulList<GpmResponse>,
    current_block: GrmBlock,
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
            responses: StatefulList::new(),
            current_block: GrmBlock::CommandList,
        }
    }

    fn get_palette(resource: gpm::sgcp::Resource) -> Color {
        match resource {
            gpm::sgcp::Resource::UndefinedComponent => Color::Red,
            gpm::sgcp::Resource::Bms => Color::LightGreen,
            gpm::sgcp::Resource::Emg => Color::DarkGray,
            gpm::sgcp::Resource::Maestro => Color::Blue,
        }
    }

    fn toggle_selected_block(&mut self) {
        match self.current_block {
            GrmBlock::CommandList => self.current_block = GrmBlock::History,
            GrmBlock::History => self.current_block = GrmBlock::CommandList,
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
            KeyCode::Down | KeyCode::Char('j') => match self.current_block {
                GrmBlock::History => self.responses.next(),
                GrmBlock::CommandList => self.command_list.next(),
            },
            KeyCode::Up | KeyCode::Char('k') => match self.current_block {
                GrmBlock::History => self.responses.previous(),
                GrmBlock::CommandList => self.command_list.previous(),
            },
            KeyCode::Char('t') => self.toggle_selected_block(),
            KeyCode::Enter => {
                if let GrmBlock::CommandList = self.current_block {
                    self.command_list.state.selected().map_or((), |index| {
                        let node = self.command_list.items.get_mut(index).unwrap();

                        if node.is_root() {
                            node.toggle_expansion(&mut self.command_tree);

                            self.command_list = StatefulList::with_items(NestedListNode::flatten(
                                &self.command_tree,
                            ));

                            self.command_list.state.select(Some(
                                FlattenedListNode::clamp_selection(
                                    self.command_list.items.len(),
                                    index,
                                ),
                            ))
                        } else {
                            let root = node.path.get(0).unwrap();
                            let root_node = self.command_tree.get(*root).unwrap();
                            let task_code = node.path.get(1).unwrap();

                            let response = self
                                .gpm_client
                                .send(
                                    gpm::sgcp::Resource::from_str_name(root_node.name.as_str())
                                        .unwrap(),
                                    *task_code as i32 + 1, // +1 since we don't render task 0
                                )
                                .unwrap();
                            self.responses.items.push(response);
                            self.responses
                                .state
                                .select(Some(self.responses.items.len() - 1));

                            self.responses.scrollbar_state = self
                                .responses
                                .scrollbar_state
                                .position(self.responses.items.len() - 1);
                        }
                    })
                }
            },
            _ => {},
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let super_chunks =
            Layout::vertical([Constraint::Min(0), Constraint::Percentage(5)]).split(frame.area());

        // Footer
        let footer_panel = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        frame.render_widget(
            Line::raw(
                "<↑>/<k> <↓>/<j> move up/down | <enter> run command | <t> toggle tabs | <q> quit",
            )
            .style(Style::default().fg(Color::Cyan)),
            footer_panel.inner(super_chunks[1]),
        );

        // Main content

        let main_chunks =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(super_chunks[0]);

        let left_column = main_chunks[0];
        let right_column_chunks =
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(main_chunks[1]);

        // Status Block

        let right_panel_top = Block::default()
            .title("Status")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::new(1, 0, 0, 0));

        frame.render_widget(
            Paragraph::new("Connected to 127.0.0.1:4760")
                .style(Style::default().fg(Color::LightGreen)),
            right_panel_top.inner(right_column_chunks[0]),
        );

        frame.render_widget(right_panel_top, right_column_chunks[0]);

        // History Block

        let mut right_panel_bottom = Block::default()
            .title("History")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::new(1, 1, 0, 0));

        let inner_area = right_panel_bottom.inner(right_column_chunks[1]);

        if let GrmBlock::History = self.current_block {
            right_panel_bottom = right_panel_bottom.border_style(Style::default().bold());
        }

        frame.render_widget(right_panel_bottom, right_column_chunks[1]);

        if !self.responses.items.is_empty() {
            let base_widget = List::new(self.responses.items.iter().map(|v| {
                ListItem::new(Text::from(vec![Line::from(vec![
                    Span::styled(
                        format!("{}::{}", v.resource.as_str_name(), &v.task_code),
                        Style::default()
                            .fg(Self::get_palette(v.resource))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" ", Style::default()),
                    Span::styled(&v.message, Style::default()),
                ])]))
            }))
            .highlight_style(Style::default().bold());

            frame.render_stateful_widget(base_widget, inner_area, &mut self.responses.state);
        }

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        self.responses.scrollbar_state = ScrollbarState::new(self.responses.items.len())
            .position(self.responses.state.selected().unwrap_or(0));

        frame.render_stateful_widget(
            scrollbar,
            right_column_chunks[1].inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.responses.scrollbar_state,
        );

        // Command List Block

        let base_widget = List::new(self.command_list.items.iter().map(|v| ListItem::new(v)))
            .block(
                Block::default()
                    .title("Resources")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded)
                    .padding(Padding::new(1, 1, 0, 0))
                    .border_style(if let GrmBlock::CommandList = self.current_block {
                        Style::default().bold()
                    } else {
                        Style::default()
                    }),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(base_widget, left_column, &mut self.command_list.state)
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
