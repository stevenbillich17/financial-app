use crate::db::repository;
use crate::models::transaction::{Transaction, TransactionType};
use chrono::NaiveDate;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::{Alignment, Color, Constraint, Direction, Layout, Rect, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState, Wrap},
};
use rusqlite::Connection;
use std::cmp::{max, min};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortOrder {
    DateDesc,
    DateAsc,
}

impl SortOrder {
    fn toggle(self) -> Self {
        match self {
            SortOrder::DateDesc => SortOrder::DateAsc,
            SortOrder::DateAsc => SortOrder::DateDesc,
        }
    }

    fn label(self) -> &'static str {
        match self {
            SortOrder::DateDesc => "date ↓",
            SortOrder::DateAsc => "date ↑",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    List,
    Details,
    Input(InputKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputKind {
    Category,
    DateRange,
}

struct BrowseState {
    mode: Mode,

    transactions: Vec<Transaction>,
    filtered_indices: Vec<usize>,

    table_state: TableState,

    filter_category: Option<String>,
    filter_type: Option<TransactionType>,
    filter_from: Option<NaiveDate>,
    filter_to: Option<NaiveDate>,

    sort_order: SortOrder,

    // Input modal
    input_buffer: String,
    input_error: Option<String>,

    // Details view
    details_tx: Option<Transaction>,

    // Cached per-draw
    last_page_size: usize,
}

impl BrowseState {
    fn new(transactions: Vec<Transaction>) -> Self {
        let mut state = Self {
            mode: Mode::List,
            transactions,
            filtered_indices: Vec::new(),
            table_state: TableState::default(),
            filter_category: None,
            filter_type: None,
            filter_from: None,
            filter_to: None,
            sort_order: SortOrder::DateDesc,
            input_buffer: String::new(),
            input_error: None,
            details_tx: None,
            last_page_size: 10,
        };
        state.recompute();
        state
    }

    fn selected_index(&self) -> Option<usize> {
        self.table_state.selected()
    }

    fn selected_transaction(&self) -> Option<&Transaction> {
        let selected = self.selected_index()?;
        let idx = *self.filtered_indices.get(selected)?;
        self.transactions.get(idx)
    }

    fn recompute(&mut self) {
        self.filtered_indices = (0..self.transactions.len())
            .filter(|&i| self.matches_filters(&self.transactions[i]))
            .collect();

        self.sort_filtered();

        if self.filtered_indices.is_empty() {
            self.table_state.select(None);
        } else {
            let new_selected = match self.table_state.selected() {
                Some(sel) => min(sel, self.filtered_indices.len().saturating_sub(1)),
                None => 0,
            };
            self.table_state.select(Some(new_selected));
        }
    }

    fn matches_filters(&self, tx: &Transaction) -> bool {
        if let Some(t) = self.filter_type {
            if tx.transaction_type != t {
                return false;
            }
        }

        if let Some(from) = self.filter_from {
            if tx.date < from {
                return false;
            }
        }
        if let Some(to) = self.filter_to {
            if tx.date > to {
                return false;
            }
        }

        if let Some(ref category) = self.filter_category {
            if tx.category.to_lowercase() != category.to_lowercase() {
                return false;
            }
        }

        true
    }

    fn sort_filtered(&mut self) {
        let txs = &self.transactions;
        match self.sort_order {
            SortOrder::DateDesc => {
                self.filtered_indices.sort_by(|&a, &b| {
                    let ta = &txs[a];
                    let tb = &txs[b];
                    tb.date
                        .cmp(&ta.date)
                        .then_with(|| tb.id.cmp(&ta.id))
                });
            }
            SortOrder::DateAsc => {
                self.filtered_indices.sort_by(|&a, &b| {
                    let ta = &txs[a];
                    let tb = &txs[b];
                    ta.date
                        .cmp(&tb.date)
                        .then_with(|| ta.id.cmp(&tb.id))
                });
            }
        }
    }

    fn move_selection(&mut self, delta: i32) {
        if self.filtered_indices.is_empty() {
            self.table_state.select(None);
            return;
        }

        let current = self.table_state.selected().unwrap_or(0) as i32;
        let max_index = self.filtered_indices.len().saturating_sub(1) as i32;
        let next = (current + delta).clamp(0, max_index) as usize;
        self.table_state.select(Some(next));
    }

    fn page_up(&mut self) {
        let page = max(1, self.last_page_size) as i32;
        self.move_selection(-page);
    }

    fn page_down(&mut self) {
        let page = max(1, self.last_page_size) as i32;
        self.move_selection(page);
    }

    fn refresh_from_db(&mut self, conn: &Connection) -> Result<(), String> {
        self.transactions = repository::get_all_transactions(conn)?;
        self.recompute();
        Ok(())
    }

    fn cycle_type_filter(&mut self) {
        self.filter_type = match self.filter_type {
            None => Some(TransactionType::Expense),
            Some(TransactionType::Expense) => Some(TransactionType::Income),
            Some(TransactionType::Income) => None,
        };
        self.recompute();
    }

    fn clear_filters(&mut self) {
        self.filter_category = None;
        self.filter_type = None;
        self.filter_from = None;
        self.filter_to = None;
        self.recompute();
    }

    fn open_details(&mut self) {
        self.details_tx = self.selected_transaction().cloned();
        self.mode = Mode::Details;
    }

    fn close_details(&mut self) {
        self.details_tx = None;
        self.mode = Mode::List;
    }

    fn start_input(&mut self, kind: InputKind) {
        self.input_buffer.clear();
        self.input_error = None;

        match kind {
            InputKind::Category => {
                if let Some(ref c) = self.filter_category {
                    self.input_buffer = c.clone();
                }
            }
            InputKind::DateRange => {
                let from = self
                    .filter_from
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_default();
                let to = self
                    .filter_to
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_default();
                if !from.is_empty() || !to.is_empty() {
                    self.input_buffer = format!("{}..{}", from, to);
                }
            }
        }

        self.mode = Mode::Input(kind);
    }

    fn cancel_input(&mut self) {
        self.input_error = None;
        self.mode = Mode::List;
    }

    fn commit_input(&mut self, kind: InputKind) {
        let raw = self.input_buffer.trim();
        match kind {
            InputKind::Category => {
                if raw.is_empty() {
                    self.filter_category = None;
                } else {
                    self.filter_category = Some(raw.to_string());
                }
                self.mode = Mode::List;
                self.recompute();
            }
            InputKind::DateRange => {
                if raw.is_empty() {
                    self.filter_from = None;
                    self.filter_to = None;
                    self.mode = Mode::List;
                    self.recompute();
                    return;
                }

                match parse_date_range(raw) {
                    Ok((from, to)) => {
                        self.filter_from = from;
                        self.filter_to = to;
                        self.input_error = None;
                        self.mode = Mode::List;
                        self.recompute();
                    }
                    Err(e) => {
                        self.input_error = Some(e);
                    }
                }
            }
        }
    }
}

pub fn run_browse(conn: &Connection) -> Result<(), String> {
    enable_raw_mode().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| format!("Failed to enter alternate screen: {}", e))?;

    let result = (|| {
        let backend = ratatui::backend::CrosstermBackend::new(stdout);
        let mut terminal = ratatui::Terminal::new(backend)
            .map_err(|e| format!("Failed to initialize terminal: {}", e))?;

        let initial = repository::get_all_transactions(conn)?;
        let mut state = BrowseState::new(initial);

        loop {
            terminal
                .draw(|frame| {
                    let size = frame.area();
                    let layout = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3),
                            Constraint::Min(5),
                            Constraint::Length(2),
                        ])
                        .split(size);

                    render_header(frame, layout[0], &state);
                    render_table(frame, layout[1], &mut state);
                    render_footer(frame, layout[2], &state);

                    if let Mode::Input(kind) = state.mode {
                        render_input_modal(frame, size, &state, kind);
                    }

                    if state.mode == Mode::Details {
                        render_details_modal(frame, size, &state);
                    }
                })
                .map_err(|e| format!("Failed to draw terminal UI: {}", e))?;

            if event::poll(std::time::Duration::from_millis(200))
                .map_err(|e| format!("Failed to poll input: {}", e))?
            {
                let event = event::read().map_err(|e| format!("Failed to read input: {}", e))?;
                match event {
                    Event::Key(key) => {
                        if handle_key(conn, &mut state, key)? {
                            break;
                        }
                    }
                    Event::Resize(_, _) => {}
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

fn handle_key(conn: &Connection, state: &mut BrowseState, key: KeyEvent) -> Result<bool, String> {
    // Many terminals emit both a Press and a Release event. Only act on Press/Repeat.
    if key.kind == KeyEventKind::Release {
        return Ok(false);
    }

    // Global quit in list mode
    if state.mode == Mode::List {
        if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
            return Ok(true);
        }
    }

    match state.mode {
        Mode::List => match key.code {
            KeyCode::Up => state.move_selection(-1),
            KeyCode::Down => state.move_selection(1),
            KeyCode::PageUp => state.page_up(),
            KeyCode::PageDown => state.page_down(),
            KeyCode::Home => state.table_state.select(Some(0)),
            KeyCode::End => {
                if !state.filtered_indices.is_empty() {
                    state
                        .table_state
                        .select(Some(state.filtered_indices.len().saturating_sub(1)));
                }
            }
            KeyCode::Enter => state.open_details(),
            KeyCode::Char('r') => state.refresh_from_db(conn)?,
            KeyCode::Char('c') => state.start_input(InputKind::Category),
            KeyCode::Char('d') => state.start_input(InputKind::DateRange),
            KeyCode::Char('t') => state.cycle_type_filter(),
            KeyCode::Char('s') => {
                state.sort_order = state.sort_order.toggle();
                state.recompute();
            }
            KeyCode::Char('x') => state.clear_filters(),
            _ => {}
        },
        Mode::Details => match key.code {
            KeyCode::Esc => state.close_details(),
            KeyCode::Char('q') => state.close_details(),
            KeyCode::Char('b') => state.close_details(),
            _ => {}
        },
        Mode::Input(kind) => {
            // Allow Ctrl+C / Ctrl+Q to cancel
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('q'))
            {
                state.cancel_input();
                return Ok(false);
            }

            match key.code {
                KeyCode::Esc => state.cancel_input(),
                KeyCode::Enter => state.commit_input(kind),
                KeyCode::Backspace => {
                    state.input_buffer.pop();
                }
                KeyCode::Char(ch) => {
                    state.input_buffer.push(ch);
                }
                _ => {}
            }
        }
    }

    Ok(false)
}

fn render_header(frame: &mut ratatui::Frame, area: Rect, state: &BrowseState) {
    let category = state
        .filter_category
        .as_deref()
        .unwrap_or("(any)")
        .to_string();

    let ttype = match state.filter_type {
        None => "(any)",
        Some(TransactionType::Income) => "income",
        Some(TransactionType::Expense) => "expense",
    };

    let from = state
        .filter_from
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "(any)".to_string());
    let to = state
        .filter_to
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "(any)".to_string());

    let line = Line::from(vec![
        Span::styled("FINO Browse", Style::default().fg(Color::Cyan).bold()),
        Span::raw("  "),
        Span::styled(format!("Sort: {}", state.sort_order.label()), Style::default().fg(Color::White)),
        Span::raw("  |  "),
        Span::raw(format!("Category: {}", category)),
        Span::raw("  |  "),
        Span::raw(format!("Type: {}", ttype)),
        Span::raw("  |  "),
        Span::raw(format!("Date: {}..{}", from, to)),
        Span::raw("  |  "),
        Span::raw(format!("Rows: {}", state.filtered_indices.len())),
    ]);

    let block = Block::default().borders(Borders::ALL);
    let paragraph = Paragraph::new(line).block(block).alignment(Alignment::Left);
    frame.render_widget(paragraph, area);
}

fn render_footer(frame: &mut ratatui::Frame, area: Rect, state: &BrowseState) {
    let hint = match state.mode {
        Mode::List => "↑/↓ move  PgUp/PgDn page  Enter details  c category  d dates  t type  s sort  r refresh  x clear  q/Esc exit",
        Mode::Details => "Esc/q/b back",
        Mode::Input(_) => "Type, Enter apply, Esc cancel",
    };

    let block = Block::default().borders(Borders::ALL);
    frame.render_widget(
        Paragraph::new(hint)
            .block(block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_table(frame: &mut ratatui::Frame, area: Rect, state: &mut BrowseState) {
    let block = Block::default().title("Transactions").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let header = Row::new([
        Cell::from("Date").style(Style::default().bold()),
        Cell::from("Description").style(Style::default().bold()),
        Cell::from("Amount").style(Style::default().bold()),
        Cell::from("Type").style(Style::default().bold()),
        Cell::from("Category").style(Style::default().bold()),
        Cell::from("Id").style(Style::default().bold()),
    ])
    .style(Style::default().fg(Color::White));

    let rows = state
        .filtered_indices
        .iter()
        .map(|&idx| &state.transactions[idx])
        .map(|tx| {
            let date = tx.date.format("%Y-%m-%d").to_string();
            let mut desc = tx.description.clone();
            if desc.len() > 42 {
                desc.truncate(39);
                desc.push_str("...");
            }
            let amount = tx.amount.to_string();
            let ttype = match tx.transaction_type {
                TransactionType::Income => "income",
                TransactionType::Expense => "expense",
            };
            let mut id_short = tx.id.clone();
            if id_short.len() > 8 {
                id_short.truncate(8);
            }

            Row::new([
                Cell::from(date),
                Cell::from(desc),
                Cell::from(amount),
                Cell::from(ttype),
                Cell::from(tx.category.clone()),
                Cell::from(id_short),
            ])
        });

    // Estimate a page size based on the table height.
    // Leave room for the header row.
    state.last_page_size = inner.height.saturating_sub(2) as usize;
    if state.last_page_size == 0 {
        state.last_page_size = 1;
    }

    let widths = [
        Constraint::Length(10),
        Constraint::Percentage(40),
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Length(14),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White).bold())
        .highlight_symbol("➤ ")
        .column_spacing(1);

    frame.render_stateful_widget(table, inner, &mut state.table_state);

    if state.filtered_indices.is_empty() {
        let empty = Paragraph::new("No transactions match the current filters")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, inner);
    }
}

fn render_input_modal(frame: &mut ratatui::Frame, area: Rect, state: &BrowseState, kind: InputKind) {
    let popup_area = centered_rect(80, 30, area);
    frame.render_widget(Clear, popup_area);

    let title = match kind {
        InputKind::Category => "Filter Category",
        InputKind::DateRange => "Filter Date Range",
    };

    let help = match kind {
        InputKind::Category => "Enter category name (empty clears)",
        InputKind::DateRange => "Enter range like 2025-01-01..2025-01-31 (empty clears)",
    };

    let mut lines = vec![
        Line::from(vec![Span::styled(title, Style::default().bold())]),
        Line::from(help),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("> {}", state.input_buffer),
            Style::default().fg(Color::Yellow),
        )]),
    ];

    if let Some(ref err) = state.input_error {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            err,
            Style::default().fg(Color::Red),
        )]));
    }

    let block = Block::default().borders(Borders::ALL).title("Input");
    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, popup_area);
}

