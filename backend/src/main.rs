//! Rabovel backend - blockchain stock broker and staking platform.
//! Entry point: starts HTTP server with routes, DB pool, Ethereum provider.

use actix_cors::Cors;
use actix_governor::GovernorConfigBuilder;
use actix_web::{web, App, HttpServer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use rabovel_backend::{
    config::Config,
    db,
    ethereum,
    routes::{auth, stake, stocks, trade, user},
};

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    let pool = db::create_pool(&config).await?;
    let eth_provider = ethereum::create_provider(&config).await?;

    let governor_conf = GovernorConfigBuilder::default()
        .per_second(2)
        .burst_size(config.rate_limit_requests)
        .finish()
        .unwrap();

    let cors = build_cors(&config);
    let port = config.server_port;
    let host = config.server_host.clone();

    tracing::info!("Starting server on {}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(eth_provider.clone()))
            .wrap(tracing_actix_web::TracingLogger::default())
            .wrap(cors.clone())
            .wrap(actix_governor::Governor::new(&governor_conf))
            .route("/health", web::get().to(health))
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(auth::register))
                    .route("/login", web::post().to(auth::login)),
            )
            .service(
                web::scope("/user").service(
                    web::resource("/profile")
                        .route(web::get().to(user::profile))
                        .route(web::patch().to(user::update_profile)),
                ),
            )
            .service(
                web::scope("/trade")
                    .route("/place-order", web::post().to(trade::place_order)),
            )
            .service(
                web::scope("/stake")
                    .route("/lock", web::post().to(stake::lock))
                    .route("/claim-rewards", web::post().to(stake::claim_rewards))
                    .route("/position", web::get().to(stake::position)),
            )
            .service(
                web::scope("/stocks")
                    .route("/list", web::get().to(stocks::list))
                    .route("/view/{id_or_symbol}", web::get().to(stocks::view))
                    .route("/choose", web::post().to(stocks::choose)),
            )
    })
    .bind((host.as_str(), port))?
    .run()
    .await?;

    Ok(())
}

async fn health() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

fn build_cors(config: &Config) -> Cors {
    let mut cors = Cors::default();
    for origin in &config.cors_origins {
        if origin == "*" {
            return Cors::permissive();
        }
        cors = cors.allowed_origin(origin);
    }
    cors.allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            actix_web::http::header::AUTHORIZATION,
            actix_web::http::header::CONTENT_TYPE,
        ])
}
