mod diff;
mod errors;
mod inspect;

use clap::Parser;
use errors::*;

#[derive(Parser)]
struct Args {
    db1: String,
    db2: String,
}

fn main() -> Result {
    let args = Args::parse();

    diff(&args)
}

fn diff(args: &Args) -> Result {
    let s1 = inspect::Database::new(&args.db1)?;
    let s2 = inspect::Database::new(&args.db2)?;

    let diff = diff::Diff::from(&s1, &s2);

    print!("{}", diff.sql()?);

    Ok(())
}
