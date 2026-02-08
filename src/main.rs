use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
struct CliArgs {}

fn main() -> anyhow::Result<()> {
    CliArgs::parse();
    Ok(())
}
