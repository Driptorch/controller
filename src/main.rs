#[macro_use] extern crate log;

use std::{env, fs};
use std::net::SocketAddr;
use std::path::Path;

use axum::extract::Extension;
use axum::Router;
use axum::routing::{delete, get, post};
use dotenv::dotenv;
use lapin::ConnectionProperties;
use picky::key::PrivateKey;
use picky::x509::Cert;
use sea_orm::{ActiveValue, ColumnTrait, ConnectOptions, Database, EntityTrait, QueryFilter};
use sea_orm_migration::prelude::*;
use tower::ServiceBuilder;
use ulid::Ulid;
use crate::cert::encrypt_priv_key;
use crate::cert::generate::{generate_inter_cert, generate_root_cert};
use crate::cert::generate::InterTarget::{CLIENT, PROXY};
use crate::cert::Types::{CLIENTINTER, PROXYINTER, ROOT};
use crate::certificate::Model;
use crate::entities::certificate;

mod entities;
mod dns;
mod util;
mod routes;
mod rpc;
mod cert;

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
    if !Path::new(&env::var("RSA_KEY").expect("RSA_KEY must be set! Halting start-up.")).exists() {
        error!("Please generate an RSA private key for creating certificates!")
    }
    if !Path::new(&env::var("XCC20_KEY").expect("XCC20_KEY must be set! Halting start-up.")).exists() {
        error!("Please generate a 32 bits of randomness to encrypt private keys!")
    }

    let root_rsa_key = PrivateKey::from_pem_str(
        &*fs::read_to_string(
            Path::new(&env::var("RSA_KEY").expect("RSA_KEY must be set! Halting start-up."))
        ).expect("Failed to load the root RSA key! Halting start-up.")
    ).expect("Failed to load the root RSA key! Halting start-up.");

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

    info!("Checking for root cert...");
    let root_cert_model: Option<Model> = certificate::Entity::find()
        .filter(certificate::Column::CertType.eq(cert::Types::ROOT.to_string()))
        .one(&connection)
        .await
        .expect("Failed to retrieve the root cert from the database! Halting start-up.");

    let root_cert: Cert;

    match root_cert_model {
        None => {
            info!("Generating new root cert...");

            let new_root_cert = generate_root_cert(&root_rsa_key)
                .await
                .expect("Failed to generate the root cert from private key! Halting start-up.");

            root_cert = new_root_cert.clone();

            let root_insert = certificate::Entity::insert(certificate::ActiveModel {
                id: ActiveValue::Set(Ulid::new().to_string()),
                data: ActiveValue::Set(
                    new_root_cert.to_der().expect("Failed to convert cert into der!")
                ),
                key: ActiveValue::set(vec![145, 66, 62, 61, 56, 156, 145, 164]),
                nonce: ActiveValue::set(vec![145, 71, 62, 66, 56, 156, 145, 164]),
                cert_type: ActiveValue::Set(ROOT.to_string())
            })
                .exec(&connection)
                .await
                .expect("Failed to insert new root cert into database! Halting start-up.");

            info!("Generated new root cert: {}", root_insert.last_insert_id);
        }
        Some(root_cert_model) => {
            info!("Found root cert: {}", root_cert_model.id);

            root_cert = Cert::from_der(&root_cert_model.data)
                .expect("Failed to decode root cert!");
        }
    }

    info!("Checking for proxy intermediate cert...");
    let proxy_inter_model: Option<Model> = certificate::Entity::find()
        .filter(certificate::Column::CertType.eq(PROXYINTER.to_string()))
        .one(&connection)
        .await
        .expect("Failed to retrieve the proxy intermediate cert from the database! Halting start-up.");

    let proxy_inter_cert: Cert;

    match proxy_inter_model {
        None => {
            info!("Generating proxy intermediate cert...");

            // Generate 4096 bit RSA private key
            let priv_key = PrivateKey::generate_rsa(4096)
                .expect("Failed to generate a key");

            let encrypted_priv_key = encrypt_priv_key(priv_key
                .clone()
                .to_pkcs8()
                .expect("Failed to convert generated private key to pkcs8!")
            ).await;

            let new_proxy_inter_cert = generate_inter_cert(&priv_key, PROXY, (&root_cert, &root_rsa_key))
                .await
                .expect("Failed to generate new proxy intermediate cert");

            proxy_inter_cert = new_proxy_inter_cert.clone();

            let proxy_inter_insert = certificate::Entity::insert(certificate::ActiveModel {
                id: ActiveValue::Set(Ulid::new().to_string()),
                data: ActiveValue::Set(
                    new_proxy_inter_cert.to_der().expect("Failed to convert cert into der!")
                ),
                key: ActiveValue::set(encrypted_priv_key.1),
                nonce: ActiveValue::set(encrypted_priv_key.0),
                cert_type: ActiveValue::Set(PROXYINTER.to_string())
            })
                .exec(&connection)
                .await
                .expect("Failed to insert new proxy intermediate cert into database! Halting start-up.");

            info!("Generated new proxy intermediate cert: {}", proxy_inter_insert.last_insert_id)
        }
        Some(proxy_inter_model) => {
            info!("Found proxy intermediate cert: {}", proxy_inter_model.id);

            proxy_inter_cert = Cert::from_der(&proxy_inter_model.data)
                .expect("Failed to decode proxy intermediate cert!");
        }
    }

    info!("Checking for client intermediate cert...");
    let client_inter_model: Option<Model> = certificate::Entity::find()
        .filter(certificate::Column::CertType.eq(CLIENTINTER.to_string()))
        .one(&connection)
        .await
        .expect("Failed to retrieve the client intermediate cert from the database! Halting start-up.");

    let client_inter_cert: Cert;

    match client_inter_model {
        None => {
            info!("Generating client intermediate cert...");

            // Generate 4096 bit RSA private key
            let priv_key = PrivateKey::generate_rsa(4096)
                .expect("Failed to generate a key");

            let encrypted_priv_key = encrypt_priv_key(priv_key
                .clone()
                .to_pkcs8()
                .expect("Failed to convert generated private key to pkcs8!")
            ).await;

            let new_client_inter_cert = generate_inter_cert(&priv_key, CLIENT, (&root_cert, &root_rsa_key))
                .await
                .expect("Failed to generate new client intermediate cert");

            client_inter_cert = new_client_inter_cert.clone();

            let client_inter_insert = certificate::Entity::insert(certificate::ActiveModel {
                id: ActiveValue::Set(Ulid::new().to_string()),
                data: ActiveValue::Set(
                    new_client_inter_cert.to_der().expect("Failed to convert cert into der!")
                ),
                key: ActiveValue::set(encrypted_priv_key.1),
                nonce: ActiveValue::set(encrypted_priv_key.0),
                cert_type: ActiveValue::Set(CLIENTINTER.to_string())
            })
                .exec(&connection)
                .await
                .expect("Failed to insert new client intermediate cert into database! Halting start-up.");

            info!("Generated new client intermediate cert: {}", client_inter_insert.last_insert_id)
        }
        Some(client_inter_model) => {
            info!("Found client intermediate cert: {}", client_inter_model.id);

            client_inter_cert = Cert::from_der(&client_inter_model.data)
                .expect("Failed to decode client intermediate cert!");
        }
    }

    info!("Connecting to message broker...");
    let amqp_addr = env::var("AMQP_ADDR")
        .expect("AMQP_ADDR mut be set! Halting start-up.");
    let amqp_connection = lapin::Connection::connect(&amqp_addr, ConnectionProperties::default())
        .await
        .expect("Failed to connect to the message broker! Halting start-up.");
    let amqp_channel = amqp_connection.create_channel()
        .await
        .expect("Failed to create a message broker channel! Halting start-up.");

    info!("Starting web server...");
    let app = Router::new()
        // Users
        //-- Auth
        .route("/user/register", post(routes::users::register::register))
        .route("/user/login", post(routes::users::login::login))
        .route("/user/logout", post(routes::users::logout::logout))
        .route("/user/delete", delete(routes::users::delete::delete))
        //-- Information
        .route("/user/list_sessions", get(routes::users::list_sessions::list_sessions))
        //-- Settings

        // Teams

        // Zones

        // Records

        // Proxies

        // Admin

        // RPC
        .route("/rpc", post(rpc::rpc))

        // Misc
        .route("/", get(routes::status::status))

        .layer(
        	ServiceBuilder::new()
        		.layer(Extension(connection))
                .layer(Extension(amqp_channel))
        );
    
    let addr = env::var("LISTEN_ADDR")
        .unwrap_or("127.0.0.1:32204".to_string());
    
    let socket_addr: SocketAddr = addr.parse().expect("Failed to parse LISTEN_ADDR! Halting start-up.");

    let axum_builder = axum::Server::try_bind(&socket_addr);

    match axum_builder {
        Ok(axum_builder) => {
            info!("Driptorch Controller v{} is now listening on {}!", VERSION, socket_addr);

            axum_builder
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
