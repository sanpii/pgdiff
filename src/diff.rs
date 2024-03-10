use std::collections::BTreeMap;

trait Comparable: std::fmt::Debug + Eq {}

trait Stack<C: Comparable, CH>: Default {
    fn add(&mut self, new: &C);
    fn remove(&mut self, old: &C);
    fn update(&mut self, old: &C, new: &C);
    fn add_child(&mut self, children: CH);
    fn is_empty(&self) -> bool;
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
        let schema = Self::database(old, new);

        Self { schema }
    }

    fn database(old: &crate::inspect::Database, new: &crate::inspect::Database) -> Schema {
        iter(&old.schemas, &new.schemas, |old, new| {
            Self::schema(old, new)
        })
    }

    fn schema(old: &crate::inspect::Schema, new: &crate::inspect::Schema) -> SchemaComponents {
        let relation = iter(&old.relations, &new.relations, |old, new| {
            if old.kind == elephantry::inspect::Kind::OrdinaryTable {
                Self::relation(old, new)
            } else {
                RelationComponents::default()
            }
        });
        let r#enum = iter(&old.enums, &new.enums, |_, _| {});
        let domain = iter(&old.domains, &new.domains, |old, new| {
            Self::constraint(old, new)
        });
        let composite = iter(&old.composites, &new.composites, |_, _| {});
        let extension = iter(&old.extensions, &new.extensions, |_, _| {});
        let function = iter(&old.functions, &new.functions, |_, _| {});
        let trigger = iter(&old.triggers, &new.triggers, |_, _| {});

        SchemaComponents {
            relation,
            r#enum,
            domain,
            composite,
            extension,
            function,
            trigger,
        }
    }

    fn relation(
        old: &crate::inspect::Relation,
        new: &crate::inspect::Relation,
    ) -> RelationComponents {
        let column = iter(&old.columns, &new.columns, |_, _| {});
        let constraint = iter(&old.constraints, &new.constraints, |_, _| {});
        let index = iter(&old.indexes, &new.indexes, |_, _| {});

        RelationComponents {
            column,
            constraint,
            index,
        }
    }

    fn constraint(old: &crate::inspect::Domain, new: &crate::inspect::Domain) -> Constraint {
        iter(&old.constraints, &new.constraints, |_, _| {})
    }

    pub fn sql(&self) -> String {
        let mut s = String::new();

        s.push_str("begin;\n\n");
        self.schema.sql(&mut s);
        s.push_str("commit;\n");

        s
    }
}

trait Sql {
    fn sql(&self, output: &mut dyn std::fmt::Write);
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

            fn is_empty(&self) -> bool {
                self.added.is_empty()
                    && self.updated.is_empty()
                    && self.removed.is_empty()
            }
        }

        impl Sql for $ty {
            fn sql(&self, output: &mut dyn std::fmt::Write) {
                if !self.is_empty() {
                    write!(output, "--\n-- {}\n--\n", stringify!($ty)).ok();
                }

                for new in &self.added {
                    output.write_str(&self.sql_added(new)).ok();
                }

                for old in &self.removed {
                    output.write_str(&self.sql_removed(old)).ok();
                }

                for (old, new) in &self.updated {
                    output.write_str(&self.sql_updated(old, new)).ok();
                }

                for child in &self.children {
                    child.sql(output);
                }

                if !self.is_empty() {
                    output.write_str("\n").ok();
                }
            }
        }
    };
}

impl Comparable for () {}

impl Sql for () {
    fn sql(&self, _: &mut dyn std::fmt::Write) {}
}

impl Stack<(), ()> for () {
    fn add(&mut self, _: &()) {}

    fn remove(&mut self, _: &()) {}

    fn update(&mut self, _: &(), _: &()) {}

    fn add_child(&mut self, _: ()) {}

    fn is_empty(&self) -> bool { true }
}

diff!(Schema, SchemaComponents, crate::inspect::Schema);

impl Schema {
    fn sql_added(&self, new: &crate::inspect::Schema) -> String {
        let mut sql = format!("create schema {};\n", new.fullname());

        let comment = comment("schema", &new.fullname(), None, Some(&new.comment));
        sql.push_str(&comment);

        sql
    }

    fn sql_removed(&self, old: &crate::inspect::Schema) -> String {
        format!("drop schema {};\n", old.fullname())
    }

