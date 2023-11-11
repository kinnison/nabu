use clap::Parser;

#[derive(Debug, Parser)]
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
