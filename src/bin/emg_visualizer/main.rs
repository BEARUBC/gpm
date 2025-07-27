//! Real-time EMG Data Visualizer

mod client;
mod error;
mod tui;

use chrono::{DateTime, Local, Utc};
use client::EmgExporterClient;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::*,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
};
use serde::Deserialize;
use std::time::Duration;
use tokio::sync::mpsc;

//////////////////////////////////
/// Config
//////////////////////////////////

/// The address of the TCP server providing EMG data.
const SERVER_ADDR: &str = "localhost:9998";
/// The time window for the charts, in milliseconds.
const TIME_WINDOW_MS: f64 = 30_000.0;
/// The polling rate for UI events, in milliseconds.
const EVENT_POLL_RATE_MS: u64 = 50;
/// The buffer size for the MPSC channel.
const CHANNEL_CAPACITY: usize = 100;
/// Color for the first data channel.
const CHANNEL_1_COLOR: Color = Color::Cyan;
/// Color for the second data channel.
const CHANNEL_2_COLOR: Color = Color::Magenta;
/// Padding added to the Y-axis bounds for better visualization.
const Y_AXIS_PADDING: f64 = 10.0;

type AppResult<T> = Result<T, error::AppError>;

/// Represents a single data point received from the server.
#[derive(Deserialize, Debug, Clone, Copy)]
struct DataPoint {
    channel_0: f64,
    channel_1: f64,
    timestamp: u64, // Unix timestamp in milliseconds
}

#[derive(Debug)]
struct App {
    /// Data for the first channel, stored as `(timestamp, value)`.
    data_ch1: Vec<(f64, f64)>,
    /// Data for the second channel, stored as `(timestamp, value)`.
    data_ch2: Vec<(f64, f64)>,
    /// The visible time window for the x-axis, as `[start, end]`.
    window: [f64; 2],
}

impl App {
    /// Creates a new `App` instance with an initial time window.
    fn new() -> Self {
        let now = Utc::now().timestamp_millis() as f64;
        Self {
            data_ch1: Vec::new(),
            data_ch2: Vec::new(),
            window: [now - TIME_WINDOW_MS, now],
        }
    }

    /// Adds a new data point to the state and updates the time window.
    fn add_data(&mut self, point: DataPoint) {
        let timestamp_f64 = point.timestamp as f64;
        self.data_ch1.push((timestamp_f64, point.channel_0));
        self.data_ch2.push((timestamp_f64, point.channel_1));

        self.window[1] = timestamp_f64;
        self.window[0] = timestamp_f64 - TIME_WINDOW_MS;

        self.data_ch1.retain(|(t, _)| *t >= self.window[0]);
        self.data_ch2.retain(|(t, _)| *t >= self.window[0]);
    }

    //////////////////////////////////
    /// Rendering
    //////////////////////////////////

    /// Renders the user interface widgets.
    fn render(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(f.area());

        let x_axis = Self::create_x_axis(self.window);

        // Render Chart 1
        let y_axis1 = Self::create_y_axis(&self.data_ch1);
        let chart1 = Self::draw_channel_chart(
            "Channel 1 EMG Data",
            &self.data_ch1,
            CHANNEL_1_COLOR,
            x_axis.clone(),
            y_axis1,
        );
        f.render_widget(chart1, chunks[0]);

        // Render Chart 2
        let y_axis2 = Self::create_y_axis(&self.data_ch2);
        let chart2 = Self::draw_channel_chart(
            "Channel 2 EMG Data",
            &self.data_ch2,
            CHANNEL_2_COLOR,
            x_axis,
            y_axis2,
        );
        f.render_widget(chart2, chunks[1]);
    }

    /// Creates a configured X-axis based on the current time window.
    fn create_x_axis(window: [f64; 2]) -> Axis<'static> {
        let labels: Vec<Span> = window
            .iter()
            .map(|&t| {
                let dt = DateTime::from_timestamp_millis(t as i64)
                    .unwrap_or_default()
                    .with_timezone(&Local);
                Span::from(dt.format("%H:%M:%S").to_string())
            })
            .collect();

        Axis::default()
            .title("Time")
            .style(Style::default().gray())
            .bounds(window)
            .labels(labels)
    }

    /// Creates a dynamically scaled Y-axis based on the provided data.
    fn create_y_axis(data: &[(f64, f64)]) -> Axis<'static> {
        let (min, max) = data
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), &(_, v)| {
                (min.min(v), max.max(v))
            });

        let (min, max) = if min.is_infinite() {
            (0.0, 100.0)
        } else {
            (min, max)
        };

        let bounds = [min - Y_AXIS_PADDING, max + Y_AXIS_PADDING];
        let labels = vec![
            Span::from(format!("{:.1}", min)),
            Span::from(format!("{:.1}", max)),
        ];

        Axis::default()
            .title("Value")
            .style(Style::default().gray())
            .bounds(bounds)
            .labels(labels)
    }

    /// Draws a single channel chart
    fn draw_channel_chart<'a>(
        title: &'a str,
        data: &'a [(f64, f64)],
        color: Color,
        x_axis: Axis<'a>,
        y_axis: Axis<'a>,
    ) -> Chart<'a> {
        let dataset = Dataset::default()
            .name(title)
            .marker(symbols::Marker::HalfBlock)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(color))
            .data(data);

        Chart::new(vec![dataset])
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::Reset)),
            )
            .x_axis(x_axis)
            .y_axis(y_axis)
    }
}

#[tokio::main]
async fn main() -> AppResult<()> {
    let (tx, rx) = mpsc::channel::<DataPoint>(CHANNEL_CAPACITY);
    let client = EmgExporterClient::new(tx);

    // Spawn the network task to produce data.
    tokio::spawn(async move {
        if let Err(e) = client.listen_for_data().await {
            eprintln!("Network task failed: {}", e);
        }
    });

    let mut tui = tui::Tui::new()?;
    run_ui_loop(&mut tui.terminal, rx).await?;

    Ok(())
}

async fn run_ui_loop<B: Backend>(
    terminal: &mut Terminal<B>,
    mut rx: mpsc::Receiver<DataPoint>,
) -> AppResult<()> {
    let tick_rate = Duration::from_millis(EVENT_POLL_RATE_MS);
    let mut app = App::new();

    loop {
        // QUESTION: Consumes faster than the network bound producer -- so it should never be stuck in a
        // infiinite loop...maybe?
        while let Ok(point) = rx.try_recv() {
            app.add_data(point);
        }

        terminal.draw(|f| app.render(f))?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    return Ok(());
                }
            }
        }
    }
}
