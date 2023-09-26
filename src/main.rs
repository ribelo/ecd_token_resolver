use std::net::SocketAddr;

use anyhow::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use ecd_token_resolver::Ecd;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct EcdRequest {
    login: String,
    password: String,
    chrome_path: Option<String>,
}

#[derive(Serialize)]
struct EcdResponse {
    token: String,
}

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

async fn ecd_get_token(Json(req): Json<EcdRequest>) -> Result<Json<EcdResponse>, AppError> {
    let mut ecd = Ecd::new(req.login, req.password, false, req.chrome_path)
        .await
        .unwrap();
    let token = ecd.login().await.unwrap();

    Ok(Json(EcdResponse { token }))
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/ecd_get_token", post(ecd_get_token));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", &addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
