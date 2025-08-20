use opentelemetry::trace::TracerProvider as _;
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*};
use serde::{Serialize, Deserialize};

mod models;

pub use crate::models::history::HistoryEntry;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrderByEnum {
    Desc,
    Asc,
}

impl OrderByEnum {
    pub fn to_string(self) -> String {
        match self {
            OrderByEnum::Desc => "DESC",
            OrderByEnum::Asc => "ASC"
        }.to_string()
    }
}

#[tarpc::service]
pub trait HistoryService {
    async fn get_entry(id: usize) -> Option<HistoryEntry>;
    async fn list_entries(page: usize, page_size: usize, order: OrderByEnum) -> Vec<HistoryEntry>;
    async fn count_entries() -> usize;
}

/// Initializes an OpenTelemetry tracing subscriber with a OTLP backend.
pub fn init_tracing(
    service_name: &'static str,
) -> anyhow::Result<opentelemetry_sdk::trace::SdkTracerProvider> {
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_service_name(service_name)
                .build(),
        )
        .build();
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());
    let tracer = tracer_provider.tracer(service_name);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::NEW | FmtSpan::CLOSE))
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()?;

    Ok(tracer_provider)
}
