pub mod action;
pub mod app;
pub mod cli;
pub mod components;
pub mod config;
pub mod pages;
pub mod panes;
pub mod tui;
pub mod utils;

use clap::Parser;
use cli::Cli;
use color_eyre::eyre::Result;

use crate::{
  app::App,
  utils::{initialize_logging, initialize_panic_handler},
};

async fn tokio_main() -> Result<()> {
  initialize_logging()?;

  initialize_panic_handler()?;

  let args = Cli::parse();
  let mut app = App::new(args.openapi_path)?;
  app.run().await?;

  Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
  if let Err(e) = tokio_main().await {
    eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
    Err(e)
  } else {
    Ok(())
  }
}
