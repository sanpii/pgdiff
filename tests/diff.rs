#[test]
fn diff() -> Result<(), Box<dyn std::error::Error>> {
    let actual = load_diff()?;
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

#[test]
fn syntax() -> Result<(), Box<dyn std::error::Error>> {
    let diff = load_diff()?;

    let sql = format!("do $syntax_check$ begin return;{diff}end; $syntax_check$;");

    let url = std::env::var("NEW_URL").unwrap();
    let db = elephantry::Connection::new(&url)?;
    db.execute(&sql)?;

    Ok(())
}

fn load_diff() -> Result<String, Box<dyn std::error::Error>> {
    let old = db("OLD_URL", include_str!("old.sql"))?;
    let new = db("NEW_URL", include_str!("new.sql"))?;

    let pgdiff = pgdiff::diff::Diff::from(&old, &new);

    let diff = pgdiff.sql();

    Ok(diff)
}

fn db(env: &str, sql: &str) -> Result<pgdiff::inspect::Database, Box<dyn std::error::Error>> {
    let url = std::env::var(env).unwrap();
    let db = elephantry::Connection::new(&url)?;

    db.execute(&sql)?;

    let url = std::env::var(env).unwrap();
    let diff = pgdiff::inspect::Database::new(&url)?;

    Ok(diff)
}
