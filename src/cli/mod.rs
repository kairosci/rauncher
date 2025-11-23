use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rauncher")]
#[command(author, version, about = "Epic Games launcher for Linux written in Rust - GUI-first application", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Authenticate with Epic Games Store
    Auth {
        /// Logout instead of login
        #[arg(short, long)]
        logout: bool,
    },

    /// List games in your library
    List {
        /// Show installed games only
        #[arg(short, long)]
        installed: bool,
    },

    /// Install a game
    Install {
        /// App name of the game to install
        app_name: String,
    },

    /// Launch a game
    Launch {
        /// App name of the game to launch
        app_name: String,
    },

    /// Uninstall a game
    Uninstall {
        /// App name of the game to uninstall
        app_name: String,
    },

    /// Show information about a game
    Info {
        /// App name of the game
        app_name: String,
    },

    /// Show status and configuration
    Status,

    /// Check for game updates
    Update {
        /// App name of the game to check/update
        app_name: String,

        /// Only check for updates, don't install them
        #[arg(short, long)]
        check_only: bool,
    },

    /// Manage cloud saves
    CloudSave {
        /// App name of the game
        app_name: String,

        /// Download cloud saves
        #[arg(short, long)]
        download: bool,

        /// Upload local saves to cloud
        #[arg(short, long)]
        upload: bool,
    },

    /// Launch the GUI
    Gui,
}
