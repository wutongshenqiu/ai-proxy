use ai_proxy_core::config::{Config, ConfigWatcher};
use ai_proxy_provider::routing::CredentialRouter;
use arc_swap::ArcSwap;
use clap::Parser;
use std::sync::Arc;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "ai-proxy", version, about = "AI API Proxy Gateway")]
struct Cli {
    #[arg(short, long, default_value = "config.yaml", env = "AI_PROXY_CONFIG")]
    config: String,

    #[arg(long, env = "AI_PROXY_HOST")]
    host: Option<String>,

    #[arg(long, env = "AI_PROXY_PORT")]
    port: Option<u16>,

    #[arg(long, default_value = "info", env = "AI_PROXY_LOG_LEVEL")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    // Init tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&cli.log_level)),
        )
        .init();

    // Load config
    let mut config = Config::load(&cli.config).unwrap_or_else(|e| {
        tracing::warn!(
            "Failed to load config from '{}': {e}, using defaults",
            cli.config
        );
        Config::default()
    });

    // CLI overrides
    if let Some(host) = cli.host {
        config.host = host;
    }
    if let Some(port) = cli.port {
        config.port = port;
    }

    // Build provider components
    let executors =
        ai_proxy_provider::build_registry(config.proxy_url.clone());

    let router = Arc::new(CredentialRouter::new(config.routing.strategy.clone()));
    router.update_from_config(&config);

    let translators = Arc::new(ai_proxy_translator::build_registry());
    let executors = Arc::new(executors);

    tracing::info!(
        "Loaded {} claude keys, {} openai keys, {} gemini keys, {} compat keys",
        config.claude_api_key.len(),
        config.openai_api_key.len(),
        config.gemini_api_key.len(),
        config.openai_compatibility.len(),
    );

    let config = Arc::new(ArcSwap::from_pointee(config));

    let metrics = Arc::new(ai_proxy_core::metrics::Metrics::new());

    // Build AppState
    let state = ai_proxy_server::AppState {
        config: config.clone(),
        router: router.clone(),
        executors,
        translators,
        metrics,
    };

    let app_router = ai_proxy_server::build_router(state);

    // Start config watcher — update credentials on reload
    let watcher_router = router.clone();
    let _watcher = ConfigWatcher::start(cli.config.clone(), config.clone(), move |new_cfg| {
        watcher_router.update_from_config(new_cfg);
        tracing::info!(
            "Config reloaded: {} claude keys, {} openai keys, {} gemini keys, {} compat keys",
            new_cfg.claude_api_key.len(),
            new_cfg.openai_api_key.len(),
            new_cfg.gemini_api_key.len(),
            new_cfg.openai_compatibility.len(),
        );
    });

    // Start server
    let cfg = config.load();
    let addr = format!("{}:{}", cfg.host, cfg.port);

    if cfg.tls.enable {
        let cert_path = cfg.tls.cert.as_ref().expect("TLS cert required");
        let key_path = cfg.tls.key.as_ref().expect("TLS key required");

        use rustls_pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};

        let certs: Vec<CertificateDer<'static>> =
            CertificateDer::pem_file_iter(cert_path)?.collect::<Result<Vec<_>, _>>()?;
        let key = PrivateKeyDer::from_pem_file(key_path)?;

        let tls_config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;
        let tls_acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(tls_config));

        tracing::info!("Starting HTTPS server on {addr}");
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        let shutdown = shutdown_signal();
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                result = listener.accept() => {
                    let (stream, peer_addr) = result?;
                    let acceptor = tls_acceptor.clone();
                    let router = app_router.clone();
                    tokio::spawn(async move {
                        match acceptor.accept(stream).await {
                            Ok(tls_stream) => {
                                let io = hyper_util::rt::TokioIo::new(tls_stream);
                                let service = hyper::service::service_fn(
                                    move |req: hyper::Request<hyper::body::Incoming>| {
                                        let router = router.clone();
                                        async move {
                                            let (parts, body) = req.into_parts();
                                            let body = axum::body::Body::new(body);
                                            let req = axum::http::Request::from_parts(parts, body);
                                            Ok::<_, std::convert::Infallible>(
                                                tower::ServiceExt::oneshot(router, req)
                                                    .await
                                                    .expect("infallible"),
                                            )
                                        }
                                    },
                                );
                                if let Err(e) = hyper_util::server::conn::auto::Builder::new(
                                    hyper_util::rt::TokioExecutor::new(),
                                )
                                .serve_connection(io, service)
                                .await
                                {
                                    tracing::error!("TLS connection error from {peer_addr}: {e}");
                                }
                            }
                            Err(e) => tracing::error!("TLS accept error from {peer_addr}: {e}"),
                        }
                    });
                }
                _ = &mut shutdown => {
                    tracing::info!("Stopping TLS listener, waiting for connections to drain...");
                    break;
                }
            }
        }
        // Give in-flight connections time to finish
        tokio::time::sleep(Duration::from_secs(1)).await;
    } else {
        tracing::info!("Starting HTTP server on {addr}");
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app_router)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
    }

    tracing::info!("Server shut down.");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, draining connections...");
}
