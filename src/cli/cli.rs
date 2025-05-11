use clap::{Parser, Subcommand};

/// Kona - A Claude Code clone for the command line
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Command to execute
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Increase output verbosity (-v, -vv, etc.)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Enable debug mode (shows sensitive information like API keys)
    #[arg(long)]
    pub debug: bool,

    /// Enable streaming responses
    #[arg(long, default_value_t = true)]
    pub streaming: bool,

    /// Disable streaming responses
    #[arg(long, default_value_t = false)]
    pub no_streaming: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Ask Claude a question and get a response
    Ask {
        /// The question to ask Claude
        #[arg(required = true)]
        query: String,
    },

    /// Initialize a new configuration file
    Init {
        /// Force overwrite of existing config
        #[arg(short, long)]
        force: bool,
    },

    /// Show current configuration
    Config,
}