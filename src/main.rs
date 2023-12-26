use clap::Parser;
use config::Config;
use config::{File, FileFormat};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    /// List instances in Integration environment
    Int(EnvArgs),
    /// List instances in Staging environment
    Stg(EnvArgs),
    /// List instances in Production environment
    Prd(EnvArgs),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct EnvArgs {
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

fn printit(project: String, token: String, pattern: String, long: bool, ip: bool) {
    println!("project: {:?}", project);
    println!("token: {:?}", token);
    println!("pattern: {:?}", pattern);
    println!("long: {:?}", long);
    println!("ip: {:?}", ip);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

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

    match args.cmd {
        Command::Int(args) => {
            let token = config.int.token;
            let project = config.int.project;
            let pattern = args.pattern;
            let long = args.long;
            let ip = args.ip;

            printit(project, token, pattern, long, ip)
            //compute::list_instances(token, project, pattern, long, ip)?;
        }
        Command::Stg(args) => {
            let token = config.stg.token;
            let project = config.stg.project;
            let pattern = args.pattern;
            let long = args.long;
            let ip = args.ip;

            printit(project, token, pattern, long, ip);
            //compute::list_instances(token, project, pattern, long, ip)?;
        }
        Command::Prd(args) => {
            let token = config.prd.token;
            let project = config.prd.project;
            let pattern = args.pattern;
            let long = args.long;
            let ip = args.ip;

            printit(project, token, pattern, long, ip);
            //compute::list_instances(token, project, pattern, long, ip)?;
        }
    }

    println!("done");

    Ok(())
}
