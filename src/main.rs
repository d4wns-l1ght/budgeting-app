use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use ratatui::{Terminal, crossterm::event, prelude::Backend};

use crate::{controller::Controller, model::Model, view::View};

mod controller;
mod model;
mod view;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	/// File to open
	filename: Option<String>,
}

fn main() {
	let args = Args::parse();

	let terminal = ratatui::init();
	let res = run_program(terminal, args);
	ratatui::restore();
	if let Err(e) = res {
		println!("{e:?}");
	}
}

fn run_program<B: Backend>(mut terminal: Terminal<B>, args: Args) -> Result<()> {
	let mut model = Model::new(args.filename);
	let mut view = View::new();
	let mut controller = Controller::new();

	loop {
		terminal.draw(|frame| view.render(frame, &model, &controller.state))?;

		if event::poll(Duration::from_millis(10))? {
			controller.handle_events(event::read()?, &mut model, &mut view)?;
		}

		if controller.state.exit {
			return Ok(());
		}
	}
}