    fn sql_updated(&self, old: &crate::inspect::Schema, new: &crate::inspect::Schema) -> String {
        comment(
            "schema",
            &old.fullname(),
            Some(&old.comment),
            Some(&new.comment),
        )
    }
}

#[derive(Debug)]
struct SchemaComponents {
    relation: Relation,
    r#enum: Enum,
    domain: Domain,
    composite: Composite,
    extension: Extension,
    function: Function,
    trigger: Trigger,
}

impl Sql for &SchemaComponents {
    fn sql(&self, output: &mut dyn std::fmt::Write) {
        self.relation.sql(output);
        self.r#enum.sql(output);
        self.domain.sql(output);
        self.composite.sql(output);
        self.extension.sql(output);
        self.function.sql(output);
        self.trigger.sql(output);
    }
}

#[derive(Debug, Default)]
struct RelationComponents {
    column: Column,
    constraint: Constraint,
    index: Index,
}

impl Sql for &RelationComponents {
    fn sql(&self, output: &mut dyn std::fmt::Write) {
        self.column.sql(output);
        self.constraint.sql(output);
        self.index.sql(output);
    }
}

diff!(Relation, RelationComponents, crate::inspect::Relation);

impl Relation {
    fn sql_added(&self, new: &crate::inspect::Relation) -> String {
        use elephantry::inspect::Kind::*;

        match new.kind {
            OrdinaryTable => self.create_table(new),
            View | MaterializedView => self.create_view(new),
            _ => String::new(),
        }
    }

    fn create_table(&self, new: &crate::inspect::Relation) -> String {
        use elephantry::inspect::Persistence;

        let mut sql = String::from("create");

        match new.persistence {
            Persistence::Permanent => (),
            Persistence::Unlogged => sql.push_str(" unlogged"),
            Persistence::Temporary => sql.push_str(" temporary"),
        }

        sql.push_str(&format!(" table {}(", new.fullname()));

        for column in new.columns.values() {
            sql.push_str(&format!("\n    {} {}", column.name, column.ty()));
            if column.is_primary {
                sql.push_str(" primary key");
            }
            sql.push(',');
        }

        sql = sql.trim_end_matches(',').to_string();

        sql.push_str("\n);\n");

        let comment = comment("table", &new.fullname(), None, new.comment.as_deref());
        sql.push_str(&comment);

        sql
    }

    fn create_view(&self, new: &crate::inspect::Relation) -> String {
        if let Some(definition) = &new.definition {
            format!("create {} {} as {definition}\n", new.kind, new.fullname())
        } else {
            String::new()
        }
    }

    fn sql_removed(&self, old: &crate::inspect::Relation) -> String {
        format!("drop {} {};\n", old.kind, old.fullname())
    }

    fn sql_updated(
        &self,
        old: &crate::inspect::Relation,
        new: &crate::inspect::Relation,
    ) -> String {
        let mut sql = String::new();

        if old.kind == elephantry::inspect::Kind::View {
            sql.push_str(&self.sql_removed(old));
            if let Some(definition) = &new.definition {
                sql.push_str(&format!("create view {} as {definition}\n", old.fullname(),));
            }
        }

        sql.push_str(&comment(
            &old.kind.to_string(),
            &old.fullname(),
            old.comment.as_deref(),
            new.comment.as_deref(),
        ));

        sql
    }
}

diff!(Enum, (), crate::inspect::Enum);

impl Enum {
    fn sql_added(&self, new: &crate::inspect::Enum) -> String {
        let elements = new
            .elements
            .iter()
            .map(|x| format!("'{x}'"))
            .collect::<Vec<_>>()
            .join(", ");

        format!("create type {} as enum({elements});\n", new.fullname())
    }

    fn sql_removed(&self, old: &crate::inspect::Enum) -> String {
        format!("drop type {};\n", old.fullname())
    }

