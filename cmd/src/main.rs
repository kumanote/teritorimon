use config::configs::SelfValidation;
use futures::lock::Mutex;
use logger::default::DefaultLoggerBuilder;
use logger::prelude::*;
use std::collections::HashMap;
use std::panic::{self, PanicInfo};
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use structopt::StructOpt;
use teritoricli::TeritoridClient;

#[derive(Debug, StructOpt)]
#[structopt(about = "teritorimon running options")]
struct Opts {
    #[structopt(short = "c", long, help = "Path to Config")]
    config: Option<PathBuf>,
}

fn main() {
    panic::set_hook(Box::new(move |panic_info: &PanicInfo<'_>| {
        let details = format!("{}", panic_info);
        crash!("{}", details);
        logger::flush();
        // Kill the process
        process::exit(12);
    }));

    // load configs
    let options: Opts = Opts::from_args();
    let config =
        config::load_app_config(options.config.as_ref()).expect("Failed to load config file...");
    if let Err(err) = config.validate() {
        eprintln!("{}", err);
        return;
    }

    config::set_app_config(Arc::new(config));

    let config = config::app_config();

    // set up logger
    let mut logger_builder = DefaultLoggerBuilder::new();
    logger_builder.is_async(config.logger.is_async);
    if let Some(chan_size) = config.logger.chan_size {
        logger_builder.channel_size(chan_size);
    }
    if let Some(level) = config.logger.level.as_deref() {
        let level = level.parse().expect("log level must be valid");
        logger_builder.level(level);
    }
    if let Some(airbrake_host) = config.logger.airbrake_host.as_deref() {
        logger_builder.airbrake_host(airbrake_host.to_owned());
    }
    if let Some(airbrake_project_id) = config.logger.airbrake_project_id.as_deref() {
        logger_builder.airbrake_project_id(airbrake_project_id.to_owned());
    }
    if let Some(airbrake_project_key) = config.logger.airbrake_project_key.as_deref() {
        logger_builder.airbrake_project_key(airbrake_project_key.to_owned());
    }
    if let Some(airbrake_environment) = config.logger.airbrake_environment.as_deref() {
        logger_builder.airbrake_environment(airbrake_environment.to_owned());
    }
    let _logger = logger_builder.build();

    // Let's now log some important information, since the logger is set up
    debug!(
        "Loaded teritori daemon monitoring tool config, config: {:?}",
        config
    );

    // set up teritori client pool
    let mut client_pool = HashMap::new();
    for checker in &config.checkers {
        let endpoint = checker.teritori_grpc_endpoint();
        info!("this tool will connect to {}", endpoint.as_str());
        let client = TeritoridClient::new(endpoint.clone());
        client_pool.insert(endpoint, Arc::new(Mutex::new(client)));
    }
    teritoricli::set_client_pool(client_pool);

    if let Err(err) = teritorimon::start() {
        error!("{}", err);
    }

    logger::flush();
}
