use google_calendar3::{CalendarHub, hyper_rustls, hyper_util, yup_oauth2};

pub type Hub =
    CalendarHub<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>;

pub async fn login() -> Hub {
    let secret = yup_oauth2::read_application_secret("client_secret.json")
        .await
        .expect("client_secret.json not found");
    let connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .unwrap()
        .https_only()
        .enable_http2()
        .build();

    let executor = hyper_util::rt::TokioExecutor::new();
    let auth = yup_oauth2::InstalledFlowAuthenticator::with_client(
        secret,
        yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        yup_oauth2::client::CustomHyperClientBuilder::from(
            hyper_util::client::legacy::Client::builder(executor).build(connector),
        ),
    )
    .persist_tokens_to_disk("token_cache.json")
    .build()
    .await
    .unwrap();

    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .unwrap()
                .https_or_http()
                .enable_http2()
                .build(),
        );

    CalendarHub::new(client, auth)
}
