#[macro_use] extern crate log;

use std::env;
use std::net::SocketAddr;
use std::path::Path;

use axum::extract::Extension;
use axum::Router;
use axum::routing::{delete, get, post};
use dotenv::dotenv;
use sea_orm::{ConnectOptions, Database};
use sea_orm_migration::prelude::*;
use tower::ServiceBuilder;

mod entities;
mod dns;
mod util;
mod routes;
mod rpc;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Default to logging all info logs
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }

    pretty_env_logger::init();

    info!("██████╗ ██████╗ ██╗██████╗ ████████╗ ██████╗ ██████╗  ██████╗██╗  ██╗");
    info!("██╔══██╗██╔══██╗██║██╔══██╗╚══██╔══╝██╔═══██╗██╔══██╗██╔════╝██║  ██║");
    info!("██║  ██║██████╔╝██║██████╔╝   ██║   ██║   ██║██████╔╝██║     ███████║");
    info!("██║  ██║██╔══██╗██║██╔═══╝    ██║   ██║   ██║██╔══██╗██║     ██╔══██║");
    info!("██████╔╝██║  ██║██║██║        ██║   ╚██████╔╝██║  ██║╚██████╗██║  ██║");
    info!("╚═════╝ ╚═╝  ╚═╝╚═╝╚═╝        ╚═╝    ╚═════╝ ╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝");

    info!("._______ ._______  .______  _____._.______  ._______  .___    .___    ._______.______");
    info!(":_.  ___\\: .___  \\ :      \\ \\__ _:|: __   \\ : .___  \\ |   |   |   |   : .____/: __   \\ ");
    info!("|  : |/\\ | :   |  ||       |  |  :||  \\____|| :   |  ||   |   |   |   | : _/\\ |  \\____|");
    info!("|    /  \\|     :  ||   |   |  |   ||   :  \\ |     :  ||   |/\\ |   |/\\ |   /  \\|   :  \\ ");
    info!("|. _____/ \\_. ___/ |___|   |  |   ||   |___\\ \\_. ___/ |   /  \\|   /  \\|_.: __/|   |___\\");
    info!(" :/         :/         |___|  |___||___|       :/     |______/|______/   :/   |___|    ");
    info!(" :          :                                  :                                       ");

    info!("Version {}", VERSION);

    info!("Checking for supplemental files...");
    if !Path::new(&env::var("UAP_REGEXES").unwrap_or(String::from("./regexes.yaml"))).exists(){
        error!("Please download https://github.com/ua-parser/uap-core/blob/master/regexes.yaml either place it next to the executable or add it's path to env variable UAP_REGEXES! Halting start-up.");
        std::process::exit(1);
    }

    info!("Connecting to database...");
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set! Halting start-up.");
    let connection = Database::connect(
        ConnectOptions::new(database_url).sqlx_logging(false).to_owned())
        .await
        .expect("Failed to connect to the database! Halting start-up.");

    info!("Running migrations...");
    migration::Migrator::up(&connection, None)
        .await
        .expect("Failed to run migrations! Halting start-up.");

    info!("Starting web server...");
    let app = Router::new()
        // Auth
        .route("/user/register", post(routes::auth::register::register))
        .route("/user/login", post(routes::auth::login::login))
        .route("/user/logout", post(routes::auth::logout::logout))
        .route("/user/delete", delete(routes::auth::delete::delete))
        .route("/user/list_sessions", get(routes::auth::list_sessions::list_sessions))

        // Teams

        // Zones

        // Records

        // Proxies

        // RPC
        .route("/rpc", post(rpc::rpc))

        // Misc
        .route("/", get(routes::status::status))

        .layer(
        	ServiceBuilder::new()
        		.layer(Extension(connection))
        );
    
    let addr = env::var("LISTEN_ADDR")
        .unwrap_or("127.0.0.1:32204".to_string());
    
    let socket_addr: SocketAddr = addr.parse().expect("Failed to parse LISTEN_ADDR! Halting start-up.");

    let axum_builder = axum::Server::try_bind(&socket_addr);

    match axum_builder {
        Ok(_) => {
            info!("Driptorch Controller v{} is now listening on {}!", VERSION, socket_addr);

            axum_builder
                .expect("Passed builder match but still returned error? Halting start-up.")
                .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                .await
                .expect("Failed to bind to port! Halting start-up.");
        }
        Err(_) => {
            error!("Driptorch Controller v{} failed to bind to {}! Halting start-up.", VERSION, socket_addr);

            std::process::exit(1);
        }
    }
}
