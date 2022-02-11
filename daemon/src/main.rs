use std::error::Error;
use tracing::info;
use tracing_subscriber::Layer;
use wg_activity_notify_core::Daemon;

fn main() -> Result<(), Box<dyn Error>> {
    let conf = match wg_activity_notify_core::config::Config::load() {
        Ok(val) => val,
        Err(err) => {
            panic!("{:?}", err);
        }
    };
    setup_tracing(get_trace_level(&conf.log_level));
    info!("Loading wg_activity_notify");
    let mut wg = Daemon::new(conf);
    wg.run();

    Ok(())
}

fn setup_tracing(log_level : Option<tracing::Level>) {
    let max_level = log_level.unwrap_or(tracing::Level::TRACE);
    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(format!("wg_activity_notify_daemon={}", max_level.as_str().to_lowercase()).parse().unwrap())
        .add_directive(format!("wg_activity_notify_core={}", max_level.as_str().to_lowercase()).parse().unwrap())
        .add_directive("reqwest=info".parse().unwrap())
        .add_directive("mio=info".parse().unwrap())
        .add_directive("want=info".parse().unwrap())
        .add_directive("hyper=info".parse().unwrap());


    let subscriber = tracing_subscriber::fmt()
        .with_max_level(max_level)
        .with_env_filter(filter)
        .finish();
    tracing_log::LogTracer::init().unwrap();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

fn get_trace_level(input : &str) -> Option<tracing::Level> {
    return match input.to_uppercase().as_str() {
        "TRACE" => Some(tracing::Level::TRACE),
        "DEBUG" => Some(tracing::Level::DEBUG),
        "INFO" => Some(tracing::Level::INFO),
        "WARN" => Some(tracing::Level::WARN),
        "ERROR" => Some(tracing::Level::ERROR),
        _ => None
    }
}