use clap::Parser;
use config::Config;
use config::{File, FileFormat};
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of environment / habitat to use. E.g. int/stg/prd
    #[arg(short, long)]
    env: String,

    /// Long output. Show machine-type, cpu-platform, zone, cell, etc. info.
    /// By default only instance-name and IP are shown.
    /// Can't be used with ip option
    #[arg(short, long, conflicts_with = "ip")]
    long: bool,

    /// Show IP only. Handy for pipeing to other commands like bolt.
    /// Can't be used with long option
    #[arg(short, long, conflicts_with = "long")]
    ip: bool,

    /// Fields

    /// Search pattern to match against instance names. E.g. "^store-lb"
    pattern: String,
}

#[derive(Debug, Deserialize)]
struct Habitat {
    project: String,
    token: String,
}

#[derive(Debug, Deserialize)]
struct Configurations {
    int: Habitat,
    stg: Habitat,
    prd: Habitat,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("habitat is {}", args.env);
    println!("-l is {}", args.long);
    println!("-i is {}", args.ip);
    println!("pattern is '{}'", args.pattern);

    // get habitat and token from config file
    let builder = Config::builder()
        .set_default("default", "1")?
        .add_source(File::new("config", FileFormat::Toml));

    let config = builder.build()?;

    let config: Configurations = config.try_deserialize()?;

    match args.env.as_str() {
        "int" => println!(
            "Project: {}, Token: {}",
            config.int.project, config.int.token
        ),
        "stg" => println!(
            "Project: {}, Token: {}",
            config.stg.project, config.stg.token
        ),
        "prd" => println!(
            "Project: {}, Token: {}",
            config.prd.project, config.prd.token
        ),
        _ => println!("Invalid environment"),
    }

    Ok(())
}
