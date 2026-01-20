use actix_web::{web, App, HttpResponse, HttpServer};
use mmo_api::openapi::ApiDoc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
struct Config {
    host: String,
    port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8081,
        }
    }
}

fn parse_args() -> Config {
    let args: Vec<String> = std::env::args().collect();
    let mut config = Config::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--host" | "-h" => {
                if i + 1 < args.len() {
                    config.host = args[i + 1].clone();
                    i += 2;
                    continue;
                }
            }
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    config.port = args[i + 1].parse().unwrap_or(8081);
                    i += 2;
                    continue;
                }
            }
            "--help" => {
                println!("Swagger UI Server for MMO API");
                println!();
                println!("USAGE:");
                println!("    cargo run --bin swagger_server [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("    -h, --host <HOST>    Host to bind (default: 127.0.0.1)");
                println!("    -p, --port <PORT>    Port to bind (default: 8081)");
                println!("        --help           Show this help message");
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    config
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().body("OK")
}

async fn openapi_json() -> HttpResponse {
    let json = ApiDoc::openapi()
        .to_pretty_json()
        .unwrap_or_else(|_| "{}".to_string());

    HttpResponse::Ok()
        .content_type("application/json")
        .body(json)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = parse_args();
    let bind_address = format!("{}:{}", config.host, config.port);

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║              MMO API - Swagger UI Server                   ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  Swagger UI:   http://{}:{}/swagger-ui/            ║", config.host, config.port);
    println!("║  OpenAPI JSON: http://{}:{}/api-docs/openapi.json  ║", config.host, config.port);
    println!("║  Health:       http://{}:{}/health                 ║", config.host, config.port);
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("Press Ctrl+C to stop");

    HttpServer::new(move || {
        App::new()
            .route("/health", web::get().to(health_check))
            .route("/api-docs/openapi.json", web::get().to(openapi_json))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
            .route(
                "/",
                web::get().to(|| async {
                    HttpResponse::Found()
                        .append_header(("Location", "/swagger-ui/"))
                        .finish()
                }),
            )
    })
    .bind(&bind_address)?
    .run()
    .await
}
