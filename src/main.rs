use crate::tui::modes::App;
use ::tui::{backend::CrosstermBackend, Terminal};
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use serde::Serialize;
use std::{io, process};

use crate::tui::run_app;
use chip32_sim::{
    apf::parse_json,
    cpu::{FileLoadedState, HaltState, CPU},
};

mod tui;

/// Simple program to greet a person
#[derive(Parser, Debug)]
struct Args {
    /// The bin file to load
    #[clap(short, long, value_parser)]
    bin: String,

    /// The data slot file to load
    #[clap(short, long, value_parser)]
    data_json: Option<String>,

    /// Override the default data slot to load and put into R0. Per the docs this defaults to the ID of data slot 0 from --data-json
    #[clap(short = 's', long, value_parser)]
    data_slot: Option<u32>,

    /// Execute the simulation in JSON output mode
    #[clap(long)]
    json: bool,
}

#[derive(Serialize)]
struct JSONOutput {
    core: Option<usize>,
    logs: Vec<String>,
    file_state: FileLoadedState,
}

fn main() -> Result<(), io::Error> {
    let args = Args::parse();

    let slots = args
        .data_json
        .map_or(None, |json_path| Some(parse_json(&json_path)));

    let mut cpu = CPU::load_file(&args.bin, slots, args.data_slot)?;

    if args.json {
        let exit_code = execute_with_json(&mut cpu);

        println!("{}", build_json_output(&cpu));

        process::exit(exit_code as i32);
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen,)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::default();
    let res = run_app(&mut terminal, app, cpu);

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn execute_with_json(cpu: &mut CPU) -> usize {
    // No GUI, just run up to 1 million cycles
    for _ in 0..1_000_000 {
        cpu.step();

        match cpu.halt {
            HaltState::Success => return 0,
            HaltState::Failure => return 1,
            _ => {}
        }
    }

    // Did not terminate
    return 2;
}

fn build_json_output(cpu: &CPU) -> String {
    let output = JSONOutput {
        core: cpu.active_bitstream,
        logs: cpu.logs.clone(),
        file_state: cpu.file_state.loaded.clone(),
    };

    return serde_json::to_string(&output).expect("Couldn't generate JSON output");
}