    fn sql_updated(&self, old: &crate::inspect::Enum, new: &crate::inspect::Enum) -> String {
        let mut sql = String::new();

        let old_elements = &old.elements;
        let new_elements = &new.elements;

        for old_element in old_elements {
            if !new_elements.contains(old_element) {
                sql.push_str(&format!(
                    "delete from pg_enum e using pg_type t, pg_namespace n where e.enumtypid = t.oid and t.typname = '{}' and t.typnamespace = n.oid and n.nspname = '{}' and enumlabel = '{old_element}';\n",
                    new.name,
                    new.parent.name,
                ));
            }
        }

        for (x, new_element) in new_elements.iter().enumerate() {
            if !old_elements.contains(new_element) {
                if let Some(after) = new_elements.get(x - 1) {
                    sql.push_str(&format!(
                        "alter type {} add value '{new_element}' after '{after}';\n",
                        new.fullname()
                    ));
                } else if let Some(before) = new_elements.get(x + 1) {
                    sql.push_str(&format!(
                        "alter type {} add value '{new_element}' before '{before}';\n",
                        new.fullname()
                    ));
                } else {
                    sql.push_str(&format!(
                        "alter type {} add value '{new_element}';\n",
                        new.fullname()
                    ));
                }
            }
        }

        sql
    }
}

diff!(Domain, Constraint, crate::inspect::Domain);

impl Domain {
    fn sql_added(&self, new: &crate::inspect::Domain) -> String {
        let mut sql = format!("create domain {} as {}", new.fullname(), new.ty);

        for constraint in new.constraints.values() {
            sql.push_str(&format!(" {}", constraint.definition));
        }

        sql.push_str(";\n");

        sql
    }

    fn sql_removed(&self, old: &crate::inspect::Domain) -> String {
        format!("drop domain {};\n", old.fullname())
    }

    fn sql_updated(&self, old: &crate::inspect::Domain, new: &crate::inspect::Domain) -> String {
        let mut sql = String::new();

        if old.is_notnull != new.is_notnull {
            if new.is_notnull {
                sql.push_str(&format!("alter domain {} set not null;\n", new.fullname()));
            } else {
                sql.push_str(&format!("alter domain {} drop not null;\n", new.fullname()));
            }
        }

        match (&old.default, &new.default) {
            (None, None) => (),
            (_, Some(default)) => sql.push_str(&format!(
                "alter domain {} set default {default};\n",
                new.fullname()
            )),
            (Some(_), None) => {
                sql.push_str(&format!("alter domain {} drop default;\n", new.fullname()))
            }
        }

        sql
    }
}

diff!(Composite, (), crate::inspect::Composite);

impl Composite {
    fn sql_added(&self, new: &crate::inspect::Composite) -> String {
        let mut sql = String::new();

        sql.push_str(&format!("create type {} as (\n", new.fullname()));

        for field in &new.fields {
            sql.push_str(&format!("    {} {},\n", field.name, field.ty()));
        }

        sql = sql.trim_end_matches(",\n").to_string();
        sql.push_str("\n);\n");

        sql
    }

    fn sql_removed(&self, old: &crate::inspect::Composite) -> String {
        format!("drop type {};\n", old.fullname())
    }

    fn sql_updated(
        &self,
        old: &crate::inspect::Composite,
        new: &crate::inspect::Composite,
    ) -> String {
        let mut sql = String::new();

        sql.push_str(&self.sql_removed(old));
        sql.push_str(&self.sql_added(new));

        sql
    }
}

diff!(Column, (), crate::inspect::Column);

impl Column {
    fn sql_added(&self, new: &crate::inspect::Column) -> String {
        let mut sql = format!(
            "alter table {} add column \"{}\" {};\n",
            new.parent.fullname(),
            new.name,
            new.ty()
        );

        let comment = comment("column", &new.fullname(), None, new.comment.as_deref());
        sql.push_str(&comment);

        sql
    }

    fn sql_removed(&self, old: &crate::inspect::Column) -> String {
        format!(
            "alter table {} drop column \"{}\";\n",
            old.parent.fullname(),
            old.name
        )
    }

