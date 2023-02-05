use std::collections::BTreeMap;

trait Comparable: std::fmt::Debug + Eq {}

trait Stack<C: Comparable, CH>: Default {
    fn add(&mut self, new: &C);
    fn remove(&mut self, old: &C);
    fn update(&mut self, old: &C, new: &C);
    fn add_child(&mut self, children: CH);
}

fn iter<S: Stack<C, CH>, C: Comparable, CH, F: FnMut(&C, &C) -> CH>(
    old: &BTreeMap<String, C>,
    new: &BTreeMap<String, C>,
    mut next: F,
) -> S {
    let mut stack = S::default();

    for (name, r) in new {
        match old.get(name) {
            Some(l) => {
                if r != l {
                    stack.update(l, r);
                }
                stack.add_child(next(l, r));
            }
            None => stack.add(r),
        }
    }

    for (name, l) in old {
        if !new.contains_key(name) {
            stack.remove(l);
        }
    }

    stack
}

#[derive(Default, Debug)]
pub struct Diff {
    schema: Schema,
}

impl Diff {
    pub fn from(old: &crate::inspect::Database, new: &crate::inspect::Database) -> Self {
        let schema = Self::database(&old, &new);

        Self { schema }
    }

    fn database(old: &crate::inspect::Database, new: &crate::inspect::Database) -> Schema {
        iter(&old.schemas, &new.schemas, |old, new| Self::schema(old, new))
    }

    fn schema(old: &crate::inspect::Schema, new: &crate::inspect::Schema) -> (Relation, Enum) {
        let relation = iter(&old.relations, &new.relations, |old, new| Self::relation(old, new));

        let r#enum = iter(&old.enums, &new.enums, |_, _| {});

        (relation, r#enum)
    }

    fn relation(old: &crate::inspect::Relation, new: &crate::inspect::Relation) -> Column {
        iter(&old.columns, &new.columns, |_, _| {})
    }

    pub fn sql(&self) -> crate::Result<String> {
        let mut s = String::new();
        self.schema.sql(&mut s)?;

        Ok(s)
    }
}

trait Sql {
    fn sql(&self, output: &mut dyn std::fmt::Write) -> crate::Result;
}

macro_rules! diff {
    ($ty:ident, $child:ty, $comparable:ty) => {
        impl Comparable for $comparable {}

        #[derive(Debug, Default)]
        struct $ty {
            added: Vec<$comparable>,
            updated: Vec<($comparable, $comparable)>,
            removed: Vec<$comparable>,
            children: Vec<$child>,
        }

        impl Stack<$comparable, $child> for $ty {
            fn add(&mut self, new: &$comparable) {
                self.added.push(new.clone());
            }

            fn remove(&mut self, old: &$comparable) {
                self.removed.push(old.clone());
            }

            fn update(&mut self, old: &$comparable, new: &$comparable) {
                self.updated.push((old.clone(), new.clone()));
            }

            fn add_child(&mut self, children: $child) {
                self.children.push(children);
            }
        }

        impl Sql for $ty {
            fn sql(&self, output: &mut dyn std::fmt::Write) -> crate::Result {
                for new in &self.added {
                    write!(output, "{}", self.sql_added(new))?;
                }

                for old in &self.removed {
                    write!(output, "{}", self.sql_removed(old))?;
                }

                for (old, new) in &self.updated {
                    write!(output, "{}", self.sql_updated(old, new))?;
                }

                for child in &self.children {
                    child.sql(output)?;
                }

                Ok(())
            }
        }
    };
}

impl Comparable for () {}

impl Sql for () {
    fn sql(&self, _: &mut dyn std::fmt::Write) -> crate::Result {
        Ok(())
    }
}

impl Stack<(), ()> for () {
    fn add(&mut self, _: &()) {}

    fn remove(&mut self, _: &()) {}

    fn update(&mut self, _: &(), _: &()) {}

    fn add_child(&mut self, _: ()) {}
}

diff!(Schema, (Relation, Enum), crate::inspect::Schema);

impl Schema {
    fn sql_added(&self, new: &crate::inspect::Schema) -> String {
        let mut sql = format!("create schema {};\n", new.fullname());

        let comment = comment("schema", &new.fullname(), None, new.comment.as_deref());
        sql.push_str(&comment);

        sql
    }

    fn sql_removed(&self, old: &crate::inspect::Schema) -> String {
        format!("drop schema {};\n", old.fullname())
    }

