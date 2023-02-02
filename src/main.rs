use clap::Parser;

#[derive(Parser)]
struct Args {
    db1: String,
    db2: String,
}

fn main() -> pgdiff::Result {
    let args = Args::parse();

    diff(&args)
}

fn diff(args: &Args) -> pgdiff::Result {
    let s1 = pgdiff::inspect::Database::new(&args.db1)?;
    let s2 = pgdiff::inspect::Database::new(&args.db2)?;

    let diff = pgdiff::diff::Diff::from(&s1, &s2);

    print!("{}", diff.sql()?);

    Ok(())
}
