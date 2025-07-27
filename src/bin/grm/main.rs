//! Grasp Remote Module
//! A TUI interface to the commands exposed by GPM for testing.

mod client;
mod utils;
mod widgets;

use client::GpmResponse;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
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
use widgets::{command_list::CommandList, stateful_list::StatefulList};

/// The primary, user-selectable blocks in the TUI
enum GrmBlock {
    /// Renders GPM Command list
    CommandList,
    /// Renders command history
    History,
}

struct App {
    should_quit: bool,
    command_list: CommandList,
    responses: StatefulList<GpmResponse>,
    selected_block: GrmBlock,
}

impl App {
    fn new() -> Self {
        Self {
            should_quit: false,
            command_list: CommandList::init(),
            responses: StatefulList::new(),
            selected_block: GrmBlock::CommandList,
        }
    }

    /// Main app loop
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

    //////////////////////////////////
    /// Event Handling
    //////////////////////////////////

    fn handle_key_events(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,

            KeyCode::Down | KeyCode::Char('j') => match self.selected_block {
                GrmBlock::History => self.responses.next(),
                GrmBlock::CommandList => self.command_list.next(),
            },

            KeyCode::Up | KeyCode::Char('k') => match self.selected_block {
                GrmBlock::History => self.responses.previous(),
                GrmBlock::CommandList => self.command_list.previous(),
            },

            KeyCode::Char('t') => self.toggle_selected_block(),

            KeyCode::Enter => {
                if let GrmBlock::CommandList = self.selected_block {
                    match self.command_list.handle_select() {
                        Some(response) => {
                            self.responses.items.push(response);
                            self.responses
                                .state
                                .select(Some(self.responses.items.len() - 1));

                            self.responses.scrollbar_state = self
                                .responses
                                .scrollbar_state
                                .position(self.responses.items.len() - 1);
                        },
                        None => (),
                    }
                }
            },
            _ => {},
        }
    }

    /// Switch between the two main blocks -- History and Command List
    fn toggle_selected_block(&mut self) {
        match self.selected_block {
            GrmBlock::CommandList => self.selected_block = GrmBlock::History,
            GrmBlock::History => self.selected_block = GrmBlock::CommandList,
        }
    }

    //////////////////////////////////
    /// Rendering
    //////////////////////////////////

    fn render(&mut self, frame: &mut Frame) {
        let parent_chunks =
            Layout::vertical([Constraint::Min(0), Constraint::Percentage(5)]).split(frame.area());

        let main_area = parent_chunks[0];
        let footer_area = parent_chunks[1];

        let main_chunks =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(main_area);

        let left_column_area = main_chunks[0];

        let right_column_chunks =
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(main_chunks[1]);
        let top_right_column_area = right_column_chunks[0];
        let bottom_right_column_area = right_column_chunks[1];

        self.render_command_list_block(frame, left_column_area);
        self.render_status_block(frame, top_right_column_area);
        self.render_history_block(frame, bottom_right_column_area);
        self.render_footer_block(frame, footer_area);
    }

    fn render_history_block(&mut self, frame: &mut Frame, area: Rect) {
        let mut right_panel_bottom = Block::default()
            .title("History")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::new(1, 1, 0, 0));

        let inner_area = right_panel_bottom.inner(area);

        if let GrmBlock::History = self.selected_block {
            right_panel_bottom = right_panel_bottom.border_style(Style::default().bold());
        }

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
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.responses.scrollbar_state,
        );

        frame.render_widget(right_panel_bottom, area);
    }

    /// Text displaying the connected server address
    fn render_status_block(&mut self, frame: &mut Frame, area: Rect) {
        let panel = Block::default()
            .title("Status")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::new(1, 0, 0, 0));

        frame.render_widget(
            Paragraph::new("Connected to 127.0.0.1:4760")
                .style(Style::default().fg(Color::LightGreen)),
            panel.inner(area),
        );

        frame.render_widget(panel, area);
    }

    /// Includes minimal help text
    fn render_footer_block(&mut self, frame: &mut Frame, area: Rect) {
        let panel = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        frame.render_widget(
            Line::raw(
                "<↑>/<k> <↓>/<j> move up/down | <enter> run command | <t> toggle tabs | <q> quit",
            )
            .style(Style::default().fg(Color::Cyan)),
            panel.inner(area),
        );
    }

    /// Renders the Command list in the left column
    fn render_command_list_block(&mut self, frame: &mut Frame, area: Rect) {
        let command_list_widget = List::from(&self.command_list)
            .block(
                Block::default()
                    .title("Resources")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded)
                    .padding(Padding::new(1, 1, 0, 0))
                    .border_style(if let GrmBlock::CommandList = self.selected_block {
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

        frame.render_stateful_widget(
            command_list_widget,
            area,
            self.command_list.get_list_state(),
        )
    }

    /// History logs for each resource get their own color
    fn get_palette(resource: gpm::sgcp::Resource) -> Color {
        match resource {
            gpm::sgcp::Resource::UndefinedComponent => Color::Red,
            gpm::sgcp::Resource::Bms => Color::LightGreen,
            gpm::sgcp::Resource::Emg => Color::DarkGray,
            gpm::sgcp::Resource::Maestro => Color::Blue,
        }
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
