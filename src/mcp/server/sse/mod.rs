mod routes;

use async_stream::try_stream;
use axum::{
    body::{Body, Bytes},
    extract::{MatchedPath, Query, Request, State},
    handler::Handler,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{
        sse::{Event, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Extension, Json, Router,
};
use futures::stream::Stream;
use http_body_util::BodyExt;
use querystring::stringify;
use serde::{de::Error, Deserialize};
use std::{convert::Infallible, fmt, sync::Arc, time::Duration};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::{
    classify::ServerErrorsFailureClass,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::{info_span, Level, Span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use crate::mcp::schema;

use super::{Message, Server, SessionId};

// Sse Server should live as long as mcp_server
// But mcp_server can live longer

struct SseState {
    mcp_server: Server,
    endpoint: String,
}

#[derive(Debug, Deserialize)]
struct SessionQuery {
    #[serde(rename = "sessionId")]
    session_id: String,
}

// Got from tower_http
struct Latency {
    unit: LatencyUnit,
    duration: Duration,
}

impl fmt::Display for Latency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.unit {
            LatencyUnit::Seconds => write!(f, "{} s", self.duration.as_secs_f64()),
            LatencyUnit::Millis => write!(f, "{} ms", self.duration.as_millis()),
            LatencyUnit::Micros => write!(f, "{} μs", self.duration.as_micros()),
            LatencyUnit::Nanos => write!(f, "{} ns", self.duration.as_nanos()),
            _ => Ok(()),
        }
    }
}

#[derive(Clone)]
struct RequestContext {}

pub async fn serve(mcp_server: Server, endpoint: &str) -> Result<(), std::io::Error> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let shared_state = Arc::new(SseState {
        mcp_server,
        endpoint: endpoint.to_string(),
    });

    let port = shared_state.mcp_server.port;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    let app = Router::new()
        .route("/sse", get(sse_handler))
        .route("/messages", post(message_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    tracing::span!(
                        Level::INFO,
                        "request",
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                        // headers = ?request.headers(),
                        status_code = tracing::field::Empty,
                    )
                })
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(|response: &Response<_>, _latency: Duration, _span: &Span| {
                    _span.record(
                        "status_code",
                        &tracing::field::display(response.status().as_u16()),
                    );

                    let latency = Latency {
                        unit: LatencyUnit::Micros,
                        duration: _latency,
                    };

                    tracing::info!(%latency, "finished processing request");
                })
                .on_eos(
                    |_trailers: Option<&HeaderMap>, stream_duration: Duration, _span: &Span| {
                        tracing::debug!("stream closed after {:?}", stream_duration)
                    },
                ),
        )
        .route_layer(middleware::from_fn(print_request_response))
        .with_state(shared_state);

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await
}

async fn sse_handler(
    State(state): State<Arc<SseState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    tracing::debug!("sse handler");

    let session_id: SessionId = Uuid::new_v4().to_string();

    let mut client = {
        // Using block here so that lock can be dropped
        state.mcp_server.new_connection(&session_id)
    };

    tracing::debug!("created client");

    let session_uri = format!("{}?{}={}", state.endpoint, "sessionId", &session_id);

    let mut endpoint_sent = false;

    let stream = try_stream! {
        loop {
            if !endpoint_sent {
                endpoint_sent = true;
                yield Event::default().event("endpoint").data(session_uri.clone())
            } else {
                match client.recv.recv().await {
                    Some(v) => {
                        if let Some(message) = serde_json::to_string(&v.sse_message).ok() {
                            yield Event::default().event("message").data(message)
                        } else {
                            // TODO maybe here just send an error message
                            ()
                        }
                    },
                   None => {
                    state.mcp_server.close_connection(session_id.clone());
                    ()
                   },
                }
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

async fn message_handler(
    State(state): State<Arc<SseState>>,
    session_query: Query<SessionQuery>,
    Json(message): Json<schema::JSONRPCMessage>,
    // message: String
) -> impl IntoResponse {
    tracing::debug!("{message:#?}");
    session_query.0.session_id;

    StatusCode::OK
}

async fn print_request_response(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // let (parts, body) = req.into_parts();
    // let bytes = buffer_and_print("request", body).await?;
    // let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    // let (parts, body) = res.into_parts();
    // let bytes = buffer_and_print("response", body).await?;
    // let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{direction} body = {body:?}");
    }

    Ok(bytes)
}
