use clap::{
    builder::{styling::AnsiColor, Styles},
    Parser,
};

const CLI_STYLE: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default())
    .usage(AnsiColor::Green.on_default())
    .literal(AnsiColor::Green.on_default())
    .placeholder(AnsiColor::Green.on_default());

#[derive(Debug, Parser)]
#[command(name = "nabu", about = "A simple cargo HTTP registry")]
#[command(version)]
#[command(styles = CLI_STYLE)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Option<Cmd>,
}

#[derive(Debug, Default, Parser)]
pub enum Cmd {
    #[default]
    Serve,
    User(User),
}

#[derive(Debug, Parser)]
pub struct User {
    #[clap(subcommand)]
    pub command: UserCmd,
}

#[derive(Debug, Default, Parser)]
pub enum UserCmd {
    #[default]
    List,
    Create {
        name: String,
        #[clap(long = "admin")]
        admin: bool,
    },
    Tokens {
        name: String,
    },
    NewToken {
        name: String,
        title: String,
    },
    DeleteToken {
        name: String,
        token: String,
    },
}
