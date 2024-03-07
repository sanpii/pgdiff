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
    let diff = diff.trim_start_matches("begin;\n\n")
        .trim_end_matches("commit;\n");

    let sql = format!("do $syntax_check$ begin return;{diff}end; $syntax_check$;");

    let url = std::env::var("NEW_URL").unwrap();
    let db = elephantry::Connection::new(&url)?;
    db.execute(&sql)?;

    Ok(())
}

#[derive(envir::Deserialize)]
struct Config {
    old_url: String,
    new_url: String,
}

fn load_diff() -> Result<String, Box<dyn std::error::Error>> {
    use envir::Deserialize;

    envir::dotenv();
    let config = Config::from_env()?;
    let old = db(&config.old_url, include_str!("old.sql"))?;
    let new = db(&config.new_url, include_str!("new.sql"))?;

    let pgdiff = pgdiff::diff::Diff::from(&old, &new);

    let diff = pgdiff.sql();

    Ok(diff)
}

fn db(url: &str, sql: &str) -> Result<pgdiff::inspect::Database, Box<dyn std::error::Error>> {
    let db = elephantry::Connection::new(&url)?;

    db.execute(&sql)?;

    let diff = pgdiff::inspect::Database::new(&url)?;

    Ok(diff)
}
