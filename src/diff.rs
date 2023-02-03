use std::collections::HashMap;

trait Comparable: std::fmt::Debug + Eq {}

trait Stack<C: Comparable, CH>: Default {
    fn add(&mut self, new: &C);
    fn remove(&mut self, old: &C);
    fn update(&mut self, old: &C, new: &C);
    fn add_child(&mut self, children: CH);
    fn sql(&self, output: &mut dyn std::fmt::Write) -> crate::Result;
}

fn iter<S: Stack<C, CH>, C: Comparable, CH, F: FnMut(&C, &C) -> CH>(
    old: &HashMap<String, C>,
    new: &HashMap<String, C>,
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

    fn schema(old: &crate::inspect::Schema, new: &crate::inspect::Schema) -> Relation {
        iter(&old.relations, &new.relations, |old, new| {
            Self::relation(old, new)
        })
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

impl Stack<(), ()> for () {
    fn add(&mut self, _: &()) {}

    fn remove(&mut self, _: &()) {}

    fn update(&mut self, _: &(), _: &()) {}

    fn add_child(&mut self, _: ()) {}

    fn sql(&self, _: &mut dyn std::fmt::Write) -> crate::Result {
        Ok(())
    }
}

diff!(Schema, Relation, crate::inspect::Schema);

impl Schema {
    fn sql_added(&self, new: &crate::inspect::Schema) -> String {
        format!("create schema {};\n", new.fullname())
    }

    fn sql_removed(&self, old: &crate::inspect::Schema) -> String {
        format!("drop schema {};\n", old.fullname())
    }

    fn sql_updated(&self, old: &crate::inspect::Schema, new: &crate::inspect::Schema) -> String {
        match (&old.comment, &new.comment) {
            (_, Some(comment)) => format!("comment on schema {} is '{comment}';\n", old.fullname()),
            (Some(_), None) => format!("comment on schema {} is null;\n", old.fullname()),
            (None, None) => String::new(),
        }
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
        match (&old.comment, &new.comment) {
            (_, Some(comment)) => format!("comment on table {} is '{comment}';\n", old.fullname()),
            (Some(_), None) => format!("comment on table {} is null;\n", old.fullname()),
            (None, None) => String::new(),
        }
    }
}

diff!(Column, (), crate::inspect::Column);

impl Column {
    fn sql_added(&self, new: &crate::inspect::Column) -> String {
        format!(
            "alter table \"{}\" add column \"{}\" {};\n",
            new.parent.fullname(),
            new.name,
            new.ty
        )
    }

    fn sql_removed(&self, old: &crate::inspect::Column) -> String {
        format!(
            "alter table \"{}\" drop column \"{}\";\n",
            old.parent.fullname(),
            old.name
        )
    }

    fn sql_updated(&self, old: &crate::inspect::Column, new: &crate::inspect::Column) -> String {
        let mut s = match (&old.default, &new.default) {
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

        if old.ty != new.ty {
            s.push_str(&format!(
                "alter table \"{}\" alter column \"{}\" type {};\n",
                old.parent.fullname(),
                old.name,
                new.ty
            ));
        }

        s
    }
}
