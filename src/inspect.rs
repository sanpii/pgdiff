use derive_deref_rs::Deref;
use std::collections::BTreeMap;

#[derive(Debug, Eq, PartialEq)]
pub struct Database {
    pub schemas: BTreeMap<String, Schema>,
}

impl Database {
    pub fn new(dsn: &str) -> crate::Result<Self> {
        let conn = elephantry::Connection::new(dsn)?;
        let schemas = elephantry::inspect::database(&conn)?
            .iter()
            .map(|x| (x.name.clone(), Schema::new(x, &conn).unwrap()))
            .collect::<BTreeMap<String, Schema>>();

        Ok(Self { schemas })
    }
}

#[derive(Clone, Debug, Deref, Eq)]
pub struct Schema {
    #[deref]
    inner: elephantry::inspect::Schema,
    pub relations: BTreeMap<String, Relation>,
    pub enums: BTreeMap<String, Enum>,
    pub domains: BTreeMap<String, Domain>,
    pub composites: BTreeMap<String, Composite>,
    pub extensions: BTreeMap<String, Extension>,
}

impl Schema {
    fn new(
        inner: &elephantry::inspect::Schema,
        conn: &elephantry::Connection,
    ) -> crate::Result<Self> {
        let mut schema = Self {
            inner: inner.clone(),
            relations: BTreeMap::new(),
            enums: BTreeMap::new(),
            domains: BTreeMap::new(),
            composites: BTreeMap::new(),
            extensions: BTreeMap::new(),
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

        schema.enums = elephantry::inspect::enums(conn, &schema.name)?
            .iter()
            .map(|x| (format!("{}.{}", schema.name, x.name), Enum::new(&schema, x)))
            .collect();

        schema.domains = elephantry::inspect::domains(conn, &schema.name)?
            .iter()
            .map(|x| {
                (
                    format!("{}.{}", schema.name, x.name),
                    Domain::new(&schema, x),
                )
            })
            .collect();

        schema.composites = elephantry::inspect::composites(conn, &schema.name)?
            .iter()
            .map(|x| {
                (
                    format!("{}.{}", schema.name, x.name),
                    Composite::new(&schema, x),
                )
            })
            .collect();

        schema.extensions = elephantry::inspect::extensions(conn, &schema.name)?
            .iter()
            .map(|x| {
                (
                    format!("{}.{}", schema.name, x.name),
                    Extension::new(&schema, x),
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

#[derive(Clone, Debug, Deref, Eq)]
pub struct Relation {
    #[deref]
    inner: elephantry::inspect::Relation,
    parent: Schema,
    pub columns: BTreeMap<String, Column>,
    pub constraints: BTreeMap<String, Constraint>,
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
            columns: BTreeMap::new(),
            constraints: BTreeMap::new(),
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

        relation.constraints = elephantry::inspect::constraints(conn, relation.oid)?
            .iter()
            .map(|x| {
                (
                    format!("{}.{}", relation.fullname(), x.name),
                    Constraint::new(&relation, x),
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
            && self.inner.definition == other.inner.definition
    }
}

#[derive(Clone, Debug, Deref, Eq)]
pub struct Enum {
    #[deref]
    inner: elephantry::inspect::Enum,
    pub parent: Schema,
}

impl Enum {
    fn new(schema: &Schema, r#enum: &elephantry::inspect::Enum) -> Self {
        Self {
            parent: schema.clone(),
            inner: r#enum.clone(),
        }
    }

    pub fn fullname(&self) -> String {
        format!("{}.{}", self.parent.name, self.name)
    }
}

impl PartialEq for Enum {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

#[derive(Clone, Debug, Deref, Eq)]
pub struct Domain {
    #[deref]
    inner: elephantry::inspect::Domain,
    pub parent: Schema,
}

impl Domain {
    fn new(schema: &Schema, domain: &elephantry::inspect::Domain) -> Self {
        Self {
            parent: schema.clone(),
            inner: domain.clone(),
        }
    }

    pub fn fullname(&self) -> String {
        format!("{}.{}", self.parent.name, self.name)
    }
}

impl PartialEq for Domain {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

#[derive(Clone, Debug, Deref, Eq)]
pub struct Composite {
    #[deref]
    inner: elephantry::inspect::Composite,
    pub parent: Schema,
}

impl Composite {
    fn new(schema: &Schema, composite: &elephantry::inspect::Composite) -> Self {
        Self {
            parent: schema.clone(),
            inner: composite.clone(),
        }
    }

    pub fn fullname(&self) -> String {
        format!("{}.{}", self.parent.name, self.name)
    }
}

impl PartialEq for Composite {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

#[derive(Clone, Debug, Deref, Eq)]
pub struct Column {
    #[deref]
    inner: elephantry::inspect::Column,
    pub parent: Relation,
    pub constraints: BTreeMap<String, Constraint>,
}

impl Column {
    fn new(relation: &Relation, column: &elephantry::inspect::Column) -> Self {
        Self {
            parent: relation.clone(),
            inner: column.clone(),
            constraints: BTreeMap::new(),
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

#[derive(Clone, Debug, Deref, Eq)]
pub struct Extension {
    #[deref]
    inner: elephantry::inspect::Extension,
    pub parent: Schema,
}

impl Extension {
    fn new(schema: &Schema, extension: &elephantry::inspect::Extension) -> Self {
        Self {
            parent: schema.clone(),
            inner: extension.clone(),
        }
    }

    pub fn fullname(&self) -> String {
        format!("{}.{}", self.parent.name, self.name)
    }
}

impl PartialEq for Extension {
    fn eq(&self, other: &Self) -> bool {
        self.inner.name == other.inner.name
            && self.inner.version == other.inner.version
            && self.inner.description == other.inner.description
    }
}

#[derive(Clone, Debug, Deref, Eq)]
pub struct Constraint {
    #[deref]
    inner: elephantry::inspect::Constraint,
    pub parent: String,
}

impl Constraint {
    fn new(relation: &Relation, constraint: &elephantry::inspect::Constraint) -> Self {
        Self {
            parent: relation.name.clone(),
            inner: constraint.clone(),
        }
    }

    pub fn fullname(&self) -> String {
        format!("{}.{}", self.parent, self.name)
    }
}

impl PartialEq for Constraint {
    fn eq(&self, other: &Self) -> bool {
        self.inner.name == other.inner.name && self.inner.definition == other.inner.definition
    }
}
