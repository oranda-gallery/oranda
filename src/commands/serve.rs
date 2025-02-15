use camino::{Utf8Path, Utf8PathBuf};
use std::net::SocketAddr;

use oranda::config::Config;
use oranda::errors::*;

use axum::{http::StatusCode, routing::get_service, Router};

use clap::Parser;
use tower_http::services::ServeDir;

#[derive(Debug, Default, Parser)]
pub struct Serve {
    #[arg(long, default_value = "7979")]
    port: u16,
}

impl Serve {
    pub fn new(port: Option<u16>) -> Self {
        Serve {
            port: port.unwrap_or(7979),
        }
    }

    pub fn run(&self) -> Result<()> {
        let workspace_config_path = &Utf8PathBuf::from("./oranda-workspace.json");
        let config = if workspace_config_path.exists() {
            Config::build(workspace_config_path)?
        } else {
            Config::build(&Utf8PathBuf::from("./oranda.json"))?
        };
        if Utf8Path::new(&config.build.dist_dir).is_dir() {
            if let Some(prefix) = config.build.path_prefix {
                tracing::debug!("`path_prefix` configured: {}", &prefix);
                self.serve_prefix(&config.build.dist_dir, &prefix)?;
            } else {
                self.serve(&config.build.dist_dir)?;
            }
            Ok(())
        } else {
            Err(OrandaError::BuildNotFound {
                dist_dir: config.build.dist_dir.to_string(),
            })
        }
    }

    #[tokio::main]
    async fn serve(&self, dist_dir: &str) -> Result<()> {
        let serve_dir =
            get_service(ServeDir::new(dist_dir)).handle_error(|error: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                )
            });

        let app = Router::new().nest_service("/", serve_dir);

        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let msg = format!("Your project is available at: http://{}", addr);
        tracing::info!(success = true, "{}", &msg);
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .expect("failed to start server");
        Ok(())
    }

    #[tokio::main]
    async fn serve_prefix(&self, dist_dir: &str, prefix: &str) -> Result<()> {
        let serve_dir =
            get_service(ServeDir::new(dist_dir)).handle_error(|error: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                )
            });
        let prefix_route = format!("/{}", prefix);
        let app = Router::new().nest_service(&prefix_route, serve_dir);

        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let msg = format!("Your project is available at: http://{}/{}", addr, prefix);
        tracing::info!(success = true, "{}", &msg);
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .expect("failed to start server");
        Ok(())
    }
}
