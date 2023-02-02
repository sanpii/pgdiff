#[test]
fn diff() -> Result<(), Box<dyn std::error::Error>> {
    let old = db("DB1_URL", include_str!("db1.sql"))?;
    let new = db("DB2_URL", include_str!("db2.sql"))?;

    let pgdiff = pgdiff::diff::Diff::from(&old, &new);

    assert_eq!(pgdiff.sql()?, include_str!("diff.sql"));

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
