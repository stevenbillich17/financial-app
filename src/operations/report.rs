use crate::db::repository;
use crate::models::transaction::Transaction;
use chrono::{Duration, NaiveDate};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::{Alignment, Color, Constraint, Direction, Layout, Rect, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use ratatui::widgets::canvas::{Canvas, Points};
use rusqlite::Connection;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::collections::HashMap;
use std::io;

pub fn run_report(conn: &Connection, start_date: NaiveDate, end_date: NaiveDate) -> Result<(), String> {
    if start_date > end_date {
        return Err("Start date must be before end date.".to_string());
    }

    let total_days = (end_date - start_date).num_days().max(0) + 1;
    let bucket_days = if total_days <= 7 {
        1
    } else if total_days <= 90 {
        7
    } else if total_days <= 365 {
        14
    } else {
        ((total_days + 19) / 20) as i64
    };

    let title = format!(
        "{} - {} ({}-day buckets)",
        start_date.format("%d.%m.%Y"),
        end_date.format("%d.%m.%Y"),
        bucket_days
    );

    let transactions = repository::get_expense_transactions_in_range(conn, start_date, end_date)?;
    let report = build_report(&transactions, start_date, end_date, total_days, bucket_days);

    render_report(&title, &report)?;
    Ok(())
}

struct ReportData {
    buckets: Vec<BucketData>,
    category_totals: Vec<(String, Decimal)>,
    category_colors: HashMap<String, Color>,
    total_spend: Decimal,
}

struct BucketData {
    start: NaiveDate,
    end: NaiveDate,
    totals: Vec<(String, Decimal)>,
    total: Decimal,
}

fn build_report(
    transactions: &[Transaction],
    start_date: NaiveDate,
    end_date: NaiveDate,
    total_days: i64,
    bucket_days: i64,
) -> ReportData {
    let bucket_count = ((total_days as f64) / (bucket_days as f64)).ceil() as usize;

    let mut bucket_maps: Vec<HashMap<String, Decimal>> =
        vec![HashMap::new(); bucket_count.max(1)];
    let mut category_totals: HashMap<String, Decimal> = HashMap::new();

    for transaction in transactions {
        let idx = bucket_index(start_date, transaction.date, bucket_days, bucket_count);
        let amount = transaction.amount.abs();
        let entry = bucket_maps[idx]
            .entry(transaction.category.clone())
            .or_insert(Decimal::ZERO);
        *entry += amount;

        let total_entry = category_totals
            .entry(transaction.category.clone())
            .or_insert(Decimal::ZERO);
        *total_entry += amount;
    }

    let mut categories: Vec<String> = category_totals.keys().cloned().collect();
    categories.sort();
    let category_colors = assign_colors(&categories);

    let mut buckets = Vec::new();
    for i in 0..bucket_count.max(1) {
        let bucket_start = start_date + Duration::days(i as i64 * bucket_days);
        let bucket_end = (bucket_start + Duration::days(bucket_days - 1)).min(end_date);
        let mut totals: Vec<(String, Decimal)> = bucket_maps[i]
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        // use partial_cmp so if they are not comparable NaN < 123 
        // => Equal (some() -> unwrap Less, Equal, Greater or from None -> Equal)
        totals.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)); 
        let total = totals.iter().fold(Decimal::ZERO, |acc, (_, v)| acc + *v);
        buckets.push(BucketData {
            start: bucket_start,
            end: bucket_end,
            totals,
            total,
        });
    }

    let mut category_totals_vec: Vec<(String, Decimal)> =
        category_totals.into_iter().collect();
    category_totals_vec
        .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let total_spend = category_totals_vec
        .iter()
        .fold(Decimal::ZERO, |acc, (_, v)| acc + *v);

    ReportData {
        buckets,
        category_totals: category_totals_vec,
        category_colors,
        total_spend,
    }
}

fn bucket_index(
    start_date: NaiveDate,
    date: NaiveDate,
    bucket_days: i64,
    bucket_count: usize,
) -> usize {
    if date < start_date {
        return 0;
    }
    let diff = (date - start_date).num_days();
    let idx = (diff / bucket_days) as usize;
    idx.min(bucket_count.saturating_sub(1))
}

