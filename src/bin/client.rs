//! Run this binary with `cargo run --bin client`
use clap::Parser;
use tokio::*;
use tokio::net::*;
use tokio::sync::mpsc;
use tui::{*, backend::*, layout::*, widgets::*};
use crossterm::{event::*, terminal::*, *};
use std::io;
use super_mega_chatroom::*;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    address: String,
    #[arg(short, long)]
    port: u16,
    #[arg(short, long)]
    name: String,
}

struct App {
    name: String,
    messages: Vec<Message>,
    input: String,
    tx: mpsc::Sender<Message>,
}


#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut stream = TcpStream::connect((args.address.as_str(), args.port)).await?;
    println!("Connected to server");
    let hi_msg = read_from_stream(&mut stream).await; // throw away hi message
    dbg!(hi_msg);
    write_to_stream(&mut stream, format!("{}\n", args.name)).await;
    let (tx, mut rx) = mpsc::channel(64);
    let mut app = App { name: args.name, messages: vec![], input: String::new(), tx };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {

        select! {
            msg = read_from_stream(&mut stream) => {
                let x = msg.split(':').collect::<Vec<&str>>();
                let name = x[0].trim().to_string();
                let msg = x[1].trim().to_string();
                app.messages.push(Message { name, msg, id: 0 })
            },
            Some(msg) = rx.recv() => {
                write_to_stream(&mut stream, format!("{}\n", msg.msg)).await;
            }
            a = handle_ui(&mut app, &mut terminal) => {
                if let AppEvent::Quit = a? {
                    break
                }
            }
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;   
    Ok(())
}

enum AppEvent {
    Quit,
    Continue,
}

async fn handle_ui<B: Backend>(app: &mut App, terminal: &mut Terminal<B>) -> Result<AppEvent> {
    terminal.draw(|f| ui(f, &app))?;

    if poll(Duration::from_millis(20))? {
        if let Event::Key(k) = read()? {
            if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('d') {
                return Ok(AppEvent::Quit);
            }
            match k.code {
                KeyCode::Char(mut c) => {
                    if k.modifiers.contains(KeyModifiers::SHIFT) { c = c.to_ascii_uppercase() }
                    app.input.push(c);
                },
                KeyCode::Backspace => { app.input.pop(); },
                KeyCode::Enter => { 
                    app.tx.send(Message { name: app.name.clone(), msg: app.input.clone(), id: 0 }).await.expect("Message failed to transmit");
                    app.messages.push(Message {name: app.name.clone(), msg: app.input.clone(), id: 0});
                    app.input.clear();
                }
                _ => {}
            }
        }
    }
    Ok(AppEvent::Continue)
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
   let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(80),
                Constraint::Percentage(20)
            ].as_ref()
        )
        .split(f.size());

   let block = Block::default().borders(Borders::ALL);
   let input_field = Paragraph::new(app.input.clone()).block(block.clone());

   let messages = app.messages.iter().map(|m| ListItem::new(format!("{}: {}\r\n", m.name, m.msg))).collect::<Vec<_>>();
   let list = List::new(messages).block(block);

   f.render_widget(list, chunks[0]);
   f.render_widget(input_field, chunks[1]);
}
