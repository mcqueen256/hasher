use std::io;
use std::error::Error;
// use termion::{
//     event::Key,
//     input::MouseTerminal,
//     raw::IntoRawMode,
//     screen::AlternateScreen};
use crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
use tui::{
    backend::Backend,
    // backend::TermionBackend,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Gauge},
    Terminal,
    Frame,
};

use crate::util::event::Event;
use crate::util::event::Events;

use std::sync::Arc;
use std::sync::Mutex;
use crate::application::{
    App,
    Application,
    CurrentJob,
    ThreadState,
};

use crate::log::LogMessage;

pub fn main_loop(app: Arc<Mutex<Application>>) -> Result<(), Box<dyn Error>> {

    enable_raw_mode()?;

    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // let stdout = io::stdout().into_raw_mode()?;
    // let stdout = MouseTerminal::from(stdout);
    // let stdout = AlternateScreen::from(stdout);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    loop {
        let active_thread_count = {
            let app = app.lock().unwrap();
            (*app).threads.len()
        };

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(6),
                    Constraint::Length(active_thread_count as u16 + 2),
                    Constraint::Percentage(50),
                ].as_ref())
                .split(f.size());
            let app = App::from(&app);
            let stats = extract_statistics(App::clone(&app));
            let thread_statuses = extract_thread_statuses(App::clone(&app));
            let messages = extract_log_messages(App::clone(&app));

            draw_app_stats_window(f, chunks[0], stats);
            draw_gauge_window(f, chunks[1], thread_statuses);
            draw_log_window(f, chunks[2], messages);
        })?;



        match events.next()? {
            Event::Input(input) => match input {
                KeyCode::Char('q') => {
                    let mut app = app.lock().unwrap();
                    (*app).quitting = true;
                }
                KeyCode::Up => {
                    let mut app = app.lock().unwrap();
                    (*app).expected_thread_count += 1;
                }
                KeyCode::Down => {
                    let mut app = app.lock().unwrap();
                    if (*app).expected_thread_count != 0 {
                        (*app).expected_thread_count -= 1;
                    }
                }
                _ => {}
            },
            _ => (),
        }

        // Check if the ui can end.
        {
            let app = app.lock().unwrap();
            if (*app).quitting && (*app).threads_cleaned_up {
                break;
            }
        }
    }


    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    Ok(())
}

struct Statistics {
    hash_rate: f64,
    completed_job: usize,
    user_shares: usize,
    pool_shares: usize,
    best_bit_length: u8,
    user_hash_rate: f64,
    student_number: String,
    name: String,
    thread_count: u8,
    quitting: bool,
}

fn draw_app_stats_window<B: Backend>(f: &mut Frame<B>, area: Rect, stats: Statistics) {
    if stats.quitting {
        let info_line_items = vec![ListItem::new(vec![
            Spans::from(format!("  Shutting down... Please wait."))
        ])];
        let items = List::new(info_line_items)
            .block(Block::default().borders(Borders::ALL).title(" Hasher 0.2 - Info "));
        f.render_widget(items, area);
        return;
    }
    let info_line_items = vec![
        ListItem::new(vec![
            Spans::from(format!("  (Q) - Quit, (up) - Inc threads, (down) - Dec threads"))
        ]),
        ListItem::new(vec![
            Spans::from(
                format!(
                    "  Rate: {:.02} MH/s, Completed jobs: {}, Shares: {}/{}",
                    stats.hash_rate,
                    stats.completed_job,
                    stats.user_shares,
                    stats.pool_shares
                )
            )
        ]),
        ListItem::new(vec![
            Spans::from(
                format!(
                    "  Pool Best Zero Length: {}/48, Your total hashrate: {:.02} MH/s",
                    stats.best_bit_length,
                    stats.user_hash_rate,
                )
            )
        ]),
        ListItem::new(vec![
            Spans::from(
                format!(
                    "  Threads: {}, Student number: {}, client: {}",
                    stats.thread_count,
                    stats.student_number,
                    stats.name,
                )
            )
        ]),
    ];
    let items = List::new(info_line_items)
        .block(Block::default().borders(Borders::ALL).title(" Hasher 0.2 - Info "));
    f.render_widget(items, area);
}


struct ThreadStatus {
    current_job: Option<CurrentJob>,
    state: ThreadState,
}


