use std::{io::IsTerminal, net::SocketAddr};

use axum::{extract::DefaultBodyLimit, Router};
use clap::Parser;
use database::{apply_migrations, create_pool, AsyncPgConnection, Pool};
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::{info, warn, Level};
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, EnvFilter};

mod api;
mod auth;
mod cli;
mod configuration;
mod index;
mod state;

use cli::Cli;
use configuration::Configuration;
use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(
                    if std::io::stdout().is_terminal() {
                        LevelFilter::INFO
                    } else {
                        LevelFilter::ERROR
                    }
                    .into(),
                )
                .with_env_var("NABU_LOG")
                .from_env_lossy(),
        )
        .init();

    if dotenv::dotenv().is_ok() {
        info!("Loaded configuration from .env file");
    } else {
        warn!("No .env file detected, configuration only from process environment");
    }

    let config = Configuration::load().expect("Unable to load config from environment:");

    let cli = Cli::parse();

    info!("Applying any pending migrations...");

    apply_migrations(config.database_url().as_str()).expect("Unable to apply migrations:");

    info!("Preparing database connection pool...");

    let pool = create_pool(config.database_url().as_str())
        .await
        .expect("Unable to estable database pool");

    match cli.command {
        None | Some(cli::Cmd::Serve) => serve(config, pool).await,
        Some(cli::Cmd::User(usercmd)) => user(pool, usercmd).await,
    }
}

async fn serve(config: Configuration, pool: Pool) {
    let port = config.port();
    let state = AppState::new(config, pool);
    let app = Router::new()
        .nest("/crates", index::router(&state))
        .nest("/api", api::router(&state))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
        .layer(DefaultBodyLimit::max(20 * 1024 * 1024));
    let app = app.with_state(state);
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    info!("Starting server on {addr}...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Failure when running axum");
}

async fn user(pool: Pool, cmd: cli::User) {
    let mut conn = pool.get().await.expect("Could not get DB connection");
    match cmd.command {
        cli::UserCmd::List => listusers(&mut conn).await,
        cli::UserCmd::Create { name, admin } => createuser(&mut conn, &name, admin).await,
        cli::UserCmd::Tokens { name } => listtokens(&mut conn, &name).await,
        cli::UserCmd::NewToken { name, title } => newtoken(&mut conn, &name, &title).await,
        cli::UserCmd::DeleteToken { name, token } => deletetoken(&mut conn, &name, &token).await,
    }
}

async fn listusers(conn: &mut AsyncPgConnection) {
    let users = database::models::Identity::all(conn)
        .await
        .expect("Unable to extract user list from database");

    for user in users {
        println!(
            "{} is {}, and has {} tokens",
            user.name,
            if user.admin {
                "an admin"
            } else {
                "a normal user"
            },
            user.tokens(conn)
                .await
                .expect("Unable to list user tokens")
                .len(),
        );
    }
}

async fn createuser(conn: &mut AsyncPgConnection, name: &str, admin: bool) {
    let user = database::models::Identity::new(conn, name, admin)
        .await
        .expect("Unable to create user");
    println!("User {} created.", user.name);
}

async fn listtokens(conn: &mut AsyncPgConnection, name: &str) {
    let user = database::models::Identity::by_name(conn, name)
        .await
        .expect("Unable to query for user")
        .expect("Unable to find user");
    let tokens = user
        .tokens(conn)
        .await
        .expect("Unable to retrieve token list");
    println!("User {} has {} tokens.", user.name, tokens.len());
    for token in tokens {
        println!("{} - {}", token.content, token.title);
    }
}

async fn newtoken(conn: &mut AsyncPgConnection, name: &str, title: &str) {
    let user = database::models::Identity::by_name(conn, name)
        .await
        .expect("Unable to query for user")
        .expect("Unable to find user");
    let token = user
        .new_token(conn, title)
        .await
        .expect("Unable to create new token");
    println!("{}", token.content);
}

async fn deletetoken(conn: &mut AsyncPgConnection, name: &str, token: &str) {
    let user = database::models::Identity::by_name(conn, name)
        .await
        .expect("Unable to query for user")
        .expect("Unable to find user");
    if user
        .delete_token(conn, token)
        .await
        .expect("Unable to delete token")
        > 0
    {
        println!("Token removed");
    } else {
        println!("Token not found");
    }
}
