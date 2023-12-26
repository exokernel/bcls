use clap::Parser;
use config::Config;
use config::{File, FileFormat};

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

mod bclsconfig;
mod compute;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("habitat is {}", args.env);
    println!("-l is {}", args.long);
    println!("-i is {}", args.ip);
    println!("pattern is '{}'", args.pattern);

    let configpath = dirs::home_dir()
        .expect("Homedir not found")
        .join(".bcls/config.toml");

    // get habitat and token from config file
    let builder = Config::builder()
        // read from config in .bcls/config.toml under home directory
        // or from config in current directory
        .add_source(File::from(configpath).required(false))
        .add_source(File::new("config", FileFormat::Toml).required(false));
    let config = builder.build()?;

    let config: bclsconfig::Configurations = config.try_deserialize()?;

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

    println!("done");

    Ok(())
}
