use axum::{
    body::{Body, Bytes},
    handler::Handler,
    http::{Method, Request, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    Router,
    routing::{get, post},
};
use log::info;
use tower_http::{
    cors::{Any, CorsLayer},
};

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

pub fn init() -> Router {
    Router::new()
        .route("/", get(handler))
        .route("/", post(|| async move { "Hello from `POST /`" }))
        .layer(middleware::from_fn(print_request_response))
        .layer(
            // see https://docs.rs/tower-http/latest/tower_http/cors/index.html
            // for more details
            //
            // pay attention that for some request types like posting content-type: application/json
            // it is required to add ".allow_headers(vec![http::header::CONTENT_TYPE])"
            // or see this issue https://github.com/tokio-rs/axum/issues/849
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS]),
        )
        .fallback(handler_404.into_service())
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn print_request_response(req: Request<Body>, next: Next<Body>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
    where
        B: axum::body::HttpBody<Data=Bytes>,
        B::Error: std::fmt::Display,
{
    let bytes = match hyper::body::to_bytes(body).await {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {} body: {}", direction, err),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        info!("{} body = {:?}", direction, body);
    }

    Ok(bytes)
}