fn assign_colors(categories: &[String]) -> HashMap<String, Color> {
    let palette = vec![
        Color::Cyan,
        Color::Magenta,
        Color::Yellow,
        Color::Green,
        Color::Blue,
        Color::Red,
        Color::LightCyan,
        Color::LightMagenta,
        Color::LightYellow,
        Color::LightGreen,
        Color::LightBlue,
    ];

    let mut map = HashMap::new();
    for (idx, category) in categories.iter().enumerate() {
        map.insert(category.clone(), palette[idx % palette.len()]);
    }
    map
}

fn render_report(title: &str, data: &ReportData) -> Result<(), String> {
    enable_raw_mode().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| format!("Failed to enter alternate screen: {}", e))?;

    let result = (|| {
        let backend = ratatui::backend::CrosstermBackend::new(stdout);
        let mut terminal = ratatui::Terminal::new(backend)
            .map_err(|e| format!("Failed to initialize terminal: {}", e))?;

        loop {
            terminal
                .draw(|frame| {
                    let size = frame.area();
                    let layout = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage(60),
                            Constraint::Percentage(40),
                        ])
                        .split(size);

                    render_bar_chart(frame, layout[0], title, data);

                    let bottom = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(55),
                            Constraint::Percentage(45),
                        ])
                        .split(layout[1]);

                    render_pie_chart(frame, bottom[0], data);
                    render_category_table(frame, bottom[1], data);
                })
                .map_err(|e| format!("Failed to draw terminal UI: {}", e))?;

            if event::poll(std::time::Duration::from_millis(250))
                .map_err(|e| format!("Failed to poll input: {}", e))?
            {
                match event::read().map_err(|e| format!("Failed to read input: {}", e))? {
                    Event::Key(key) if key.code == KeyCode::Char('q') => break,
                    Event::Key(key) if key.code == KeyCode::Esc => break,
                    Event::Resize(_, _) => continue,
                    _ => {}
                }
            }
        }

        Ok(())
    })();

    disable_raw_mode().map_err(|e| format!("Failed to disable raw mode: {}", e))?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen)
        .map_err(|e| format!("Failed to leave alternate screen: {}", e))?;

    result
}

fn render_bar_chart(frame: &mut ratatui::Frame, area: Rect, title: &str, data: &ReportData) {
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(area);

    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            format!("{}  (press q to exit)", title),
            Style::default().fg(Color::White),
        )]))
        .borders(Borders::ALL);

    let chart_area = block.inner(inner[0]);
    frame.render_widget(block, inner[0]);

    let bar_height = chart_area.height.saturating_sub(1) as usize;
    if bar_height == 0 || data.buckets.is_empty() {
        return;
    }

    let bucket_count = data.buckets.len();
    let bucket_width = std::cmp::max(1, chart_area.width as usize / bucket_count);

    let max_total = data
        .buckets
        .iter()
        .map(|b| b.total.to_f64().unwrap_or(0.0))
        .fold(0.0_f64, f64::max)
        .max(1.0);

    let mut lines: Vec<Line> = Vec::new();

    for row in 0..bar_height {
        let mut spans: Vec<Span> = Vec::new();
        let level = (bar_height - row) as f64;

        for bucket in &data.buckets {
            let total = bucket.total.to_f64().unwrap_or(0.0);
            if total <= 0.0 {
                spans.push(Span::raw(" ".repeat(bucket_width)));
                continue;
            }

            let scaled_height = (total / max_total * bar_height as f64).ceil();
            if level > scaled_height {
                spans.push(Span::raw(" ".repeat(bucket_width)));
                continue;
            }

            let category_heights = compute_category_heights(&bucket.totals, total, bar_height);
            let mut current_height = 0usize;
            let mut color = Color::DarkGray;
            for (category, height) in category_heights {
                current_height += height;
                if (bar_height - row) <= current_height {
                    color = data
                        .category_colors
                        .get(&category)
                        .copied()
                        .unwrap_or(Color::White);
                    break;
                }
            }
            spans.push(Span::styled("█".repeat(bucket_width), Style::default().fg(color)));
        }
        lines.push(Line::from(spans));
    }

    let chart = Paragraph::new(lines).alignment(Alignment::Left);
    frame.render_widget(chart, chart_area);

    let labels = build_bucket_labels(&data.buckets, chart_area.width as usize, bucket_width);
    let label_paragraph = Paragraph::new(labels)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(label_paragraph, inner[1]);
}

