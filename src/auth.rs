use google_calendar3::{CalendarHub, hyper_rustls, hyper_util, yup_oauth2};

pub type Hub =
    CalendarHub<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>;

async fn login_service_account(sa_json: &str) -> Hub {
    let sa_key: yup_oauth2::ServiceAccountKey =
        serde_json::from_str(sa_json).expect("Invalid GOOGLE_SERVICE_ACCOUNT_KEY JSON");

    let auth = yup_oauth2::ServiceAccountAuthenticator::builder(sa_key)
        .build()
        .await
        .expect("Failed to build service account authenticator");

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

async fn login_installed_flow() -> Hub {
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

pub async fn login() -> Hub {
    match std::env::var("GOOGLE_SERVICE_ACCOUNT_KEY") {
        Ok(sa_json) => login_service_account(&sa_json).await,
        Err(_) => login_installed_flow().await,
    }
}