fn render_details_modal(frame: &mut ratatui::Frame, area: Rect, state: &BrowseState) {
    let popup_area = centered_rect(90, 60, area);
    frame.render_widget(Clear, popup_area);

    let tx = match state.details_tx.as_ref() {
        Some(tx) => tx,
        None => {
            frame.render_widget(
                Paragraph::new("No selection")
                    .block(Block::default().borders(Borders::ALL).title("Details"))
                    .alignment(Alignment::Center),
                popup_area,
            );
            return;
        }
    };

    let ttype = match tx.transaction_type {
        TransactionType::Income => "income",
        TransactionType::Expense => "expense",
    };

    let lines = vec![
        Line::from(vec![Span::styled(
            "Transaction Details",
            Style::default().fg(Color::Cyan).bold(),
        )]),
        Line::from(""),
        Line::from(format!("Id: {}", tx.id)),
        Line::from(format!("Date: {}", tx.date.format("%Y-%m-%d"))),
        Line::from(format!("Type: {}", ttype)),
        Line::from(format!("Category: {}", tx.category)),
        Line::from(format!("Amount: {}", tx.amount)),
        Line::from(""),
        Line::from("Description:"),
        Line::from(format!("{}", tx.description)),
        Line::from(""),
        Line::from(Span::styled(
            "Esc/q/b to go back",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let block = Block::default().borders(Borders::ALL).title("Details");
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false }),
        popup_area,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn parse_date_range(input: &str) -> Result<(Option<NaiveDate>, Option<NaiveDate>), String> {
    let s = input.trim();

    // Supported formats:
    //  - YYYY-MM-DD..YYYY-MM-DD
    //  - YYYY-MM-DD-YYYY-MM-DD
    //  - YYYY-MM-DD,YYYY-MM-DD
    let (left, right) = if let Some((a, b)) = s.split_once("..") {
        (a.trim(), b.trim())
    } else if let Some((a, b)) = s.split_once(',') {
        (a.trim(), b.trim())
    } else if let Some((a, b)) = split_once_dash_range(s) {
        (a.trim(), b.trim())
    } else {
        return Err("Invalid date range. Use YYYY-MM-DD..YYYY-MM-DD".to_string());
    };

    let from = if left.is_empty() {
        None
    } else {
        Some(parse_iso_date(left)?)
    };

    let to = if right.is_empty() {
        None
    } else {
        Some(parse_iso_date(right)?)
    };

    if let (Some(f), Some(t)) = (from, to) {
        if f > t {
            return Err("Invalid range: start date must be <= end date".to_string());
        }
    }

    Ok((from, to))
}

fn parse_iso_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d")
        .map_err(|_| format!("Invalid date '{}'. Use YYYY-MM-DD.", s.trim()))
}

fn split_once_dash_range(s: &str) -> Option<(&str, &str)> {
    // Try to split on the last '-' that separates two ISO dates.
    // Example: 2025-01-01-2025-01-31
    let bytes = s.as_bytes();
    for i in (0..bytes.len()).rev() {
        if bytes[i] == b'-' {
            let (a, b) = s.split_at(i);
            let b = &b[1..];
            // Heuristic: both sides should look like ISO date lengths.
            if a.trim().len() >= 10 && b.trim().len() >= 10 {
                return Some((a, b));
            }
        }
    }
    None
}
