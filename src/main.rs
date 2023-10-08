use std::sync::{Arc, Mutex};

use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use anyhow::Context;
use askama::Template;
use rand::Rng;

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Form, Router,
};

use serde::Deserialize;

use opentelemetry::global;
use opentelemetry::trace::Tracer;

#[derive(Debug)]
struct AppState {
    todos: Mutex<Vec<String>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());

    // let provider = TracerProvider::builder()
    // .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
    // .build();

    // let tracer = provider.tracer("readme_example");

    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("axum-tracing-test")
        .install_simple()
        .unwrap();

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                // .unwrap_or_else(|_| "waha=debug".into()),
                .unwrap_or_else(|_| "debug".into()),
        )
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    // tracing_subscriber::registry()
    //     .with(
    //         tracing_subscriber::EnvFilter::try_from_default_env()
    //             .unwrap_or_else(|_| "debug".into()),
    //     )
    //     // .with(tracing_subscriber::fmt::layer())
    //     .with(tracing_subscriber::fmt::layer().json())
    //     .init();
    info!("initializing router...  bro...");

    let app_state = Arc::new(AppState {
        todos: Mutex::new(vec![]),
    });

    let asset_path = std::env::current_dir().unwrap();
    let port = 8000_u16;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    let api_router = Router::new()
        .route("/hello", get(hello_from_the_server))
        .route("/todos", post(add_todo))
        .with_state(app_state);

    let router = Router::new()
        .nest("/api", api_router)
        .route("/", get(hello))
        .route("/another-page", get(another_page))
        .nest_service(
            "/assets",
            ServeDir::new(format!("{}/assets", asset_path.to_str().unwrap())),
        )
        .layer(TraceLayer::new_for_http());
    info!(
        "router initialzied... now listeing on port {},  bro...",
        port
    );

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .context("error while starting server, broo....")?;

    Ok(())
}

#[tracing::instrument]
async fn hello() -> impl IntoResponse {
    let template = HelloTemplate {};
    HtmlTemplate(template)
}

#[tracing::instrument]
async fn another_page() -> impl IntoResponse {
    let template = AnotherPageTemplate {};
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate;

#[derive(Template)]
#[template(path = "another-page.html")]
struct AnotherPageTemplate;

// wrapper type for encapsule HTML parsing by askama
struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template bro. Error: {}", err),
            )
                .into_response(),
        }
    }
}

#[tracing::instrument]
async fn hello_from_the_server() -> &'static str {
    "HELSJALKDJKASDa"
}

#[derive(Template)]
#[template(path = "todo-list.html")]
struct TodoList {
    todos: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct TodoRequest {
    todo: String,
}

#[tracing::instrument]
async fn add_todo(
    State(state): State<Arc<AppState>>,
    Form(todo): Form<TodoRequest>,
) -> impl IntoResponse {
    info!("A");
    let mut lock = state.todos.lock().unwrap();
    lock.push(todo.todo);
    let template = TodoList {
        todos: lock.clone(),
    };
    f();
    HtmlTemplate(template)
}

#[tracing::instrument]
fn f() -> () {
    info!("B");
    let rng = rand::thread_rng().gen();
    g(rng);
}

#[tracing::instrument]
fn g(rng: f32) -> () {
    if rng < 0.5 {
        info!("C");
    } else {
        error!("E")
    }
}
