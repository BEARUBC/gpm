use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use gpm::import_sgcp;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Stylize, palette::tailwind},
    symbols,
    text::Line,
    widgets::{Block, Padding, Paragraph, Tabs, Widget},
};

import_sgcp!();

const ALL_SGCP_RESOURCES: &[sgcp::Resource] = &[
    sgcp::Resource::UndefinedComponent,
    sgcp::Resource::Bms,
    sgcp::Resource::Emg,
    sgcp::Resource::Maestro,
];

fn main() -> Result<()> {
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

impl sgcp::Resource {
    fn previous(self) -> Self {
        let current_index: i32 = self as i32;
        Self::try_from(current_index.saturating_sub(1)).unwrap_or(self)
    }

    fn next(self) -> Self {
        let current_index: i32 = self as i32;
        Self::try_from(current_index.saturating_add(1)).unwrap_or(self)
    }
}

#[derive(Debug, Default)]
pub struct App {
    running: bool,
    selected_tab: sgcp::Resource,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    ///////////////////////////////
    // State Management
    ///////////////////////////////

    pub fn next_tab(&mut self) {
        self.selected_tab = self.selected_tab.next();
    }

    pub fn previous_tab(&mut self) {
        self.selected_tab = self.selected_tab.previous();
    }

    ///////////////////////////////
    // Rendering
    ///////////////////////////////

    fn render(&mut self, frame: &mut Frame) {
        use Constraint::{Length, Min};

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Min(0), Length(10)])
            .split(frame.area());

        // Render history

        let history_title = Line::from("GRASP Remote Module").bold().blue().centered();

        frame.render_widget(
            Paragraph::new("")
                .block(Block::bordered().title(history_title))
                .centered(),
            main_layout[0],
        );

        // Render tabs

        let tab_area = main_layout[1];

        let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);
        let [header_area, inner_area, footer_area] = vertical.areas(tab_area);

        let horizontal = Layout::horizontal([Min(0), Length(20)]);
        let [tabs_area, title_area] = horizontal.areas(header_area);

        let tab_title = Line::from("GRASP Resources").bold().centered();

        frame.render_widget(tab_title, header_area);
        self.render_tabs(tabs_area, frame.buffer_mut());
    }

    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = ALL_SGCP_RESOURCES
            .iter()
            .map(|resource| resource.as_str_name());

        let highlight_style = (Color::default(), self.selected_tab.palette().c700);
        let selected_tab_index = self.selected_tab as i32;

        Tabs::new(titles)
            .highlight_style(highlight_style)
            .padding("", "")
            .divider(" ")
            .render(area, buf);
    }

    ///////////////////////////////
    // Event Handling
    ///////////////////////////////

    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {},
            Event::Resize(_, _) => {},
            _ => {},
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            _ => {},
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }
}

///////////////////////////////
// Tab Rendering
///////////////////////////////

impl Widget for sgcp::Resource {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        match self {
            Resource::UndefinedComponent => self.render_tab(area, buf),
            Resource::Bms => self.render_tab(area, buf),
            Resource::Emg => self.render_tab(area, buf),
            Resource::Maestro => self.render_tab(area, buf),
        }
    }
}

impl sgcp::Resource {
    fn title(self) -> Line<'static> {
        format!("  {:?}  ", self)
            .fg(tailwind::SLATE.c200)
            .bg(self.palette().c900)
            .into()
    }

    fn render_tab(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        Paragraph::new("Hello world!")
            .block(self.block())
            .render(area, buf)
    }

    /// A block surrounding the tab's content
    fn block(self) -> Block<'static> {
        Block::bordered()
            .border_set(symbols::border::PROPORTIONAL_TALL)
            .padding(Padding::horizontal(1))
            .border_style(self.palette().c700)
    }

    const fn palette(self) -> tailwind::Palette {
        match self {
            Self::UndefinedComponent => tailwind::BLUE,
            Self::Bms => tailwind::EMERALD,
            Self::Emg => tailwind::INDIGO,
            Self::Maestro => tailwind::RED,
        }
    }
}