fn draw_gauge_window<B: Backend>(f: &mut Frame<B>, area: Rect, thread_statuses: Vec<ThreadStatus>) {

    let constraints = thread_statuses.iter().map(|_| Constraint::Length(1)).collect::<Vec<Constraint>>();

    let block = Block::default().borders(tui::widgets::Borders::ALL).title(" Threads ");
    f.render_widget(block, area);

    let chunks = Layout::default()
        .constraints(constraints)
        .margin(1)
        .split(area);

    for (i, ts) in thread_statuses.iter().enumerate() {
        // let progress = if let Some(cj) = ts.current_job { cj.progress } else { 0 };
        let job_info = if let Some(current_job) = &ts.current_job {
            let progress = current_job.progress as f64 / current_job.size as f64;
            (
                format!("Thread {} : Job {} : {:.2}% : Solutions {} : Size {: <9}",
                    i + 1,
                    current_job.job_number,
                    progress * 100.0,
                    current_job.solutions,
                    current_job.size,
                ),
                progress
            )
        } else {
            (String::from("Awaiting job"), 0.0)
        };
        
        let label = format!("{} : {:?}", job_info.0, ts.state);
        let gauge = Gauge::default()
            // .block(Block::default().title("Gauge:"))
            .gauge_style(
                get_gauge_style(i)
                .bg(Color::Black)
                .add_modifier(Modifier::ITALIC | Modifier::BOLD),
            )
            .label(label)
            .ratio(job_info.1);
        f.render_widget(gauge, chunks[i]);
    }
}


fn draw_log_window<B: Backend>(f: &mut Frame<B>, area: Rect, items: Vec<ListItem>) {
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Log "));
    f.render_widget(items, area);
}

fn get_gauge_style(i: usize) -> Style {
    let style_colours: Vec<Style> = vec![
        Style::default().fg(Color::Magenta),
        Style::default().fg(Color::Cyan),
        Style::default().fg(Color::Blue),
        Style::default().fg(Color::Green),
        Style::default().fg(Color::Yellow),
        Style::default().fg(Color::Red),
    ];
    style_colours[i%style_colours.len()].clone()
}


fn extract_thread_statuses(mut app: App) -> Vec<ThreadStatus> {
    app.lock(|app| {
        app.threads
        .iter()
        .map(|mt| {
            ThreadStatus {
                current_job: *(mt.current_job.lock().unwrap()),
                state: *(mt.state.lock().unwrap()),
            }
        })
        .collect()
    })
}


fn extract_statistics(mut app: App) -> Statistics {
    app.lock(|app| {
        let mut completed_job = 0;
        let mut user_shares = 0;
        let mut pool_shares = 0;
        let mut best_bit_length = 0;
        let mut user_hash_rate = 0.0;

        if let Some(pool_status) = &app.pool_status {
            completed_job = pool_status.completed_jobs as usize;
            user_shares = pool_status.user_total_shares;
            pool_shares = pool_status.pool_total_shares;
            best_bit_length = pool_status.pool_best_zero_length;
            user_hash_rate = pool_status.user_total_hash_rate;
        }; 
        Statistics {
            hash_rate: app.total_hashrate() / 1_000_000.0,
            completed_job,
            user_shares,
            pool_shares,
            best_bit_length,
            user_hash_rate: user_hash_rate / 1_000_000.0,
            student_number: String::from(&app.student_number),
            name: String::from(&app.name),
            thread_count: app.expected_thread_count as u8,
            quitting: app.quitting,
        }
    })
}


fn extract_log_messages<'a>(mut app: App) -> Vec<ListItem<'a>> {
    let logs = app.lock(|app| {
        let messages = app.log.get();
        messages.clone()
    });
    logs.into_iter()
        .rev()
        .map(|msg| {
            match msg {
                LogMessage::Solution{hash, nounce, leading_zero_bit_length} => {
                    // Catergories message
                    let mut line = vec![
                        Span::raw(" [ "),
                        Span::styled("OK", Style::default().fg(Color::Green)),
                        Span::raw(" ]   "),
                    ];

                    // Add length
                    let zn = (leading_zero_bit_length as f64 / 4.0).ceil() as usize;
                    let length_str = format!("{:<5}", &leading_zero_bit_length);
                    line.push(Span::raw(length_str));

                    // Add sha256
                    let zeros = String::from(&hash[..zn]);
                    let zeros = Span::styled(zeros, Style::default().fg(Color::Cyan));
                    let rest = String::from(&hash[zn..]);
                    let rest  = Span::raw(rest);
                    line.push(zeros);
                    line.push(rest);
                    line.push(Span::raw("   "));

                    // Add nounce
                    line.push(Span::raw(nounce));

                    // Add to queue.
                    let lines = vec![Spans::from(line)];
                    ListItem::new(lines).style(Style::default())
                }
                LogMessage::Error(message) => {
                    // Catergories message
                    let mut line = vec![
                        Span::raw(" [ "),
                        Span::styled("ERR", Style::default().fg(Color::Red)),
                        Span::raw(" ]  "),
                    ];
                    
                    line.push(Span::raw(message));

                    // Add to queue.
                    let lines = vec![Spans::from(line)];
                    ListItem::new(lines).style(Style::default())
                }
                LogMessage::Info(message) => {
                    // Catergories message
                    let mut line = vec![
                        Span::raw(" [ "),
                        Span::styled("INF", Style::default().fg(Color::LightBlue)),
                        Span::raw(" ]  "),
                    ];
                    
                    line.push(Span::raw(message));

                    // Add to queue.
                    let lines = vec![Spans::from(line)];
                    ListItem::new(lines).style(Style::default())
                }
            }
        })
        .collect()
}