fn build_bucket_labels(buckets: &[BucketData], width: usize, bucket_width: usize) -> Vec<Line> {
    if buckets.is_empty() {
        return vec![Line::from("")];
    }

    if bucket_width < 4 {
        return vec![Line::from(" ".repeat(width))];
    }

    let mut spans = Vec::new();
    for bucket in buckets {
        let label = if bucket.start == bucket.end {
            bucket.start.format("%m-%d").to_string()
        } else {
            format!("{}", bucket.start.format("%m-%d"))
        };
        let mut label = label;
        if label.len() > bucket_width {
            label.truncate(bucket_width);
        }
        let padded = format!("{:width$}", label, width = bucket_width);
        spans.push(Span::raw(padded));
    }

    vec![Line::from(spans)]
}

fn compute_category_heights(
    totals: &[(String, Decimal)],
    bucket_total: f64,
    bar_height: usize,
) -> Vec<(String, usize)> {
    if bucket_total <= 0.0 {
        return totals.iter().map(|(c, _)| (c.clone(), 0)).collect();
    }

    let mut heights: Vec<(String, usize, f64)> = totals
        .iter()
        .map(|(c, v)| {
            let amount = v.to_f64().unwrap_or(0.0);
            let exact = amount / bucket_total * bar_height as f64;
            let floor = exact.floor() as usize;
            (c.clone(), floor, exact - floor as f64)
        })
        .collect();

    let mut used: usize = heights.iter().map(|(_, h, _)| *h).sum();
    if used == 0 {
        used = 0;
    }
    let mut remaining = bar_height.saturating_sub(used);
    heights.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    for idx in 0..heights.len() {
        if remaining == 0 {
            break;
        }
        heights[idx].1 += 1;
        remaining -= 1;
    }

    heights
        .into_iter()
        .map(|(c, h, _)| (c, h))
        .collect()
}

fn render_pie_chart(frame: &mut ratatui::Frame, area: Rect, data: &ReportData) {
    let block = Block::default().title("Category Share").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if data.total_spend <= Decimal::ZERO {
        let empty = Paragraph::new("No expenses in this range")
            .alignment(Alignment::Center);
        frame.render_widget(empty, inner);
        return;
    }

    let mut slices = Vec::new();
    let total = data.total_spend.to_f64().unwrap_or(1.0).max(1.0);
    let mut start_angle = 0.0_f64;
    for (category, amount) in &data.category_totals {
        let value = amount.to_f64().unwrap_or(0.0);
        let ratio = value / total;
        let sweep = ratio * std::f64::consts::TAU; // TAU = 2 * PI
        slices.push((start_angle, start_angle + sweep, category.clone()));
        start_angle += sweep;
    }

    let canvas = Canvas::default()
        .x_bounds([-1.0, 1.0])
        .y_bounds([-1.0, 1.0])
        .paint(|ctx| {
            let step = 0.04;
            for (start, end, category) in &slices {
                let color = data
                    .category_colors
                    .get(category)
                    .copied()
                    .unwrap_or(Color::White);
                let mut points = Vec::new();
                let mut r = 0.0; // radius 0 center ... 1 edge
                while r <= 1.0 {
                    let mut angle = *start;
                    while angle <= *end {
                        points.push((r * angle.cos(), r * angle.sin()));
                        angle += 0.05;
                    }
                    r += step;
                }
                if !points.is_empty() {
                    ctx.draw(&Points { coords: &points, color });
                }
            }
        });

    frame.render_widget(canvas, inner);
}

fn render_category_table(frame: &mut ratatui::Frame, area: Rect, data: &ReportData) {
    let block = Block::default()
        .title("Category Spend")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if data.category_totals.is_empty() {
        let empty = Paragraph::new("No expenses in this range")
            .alignment(Alignment::Center);
        frame.render_widget(empty, inner);
        return;
    }

    let mut lines = Vec::new();
    let header = Line::from(vec![
        Span::styled("Category", Style::default().fg(Color::White).bold()),
        Span::raw("  "),
        Span::styled("Amount", Style::default().fg(Color::White).bold()),
    ]);
    lines.push(header);

    for (category, amount) in &data.category_totals {
        let color = data
            .category_colors
            .get(category)
            .copied()
            .unwrap_or(Color::White);
        let line = Line::from(vec![
            Span::styled(format!("{:15}", category), Style::default().fg(color)),
            Span::raw("  "),
            Span::styled(format!("{:>12}", amount), Style::default().fg(color)),
        ]);
        lines.push(line);
    }

    let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
    frame.render_widget(paragraph, inner);
}
