use chrono::{DateTime, Utc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    prelude::*,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
};
use serde::Deserialize;
use std::{
    error::Error,
    io,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;

#[derive(Deserialize, Debug, Clone, Copy)]
struct DataPoint {
    channel_0: f64,
    channel_1: f64,
    timestamp: u64, // Unix timestamp in milliseconds
}

struct App {
    data_ch1: Vec<(f64, f64)>,
    data_ch2: Vec<(f64, f64)>,
    window: [f64; 2],
}

impl App {
    fn new() -> Self {
        let now = Utc::now().timestamp_millis() as f64;
        Self {
            data_ch1: Vec::new(),
            data_ch2: Vec::new(),
            // Display the last 30 seconds of data
            window: [now - 30_000.0, now],
        }
    }

    fn add_data(&mut self, point: DataPoint) {
        let timestamp_f64 = point.timestamp as f64;
        self.data_ch1.push((timestamp_f64, point.channel_0));
        self.data_ch2.push((timestamp_f64, point.channel_1));

        self.window[1] = timestamp_f64;
        self.window[0] = timestamp_f64 - 30_000.0;

        self.data_ch1.retain(|(t, _)| *t >= self.window[0]);
        self.data_ch2.retain(|(t, _)| *t >= self.window[0]);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = Arc::new(Mutex::new(App::new()));
    let app_clone = app.clone();

    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            if let Ok(stream) = TcpStream::connect("localhost:9998").await {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                    match serde_json::from_str::<DataPoint>(&line) {
                        Ok(point) => app_clone.lock().unwrap().add_data(point),
                        Err(err) => println!("{:?}", err),
                    }
                    line.clear();
                }
            } else {
                // TODO: @kumarpit error handling
            }
        });
    });

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: Arc<Mutex<App>>) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app.lock().unwrap()))?;

        // Exit on 'q'
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    return Ok(());
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    // --- Chart 1 ---
    let dataset_ch1 = Dataset::default()
        .name("Channel 1")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().cyan())
        .data(&app.data_ch1);

    let labels: Vec<Span> = app
        .window
        .iter()
        .map(|t: &f64| {
            let dt: DateTime<Utc> =
                chrono::DateTime::from_timestamp_millis(*t as i64).unwrap_or_default();
            Span::from(dt.format("%H:%M:%S").to_string())
        })
        .collect();

    let x_axis: Axis = Axis::default()
        .title("Time")
        .style(Style::default().gray())
        .bounds(app.window)
        .labels(labels);

    // Find min/max for dynamic y-axis scaling
    let min_y1 = app
        .data_ch1
        .iter()
        .map(|(_, v)| *v)
        .fold(f64::INFINITY, f64::min);
    let max_y1 = app
        .data_ch1
        .iter()
        .map(|(_, v)| *v)
        .fold(f64::NEG_INFINITY, f64::max);

    let y_axis1 = Axis::default()
        .title("Value")
        .style(Style::default().gray())
        .bounds([min_y1 - 10.0, max_y1 + 10.0]) // Add padding
        .labels(vec![
            Span::from(format!("{:.1}", min_y1)),
            Span::from(format!("{:.1}", max_y1)),
        ]);

    let chart1 = Chart::new(vec![dataset_ch1])
        .block(
            Block::default()
                .title("Channel 1 EMG Data")
                .borders(Borders::ALL),
        )
        .x_axis(x_axis.clone()) // Clone x_axis for the second chart
        .y_axis(y_axis1);

    f.render_widget(chart1, chunks[0]);

    // --- Chart 2 ---
    let dataset_ch2 = Dataset::default()
        .name("Channel 2")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().magenta())
        .data(&app.data_ch2);

    let min_y2 = app
        .data_ch2
        .iter()
        .map(|(_, v)| *v)
        .fold(f64::INFINITY, f64::min);
    let max_y2 = app
        .data_ch2
        .iter()
        .map(|(_, v)| *v)
        .fold(f64::NEG_INFINITY, f64::max);

    let y_axis2 = Axis::default()
        .title("Value")
        .style(Style::default().gray())
        .bounds([min_y2 - 10.0, max_y2 + 10.0])
        .labels(vec![
            Span::from(format!("{:.1}", min_y2)),
            Span::from(format!("{:.1}", max_y2)),
        ]);

    let chart2 = Chart::new(vec![dataset_ch2])
        .block(
            Block::default()
                .title("Channel 2 EMG Data")
                .borders(Borders::ALL),
        )
        .x_axis(x_axis)
        .y_axis(y_axis2);

    f.render_widget(chart2, chunks[1]);
}