    fn sql_updated(&self, old: &crate::inspect::Schema, new: &crate::inspect::Schema) -> String {
        comment("schema", &old.fullname(), old.comment.as_deref(), new.comment.as_deref())
}

impl Sql for &(Relation, Enum) {
    fn sql(&self, output: &mut dyn std::fmt::Write) -> crate::Result {
        self.0.sql(output)?;
        self.1.sql(output)?;

        Ok(())
    }
}

diff!(Relation, Column, crate::inspect::Relation);

impl Relation {
    fn sql_added(&self, new: &crate::inspect::Relation) -> String {
        let mut sql = format!("create table {}(", new.fullname());

        for column in new.columns.values() {
            sql.push_str(&format!("\n    {} {}", column.name, column.ty));
            if column.is_primary {
                sql.push_str(" primary key");
            }
            sql.push_str(",");
        }

        sql = sql.trim_end_matches(',').to_string();

        sql.push_str("\n);\n");

        let comment = comment("table", &new.fullname(), None, new.comment.as_deref());
        sql.push_str(&comment);

        sql
    }

    fn sql_removed(&self, old: &crate::inspect::Relation) -> String {
        format!("drop table {};\n", old.fullname())
    }

    fn sql_updated(
        &self,
        old: &crate::inspect::Relation,
        new: &crate::inspect::Relation,
    ) -> String {
        comment("table", &old.fullname(), old.comment.as_deref(), new.comment.as_deref())
    }
}

diff!(Enum, (), crate::inspect::Enum);

impl Enum {
    fn sql_added(&self, new: &crate::inspect::Enum) -> String {
        let elements = new.elements.iter().map(|x| format!("'{x}'")).collect::<Vec<_>>().join(", ");

        format!("create type \"{}\" as enum({elements});\n", new.fullname())
    }

    fn sql_removed(&self, old: &crate::inspect::Enum) -> String {
        format!("drop type \"{}\";\n", old.fullname())
    }

    fn sql_updated(
        &self,
        old: &crate::inspect::Enum,
        new: &crate::inspect::Enum,
    ) -> String {
        let mut sql = String::new();

        let old_elements = &old.elements;
        let new_elements = &new.elements;

        for old_element in old_elements {
            if !new_elements.contains(&old_element) {
                sql.push_str(&format!("alter type \"{}\" drop attribute '{old_element}';\n", new.fullname()));
            }
        }

        for (x, new_element) in new_elements.iter().enumerate() {
            if !old_elements.contains(&new_element) {
                if let Some(after) = new_elements.get(x - 1) {
                    sql.push_str(&format!("alter type \"{}\" add value '{new_element}' after '{after}';\n", new.fullname()));
                } else if let Some(before) = new_elements.get(x + 1) {
                    sql.push_str(&format!("alter type \"{}\" add value '{new_element}' before '{before}';\n", new.fullname()));
                } else {
                    sql.push_str(&format!("alter type \"{}\" add value '{new_element}';\n", new.fullname()));
                }
            }
        }

        sql
    }
}

diff!(Column, (), crate::inspect::Column);

impl Column {
    fn sql_added(&self, new: &crate::inspect::Column) -> String {
        let mut sql = format!(
            "alter table \"{}\" add column \"{}\" {};\n",
            new.parent.fullname(),
            new.name,
            new.ty
        );

        let comment = comment("column", &new.fullname(), None, new.comment.as_deref());
        sql.push_str(&comment);

        sql
    }

    fn sql_removed(&self, old: &crate::inspect::Column) -> String {
        format!(
            "alter table \"{}\" drop column \"{}\";\n",
            old.parent.fullname(),
            old.name
        )
    }

    fn sql_updated(&self, old: &crate::inspect::Column, new: &crate::inspect::Column) -> String {
        let mut sql = match (&old.default, &new.default) {
            (_, Some(default)) => format!(
                "alter table \"{}\" alter column \"{}\" set default '{default}';\n",
                old.parent.fullname(),
                old.name,
            ),
            (Some(_), None) => format!(
                "alter table \"{}\" alter column \"{}\" drop default;\n",
                old.parent.fullname(),
                old.name
            ),
            (None, None) => String::new(),
        };

        let comment = comment("column", &old.fullname(), old.comment.as_deref(), new.comment.as_deref());
        sql.push_str(&comment);

        if old.ty != new.ty {
            sql.push_str(&format!(
                "alter table \"{}\" alter column \"{}\" type {};\n",
                old.parent.fullname(),
                old.name,
                new.ty
            ));
        }

        sql
    }
}

fn comment(ty: &str, fullname: &str, old: Option<&str>, new: Option<&str>) -> String {
    if old == new {
        return String::new();
    }

    match (&old, &new) {
        (_, Some(comment)) => format!("comment on {ty} {fullname} is '{comment}';\n"),
        (Some(_), None) => format!("comment on {ty} {fullname} is null;\n"),
        _ => String::new(),
    }
}
