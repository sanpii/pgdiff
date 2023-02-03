use clap::Parser;

#[derive(Parser)]
struct Args {
    old: String,
    new: String,
}

fn main() -> pgdiff::Result {
    let args = Args::parse();

    diff(&args)
}

fn diff(args: &Args) -> pgdiff::Result {
    let old = pgdiff::inspect::Database::new(&args.old)?;
    let new = pgdiff::inspect::Database::new(&args.new)?;

    let diff = pgdiff::diff::Diff::from(&old, &new);

    print!("{}", diff.sql()?);

    Ok(())
}