    fn sql_updated(&self, old: &crate::inspect::Column, new: &crate::inspect::Column) -> String {
        let mut sql = match (&old.default, &new.default) {
            (_, Some(default)) => format!(
                "alter table {} alter column \"{}\" set default {default};\n",
                old.parent.fullname(),
                old.name,
            ),
            (Some(_), None) => format!(
                "alter table {} alter column \"{}\" drop default;\n",
                old.parent.fullname(),
                old.name
            ),
            (None, None) => String::new(),
        };

        let comment = comment(
            "column",
            &old.fullname(),
            old.comment.as_deref(),
            new.comment.as_deref(),
        );
        sql.push_str(&comment);

        if old.is_notnull != new.is_notnull {
            if new.is_notnull {
                sql.push_str(&format!(
                    "alter table {} alter column \"{}\" set not null;\n",
                    old.parent.fullname(),
                    old.name
                ));
            } else {
                sql.push_str(&format!(
                    "alter table {} alter column \"{}\" drop not null;\n",
                    old.parent.fullname(),
                    old.name
                ));
            }
        }

        if old.ty() != new.ty() {
            sql.push_str(&format!(
                "alter table {} alter column \"{}\" type {} using \"{}\"::{};\n",
                old.parent.fullname(),
                old.name,
                new.ty(),
                old.name,
                new.ty(),
            ));
        }

        sql
    }
}

diff!(Extension, (), crate::inspect::Extension);

impl Extension {
    fn sql_added(&self, new: &crate::inspect::Extension) -> String {
        format!("create extension \"{}\";\n", new.name)
    }

    fn sql_removed(&self, old: &crate::inspect::Extension) -> String {
        format!("drop extension \"{}\";\n", old.name)
    }

    fn sql_updated(
        &self,
        old: &crate::inspect::Extension,
        new: &crate::inspect::Extension,
    ) -> String {
        format!(
            "alter extension \"{}\" update to '{}';\n",
            old.name, new.version
        )
    }
}

diff!(Function, (), crate::inspect::Function);

impl Function {
    fn sql_added(&self, new: &crate::inspect::Function) -> String {
        format!("{};\n", new.definition.trim_end_matches('\n'))
    }

    fn sql_removed(&self, old: &crate::inspect::Function) -> String {
        format!("drop function {};\n", old.fullname())
    }

    fn sql_updated(
        &self,
        old: &crate::inspect::Function,
        new: &crate::inspect::Function,
    ) -> String {
        let mut sql = String::new();

        sql.push_str(&self.sql_removed(old));
        sql.push_str(&self.sql_added(new));

        sql
    }
}

diff!(Trigger, (), crate::inspect::Trigger);

impl Trigger {
    fn sql_added(&self, new: &crate::inspect::Trigger) -> String {
        format!(
            "create or replace trigger \"{}\" {} {} on \"{}\".\"{}\" for each {} {};\n",
            new.name,
            new.timing,
            new.event,
            new.parent.fullname(),
            new.table,
            new.orientation,
            new.action,
        )
    }

    fn sql_removed(&self, old: &crate::inspect::Trigger) -> String {
        format!(
            "drop trigger \"{}\" on \"{}\".\"{}\";\n",
            old.name,
            old.parent.fullname(),
            old.table
        )
    }

    fn sql_updated(&self, _: &crate::inspect::Trigger, new: &crate::inspect::Trigger) -> String {
        self.sql_added(new)
    }
}

diff!(Constraint, (), crate::inspect::Constraint);

impl Constraint {
    fn sql_added(&self, new: &crate::inspect::Constraint) -> String {
        format!(
            "alter {} {} add constraint \"{}\" {};\n",
            new.parent_type, new.parent_name, new.name, new.definition
        )
    }

    fn sql_removed(&self, old: &crate::inspect::Constraint) -> String {
        format!(
            "alter {} {} drop constraint \"{}\";\n",
            old.parent_type, old.parent_name, old.name
        )
    }

    fn sql_updated(
        &self,
        old: &crate::inspect::Constraint,
        new: &crate::inspect::Constraint,
    ) -> String {
        let mut sql = String::new();

        sql.push_str(&self.sql_removed(old));
        sql.push_str(&self.sql_added(new));

        sql
    }
}

diff!(Index, (), crate::inspect::Index);

impl Index {
    fn sql_added(&self, new: &crate::inspect::Index) -> String {
        format!("{};\n", new.definition)
    }

    fn sql_removed(&self, old: &crate::inspect::Index) -> String {
        format!("drop index {};\n", old.name)
    }

    fn sql_updated(&self, old: &crate::inspect::Index, new: &crate::inspect::Index) -> String {
        let mut sql = String::new();

        sql.push_str(&self.sql_removed(old));
        sql.push_str(&self.sql_added(new));

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
