mod soulver;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use std::io::{self, Read};

#[derive(Parser)]
#[command(version, author, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Calculate {},

    /// Generate shell completions
    Completions {
        /// The shell to generate the completions for
        #[arg(value_enum)]
        shell: clap_complete_command::Shell,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Calculate {} => {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            let result = soulver::run_soulver_zipped(&input)?;
            println!("{result}");
        }
        Commands::Completions { shell } => {
            shell.generate(&mut Cli::command(), &mut std::io::stdout());
        }
    }
    Ok(())
}
