use derive_deref_rs::Deref;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Database {
    pub schemas: BTreeMap<String, Schema>,
}

impl Database {
    pub fn new(dsn: &str) -> crate::Result<Self> {
        let conn = elephantry::Connection::new(dsn)?;
        let schemas = elephantry::inspect::database(&conn)?
            .iter()
            .map(|x| Ok((x.name.clone(), Schema::new(x, &conn)?)))
            .collect::<crate::Result<BTreeMap<String, Schema>>>()?;

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
    pub functions: BTreeMap<String, Function>,
    pub triggers: BTreeMap<String, Trigger>,
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
            functions: BTreeMap::new(),
            triggers: BTreeMap::new(),
        };

        schema.relations = elephantry::inspect::schema(conn, &schema.name)?
            .iter()
            .map(|x| {
                Ok((
                    format!("{}.{}", schema.name, x.name),
                    Relation::new(x, conn)?,
                ))
            })
            .collect::<crate::Result<_>>()?;

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

        schema.functions = elephantry::inspect::functions(conn, &schema.name)?
            .iter()
            .map(|x| {
                (
                    format!("{}.{}", schema.name, x.name),
                    Function::new(&schema, x),
                )
            })
            .collect();

        schema.triggers = elephantry::inspect::triggers(conn, &schema.name)?
            .iter()
            .map(|x| {
                (
                    format!("{}.{}", schema.name, x.name),
                    Trigger::new(&schema, x),
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
    pub columns: BTreeMap<String, Column>,
    pub constraints: BTreeMap<String, Constraint>,
    pub indexes: BTreeMap<String, Index>,
}

impl Relation {
    fn new(
        relation: &elephantry::inspect::Relation,
        conn: &elephantry::Connection,
    ) -> crate::Result<Self> {
        let mut relation = Self {
            inner: relation.clone(),
            columns: BTreeMap::new(),
            constraints: BTreeMap::new(),
            indexes: BTreeMap::new(),
        };

        relation.columns = elephantry::inspect::relation(conn, &relation.schema, &relation.name)?
            .iter()
            .map(|x| {
                (
                    format!("{}.{}.{}", relation.schema, relation.name, x.name),
                    Column::new(&relation, x),
                )
            })
            .collect();

        relation.constraints = elephantry::inspect::constraints(conn, relation.oid)?
            .iter()
            .map(|x| {
                (
                    format!("{}.{}", relation.fullname(), x.name),
                    Constraint::new(&relation.kind.to_string(), &relation.fullname(), x),
                )
            })
            .collect();

        relation.indexes = elephantry::inspect::indexes(conn, &relation)?
            .iter()
            .map(|x| {
                (
                    format!("{}.{}", relation.fullname(), x.name),
                    Index::new(&relation, x),
                )
            })
            .collect();

        Ok(relation)
    }

    pub fn fullname(&self) -> String {
        format!("\"{}\".\"{}\"", self.schema, self.name)
    }
}

impl PartialEq for Relation {
    fn eq(&self, other: &Self) -> bool {
        self.inner.kind == other.inner.kind
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
        format!("\"{}\".\"{}\"", self.parent.name, self.name)
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
    pub constraints: BTreeMap<String, Constraint>,
}

impl Domain {
    fn new(schema: &Schema, domain: &elephantry::inspect::Domain) -> Self {
        let mut d = Self {
            parent: schema.clone(),
            inner: domain.clone(),
            constraints: BTreeMap::new(),
        };

        d.constraints = domain
            .constraints
            .iter()
            .map(|x| (x.name.clone(), Constraint::new("domain", &d.fullname(), x)))
            .collect();

        d
    }

    pub fn fullname(&self) -> String {
        format!("\"{}\".\"{}\"", self.parent.name, self.name)
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
        format!("\"{}\".\"{}\"", self.parent.name, self.name)
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
        format!("{}.\"{}\"", self.parent.fullname(), self.name)
    }
}

impl PartialEq for Column {
    fn eq(&self, other: &Self) -> bool {
        self.inner.is_primary == other.inner.is_primary
            && self.inner.name == other.inner.name
            && self.inner.ty() == other.inner.ty()
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
        format!("\"{}\".\"{}\"", self.parent.name, self.name)
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
pub struct Function {
    #[deref]
    pub inner: elephantry::inspect::Function,
    pub parent: Schema,
}

impl Function {
    fn new(schema: &Schema, function: &elephantry::inspect::Function) -> Self {
        Self {
            parent: schema.clone(),
            inner: function.clone(),
        }
    }

    pub fn fullname(&self) -> String {
        format!("\"{}\".\"{}\"", self.parent.name, self.name)
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.inner.name == other.inner.name
            && self.inner.language == other.inner.language
            && self.inner.definition == other.inner.definition
            && self.inner.arguments == other.inner.arguments
            && self.inner.return_type == other.inner.return_type
    }
}

#[derive(Clone, Debug, Deref, Eq, PartialEq)]
pub struct Trigger {
    #[deref]
    inner: elephantry::inspect::Trigger,
    pub parent: Schema,
}

impl Trigger {
    fn new(schema: &Schema, trigger: &elephantry::inspect::Trigger) -> Self {
        Self {
            parent: schema.clone(),
            inner: trigger.clone(),
        }
    }

    pub fn fullname(&self) -> String {
        format!("\"{}\".\"{}\"", self.parent.name, self.name)
    }
}

#[derive(Clone, Debug, Deref, Eq)]
pub struct Constraint {
    #[deref]
    inner: elephantry::inspect::Constraint,
    pub parent_name: String,
    pub parent_type: String,
}

impl Constraint {
    fn new(
        parent_type: &str,
        parent_name: &str,
        constraint: &elephantry::inspect::Constraint,
    ) -> Self {
        Self {
            parent_name: parent_name.to_string(),
            parent_type: parent_type.to_string(),
            inner: constraint.clone(),
        }
    }

    pub fn fullname(&self) -> String {
        format!("\"{}\".\"{}\"", self.parent_name, self.name)
    }
}

impl PartialEq for Constraint {
    fn eq(&self, other: &Self) -> bool {
        self.inner.name == other.inner.name && self.inner.definition == other.inner.definition
    }
}

#[derive(Clone, Debug, Deref, Eq)]
pub struct Index {
    #[deref]
    inner: elephantry::inspect::Index,
    pub parent: Relation,
}

impl Index {
    fn new(relation: &Relation, index: &elephantry::inspect::Index) -> Self {
        Self {
            parent: relation.clone(),
            inner: index.clone(),
        }
    }

    pub fn fullname(&self) -> String {
        self.name.clone()
    }
}

impl PartialEq for Index {
    fn eq(&self, other: &Self) -> bool {
        self.inner.name == other.inner.name && self.inner.definition == other.inner.definition
    }
}
