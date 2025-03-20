use lambda_runtime::tracing;

pub fn init_custom_rust_subscriber() {
    tracing::subscriber::fmt().json()
        .with_max_level(tracing::Level::INFO)
        // This needs to be set to remove duplicated information in the log.
        .with_current_span(false)
        // This needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // Disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        // Remove the name of the function from every log entry.
        .with_target(false)
        .init();
}