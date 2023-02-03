#[test]
fn diff() -> Result<(), Box<dyn std::error::Error>> {
    let old = db("DB1_URL", include_str!("db1.sql"))?;
    let new = db("DB2_URL", include_str!("db2.sql"))?;

    let pgdiff = pgdiff::diff::Diff::from(&old, &new);

    let actual = pgdiff.sql()?;
    let expected = include_str!("diff.sql");

    if actual != expected {
        let diff = similar::TextDiff::from_lines(expected, &actual);

        for op in diff.ops() {
            for change in diff.iter_changes(op) {
                let (sign, style) = match change.tag() {
                    similar::ChangeTag::Delete => ("-", console::Style::new().red()),
                    similar::ChangeTag::Insert => ("+", console::Style::new().green()),
                    similar::ChangeTag::Equal => (" ", console::Style::new()),
                };
                print!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));
            }
        }
        panic!();
    }

    Ok(())
}

fn db(env: &str, sql: &str) -> Result<pgdiff::inspect::Database, Box<dyn std::error::Error>> {
    let url = std::env::var(env).unwrap();
    let db = elephantry::Connection::new(&url)?;

    db.execute(&sql)?;

    let url = std::env::var(env).unwrap();
    let diff = pgdiff::inspect::Database::new(&url)?;

    Ok(diff)
}
