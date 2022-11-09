use tui::{
    terminal::Frame,
    backend::Backend,
    layout::{Constraint, Layout, Direction},
    style::{Color, Style},
    widgets::{Gauge, BarChart, Borders, Block},
};

use crate::app::{self, Timing};
use super::DURATION;

pub fn draw<B: Backend>(f: &mut Frame<B> ,app: &mut app::App) {
    draw_main(f, app);
    // match app.tabs.index {
    //     0 => draw_first_tab(f, app, chunks[1]),
    //     1 => draw_second_tab(f, app, chunks[1]),
    //     2 => draw_third_tab(f, app, chunks[1]),
    //     _ => {}
    // };
}


// atm have just have one screen (main), in the future implement more screens to show wider scope of data
// like past 6 months...
fn draw_main<B: Backend>(f: &mut Frame<B>, app: &app::App) {
    let barchart_data: Vec<(&str, u64)> = app.display_data.data
        .iter()
        .map(|t| (&t.0[..], t.1))
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Percentage(90),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(f.size());
    
    let bar_width = f.size().width / 35;
    let barchart = BarChart::default()
        .block(Block::default().title("past 30 days".to_string()).borders(Borders::ALL))
        .data(&barchart_data) // data in the form of (label, height) 
        .bar_width(bar_width)
        .bar_gap(1)
        .max(15)
        .bar_style(Style::default().fg(Color::DarkGray))
        .value_style(Style::default().fg(Color::Black).bg(Color::Red));
    f.render_widget(barchart, chunks[0]);

    // progress gauge
    let mut ratio: f64 = 0.0;
    let mut background_color = Color::DarkGray;
    let mut label = String::from("0");
    let mut title = String::from("inactive");

    if let Some(Timing{cur_time, ..}) = app.in_progress {
        ratio = cur_time as f64 / (DURATION) as f64;
        background_color = Color::Blue;
        let remaining_secs = DURATION - cur_time;
        let secs = remaining_secs % 60;
        let mins = remaining_secs / 60;
        let mins_remaning = (DURATION - cur_time) / 60;
        let secs_remaining = 60 - cur_time % 60; 
        label = format!("{}:{} remaining", mins, secs);

        title = "active".into();
    }

    let gauge = Gauge::default()
        .block(Block::default().title(title).borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::DarkGray).bg(background_color))
        .ratio(ratio)
        .label(label);
    f.render_widget(gauge, chunks[1]);
}
