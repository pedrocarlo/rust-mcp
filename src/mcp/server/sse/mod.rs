mod routes;

use async_stream::try_stream;
use axum::{
    extract::{Query, Request, State},
    http::{HeaderMap, StatusCode},
    response::{
        sse::{Event, Sse},
        Response,
    },
    routing::{get, post},
    Json, Router,
};
use futures::stream::Stream;
use serde::Deserialize;
use std::{convert::Infallible, fmt, sync::Arc, time::Duration};
use tower_http::{
    trace::{DefaultOnRequest, TraceLayer},
    LatencyUnit,
};
use tracing::{Level, Span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use crate::mcp::{
    schema::{self},
    server::{error::ApiError, request::handle_request},
};

use super::{error::Result, Message, Server, SessionId};

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

pub async fn serve(mcp_server: Server, endpoint: &str) -> Result<()> {
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
        .or_else(|err| Err(ApiError::IoError(err)))?;

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
        // .route_layer(middleware::from_fn(print_request_response))
        .with_state(shared_state);

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .or_else(|err| Err(ApiError::IoError(err)))
}

async fn sse_handler(
    State(state): State<Arc<SseState>>,
) -> Result<Sse<impl Stream<Item = Result<Event>>>> {
    tracing::debug!("sse handler");

    let session_id: SessionId = Uuid::new_v4().to_string();

    let mut client = {
        // Using block here so that lock can be dropped
        state.mcp_server.new_connection(&session_id)?
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
                let mut_client = &mut client;
                match mut_client.recv.recv().await {
                    Some(v) => {

                        if let Some(message) = serde_json::to_string(&v.sse_message).ok() {
                            tracing::debug!("sending message");
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

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    ))
}

async fn message_handler(
    State(state): State<Arc<SseState>>,
    session_query: Query<SessionQuery>,
    Json(message): Json<schema::JSONRPCMessage>,
    // message: String
) -> Result<StatusCode> {
    tracing::debug!("{message:#?}");

    let session_id = session_query.0.session_id;

    let res = match message {
        schema::JSONRPCMessage::Request(ref req) => {
            handle_request(&state.mcp_server, req, &session_id)
        }
        _ => todo!(),
    }?;

    let client_conn = {
        // Block here to drop lock slightly earlier
        let map = state
            .mcp_server
            .clients
            .read()
            .or_else(|_| Err(ApiError::PoisonedLock))?;

        if let Some(client_conn) = map.get(&session_id) {
            client_conn.clone()
        } else {
            return Ok(StatusCode::OK);
        }
    };

    let tx = client_conn
        .lock()
        .or_else(|_| Err(ApiError::PoisonedLock))?
        .send
        .clone();

    // TODO Ignore error for now
    tx.send(Message {
        session_id: session_id.to_owned(),
        sse_message: res,
    })
    .await
    .ok();

    Ok(StatusCode::OK)
}
