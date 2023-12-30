use clap::Parser;
use config::Config;
use config::{File, FileFormat};
#[macro_use]
extern crate prettytable;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            let project = config.int.project;
            let pattern = args.pattern;
            let long = args.long;
            let ip = args.ip;

            show_instances(&project, &pattern, long, ip).await?;
        }
        Command::Stg(args) => {
            let project = config.stg.project;
            let pattern = args.pattern;
            let long = args.long;
            let ip = args.ip;

            show_instances(&project, &pattern, long, ip).await?;
        }
        Command::Prd(args) => {
            let project = config.prd.project;
            let pattern = args.pattern;
            let long = args.long;
            let ip = args.ip;

            show_instances(&project, &pattern, long, ip).await?;
        }
    }

    Ok(())
}

async fn show_instances(
    project: &str,
    pattern: &str,
    _long: bool,
    _ip: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let c = compute::new_compute(project.to_string());
    let instances = c.list_instances(pattern).await?;

    //print_instances(instances);
    print_instances_table(instances);

    Ok(())
}

#[allow(dead_code)]
fn print_instances(instances: Vec<compute::Instance>) {
    // Print each instance as a string
    for inst in instances {
        println!("{}", inst.as_string());
    }
}

fn print_instances_table(instances: Vec<compute::Instance>) {
    // Print a header for each field of the Instance struct
    // and then print each instance as a row in the table
    let mut table = prettytable::Table::new();
    table.add_row(row![
        "Name",
        "IP",
        "Zone",
        "Machine Type",
        "CPU Platform",
        "Status",
        "Labels"
    ]);

    for inst in instances {
        table.add_row(row![
            inst.name,
            inst.ip,
            inst.zone,
            inst.machine_type,
            inst.cpu_platform,
            inst.status,
            inst.labels
        ]);
    }

    table.printstd();
}
