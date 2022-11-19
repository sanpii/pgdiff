use std::collections::HashMap;

#[derive(Debug)]
pub struct Database {
    pub schemas: HashMap<String, Schema>,
}

impl Database {
    pub fn new(dsn: &str) -> crate::Result<Self> {
        let conn = elephantry::Connection::new(dsn)?;
        let schemas = elephantry::inspect::database(&conn)?
            .iter()
            .map(|x| (x.name.clone(), Schema::new(x, &conn).unwrap()))
            .collect::<HashMap<String, Schema>>();

        Ok(Self { schemas })
    }
}

impl PartialEq for Database {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}

impl Eq for Database {}

#[derive(Clone, Debug)]
pub struct Schema {
    inner: elephantry::inspect::Schema,
    pub relations: HashMap<String, Relation>,
}

impl Schema {
    fn new(
        inner: &elephantry::inspect::Schema,
        conn: &elephantry::Connection,
    ) -> crate::Result<Self> {
        let mut schema = Self {
            inner: inner.clone(),
            relations: HashMap::new(),
        };

        schema.relations = elephantry::inspect::schema(conn, &schema.name)?
            .iter()
            .map(|x| {
                (
                    format!("{}.{}", schema.name, x.name),
                    Relation::new(&schema, x, conn).unwrap(),
                )
            })
            .collect();

        Ok(schema)
    }

    pub fn fullname(&self) -> String {
        self.name.clone()
    }
}

impl PartialEq for Schema {
    fn eq(&self, other: &Self) -> bool {
        self.inner.name == other.inner.name && self.inner.comment == other.inner.comment
    }
}

impl Eq for Schema {}

impl std::ops::Deref for Schema {
    type Target = elephantry::inspect::Schema;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Clone, Debug)]
pub struct Relation {
    inner: elephantry::inspect::Relation,
    parent: Schema,
    pub columns: HashMap<String, Column>,
}

impl Relation {
    fn new(
        schema: &Schema,
        relation: &elephantry::inspect::Relation,
        conn: &elephantry::Connection,
    ) -> crate::Result<Self> {
        let mut relation = Self {
            parent: schema.clone(),
            inner: relation.clone(),
            columns: HashMap::new(),
        };

        relation.columns = elephantry::inspect::relation(conn, &schema.name, &relation.name)?
            .iter()
            .map(|x| {
                (
                    format!("{}.{}.{}", schema.name, relation.name, x.name),
                    Column::new(&relation, x),
                )
            })
            .collect();

        Ok(relation)
    }

    pub fn fullname(&self) -> String {
        format!("{}.{}", self.parent.name, self.name)
    }
}

impl PartialEq for Relation {
    fn eq(&self, other: &Self) -> bool {
        self.inner.ty == other.inner.ty
            && self.inner.name == other.inner.name
            && self.inner.comment == other.inner.comment
    }
}

impl Eq for Relation {}

impl std::ops::Deref for Relation {
    type Target = elephantry::inspect::Relation;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Clone, Debug)]
pub struct Column {
    inner: elephantry::inspect::Column,
    pub parent: Relation,
}

impl Column {
    fn new(relation: &Relation, column: &elephantry::inspect::Column) -> Self {
        Self {
            parent: relation.clone(),
            inner: column.clone(),
        }
    }

    pub fn fullname(&self) -> String {
        format!(
            "{}.{}.{}",
            self.parent.parent.name, self.parent.name, self.name
        )
    }
}

impl PartialEq for Column {
    fn eq(&self, other: &Self) -> bool {
        self.inner.is_primary == other.inner.is_primary
            && self.inner.name == other.inner.name
            && self.inner.ty == other.inner.ty
            && self.inner.default == other.inner.default
            && self.inner.is_notnull == other.inner.is_notnull
            && self.inner.comment == other.inner.comment
    }
}

impl Eq for Column {}

impl std::ops::Deref for Column {
    type Target = elephantry::inspect::Column;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
