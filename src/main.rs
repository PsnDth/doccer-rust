mod config;
mod commands;

use tokio;

use config::Config;
use serde_json::from_reader;

use serenity::{
    prelude::Client,
    framework::standard::StandardFramework,
    model::id::UserId,
};


use commands::*;

use std::fs::File;
use std::io::BufReader;


#[tokio::main]
async fn main() {
    
    let config_file = File::open("config.json")
                        .expect("Couldn't find config file");
    let config_reader = BufReader::new(config_file);
    let config: Config = from_reader(config_reader)
                            .expect("Incorrectly formatted config.json");
    
    let framework = StandardFramework::new()
                                        .configure(|c| c
                                            .owners(vec![UserId(config.owner.parse().unwrap())].into_iter().collect())
                                            .on_mention(Some(UserId(config.id)))
                                            .case_insensitivity(true)
                                            .delimiters(vec![" ", ", ", ","])
                                            .no_dm_prefix(true)
                                            .prefix(&config.prefix))
                                        .group(&GENERAL_GROUP)
                                        .help(&EXEC_HELP);
    
    let mut client = Client::builder(&config.token)
                                .framework(framework)
                                .await.expect("Error occured while building client");
    println!("STARTED");
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}