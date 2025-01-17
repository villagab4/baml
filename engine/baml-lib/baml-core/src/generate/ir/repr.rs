use anyhow::{anyhow, bail, Result};
use either::Either;

use indexmap::IndexMap;
use internal_baml_parser_database::{
    walkers::{
        ClassWalker, ClientWalker, ConfigurationWalker, EnumValueWalker, EnumWalker, FieldWalker,
        FunctionWalker, VariantWalker,
    },
    ParserDatabase, RetryPolicyStrategy, ToStringAttributes, WithStaticRenames,
};
use internal_baml_schema_ast::ast::{self, FieldArity, WithName};

/// This class represents the intermediate representation of the BAML AST.
/// It is a representation of the BAML AST that is easier to work with than the
/// raw BAML AST, and should include all information necessary to generate
/// code in any target language.
#[derive(serde::Serialize)]
pub struct IntermediateRepr {
    enums: Vec<Node<Enum>>,
    classes: Vec<Node<Class>>,
    functions: Vec<Node<Function>>,
    clients: Vec<Node<Client>>,
    retry_policies: Vec<Node<RetryPolicy>>,
}

/// A generic walker. Only walkers instantiated with a concrete ID type (`I`) are useful.
#[derive(Clone, Copy)]
pub struct Walker<'db, I> {
    /// The parser database being traversed.
    pub db: &'db IntermediateRepr,
    /// The identifier of the focused element.
    pub item: I,
}

impl IntermediateRepr {
    pub fn walk_enums<'a>(&'a self) -> impl Iterator<Item = Walker<'a, &'a Node<Enum>>> {
        self.enums.iter().map(|e| Walker { db: self, item: e })
    }

    pub fn walk_classes<'a>(&'a self) -> impl Iterator<Item = Walker<'a, &'a Node<Class>>> {
        self.classes.iter().map(|e| Walker { db: self, item: e })
    }

    pub fn walk_functions<'a>(&'a self) -> impl Iterator<Item = Walker<'a, &'a Node<Function>>> {
        self.functions.iter().map(|e| Walker { db: self, item: e })
    }

    pub fn walk_clients<'a>(&'a self) -> impl Iterator<Item = Walker<'a, &'a Node<Client>>> {
        self.clients.iter().map(|e| Walker { db: self, item: e })
    }

    pub fn walk_retry_policies<'a>(
        &'a self,
    ) -> impl Iterator<Item = Walker<'a, &'a Node<RetryPolicy>>> {
        self.retry_policies
            .iter()
            .map(|e| Walker { db: self, item: e })
    }

    pub fn from_parser_database(db: &ParserDatabase) -> Result<IntermediateRepr> {
        let mut repr = IntermediateRepr {
            enums: db
                .walk_enums()
                .map(|e| e.node(db))
                .collect::<Result<Vec<_>>>()?,
            classes: db
                .walk_classes()
                .map(|e| e.node(db))
                .collect::<Result<Vec<_>>>()?,
            functions: db
                .walk_functions()
                .map(|e| e.node(db))
                .collect::<Result<Vec<_>>>()?,
            clients: db
                .walk_clients()
                .map(|e| e.node(db))
                .collect::<Result<Vec<_>>>()?,
            retry_policies: db
                .walk_retry_policies()
                .map(|e| WithRepr::<RetryPolicy>::node(&e, db))
                .collect::<Result<Vec<_>>>()?,
        };

        // Sort each item by name.
        repr.enums.sort_by(|a, b| a.elem.name.cmp(&b.elem.name));
        repr.classes.sort_by(|a, b| a.elem.name.cmp(&b.elem.name));
        repr.functions.sort_by(|a, b| a.elem.name.cmp(&b.elem.name));
        repr.clients.sort_by(|a, b| a.elem.name.cmp(&b.elem.name));
        repr.retry_policies
            .sort_by(|a, b| a.elem.name.0.cmp(&b.elem.name.0));

        Ok(repr)
    }
}

// TODO:
//
//   [x] clients - need to finish expressions
//   [x] metadata per node (attributes, spans, etc)
//           block-level attributes on enums, classes
//           field-level attributes on enum values, class fields
//           overrides can only exist in impls
//   [x] FieldArity (optional / required) needs to be handled
//   [x] other types of identifiers?
//   [ ] `baml update` needs to update lockfile right now
//          but baml CLI is installed globally
//   [ ] baml configuration - retry policies, generator, etc
//          [x] retry policies
//   [x] rename lockfile/mod.rs to ir/mod.rs
//   [x] wire Result<> type through, need this to be more sane

#[derive(Default, serde::Serialize)]
pub struct NodeAttributes {
    /// Map of attributes on the corresponding IR node.
    ///
    /// Some follow special conventions:
    ///
    ///   - @skip becomes ("skip", "")
    ///   - @alias(...) becomes ("alias", ...)
    ///   - @get(python code) becomes ("get/python", python code)
    #[serde(with = "indexmap::map::serde_seq")]
    meta: IndexMap<String, Expression>,

    /// Overrides for the specified AST node in a given implementation (which is keyed by FunctionId
    /// and ImplementationId). In .baml files these are represented in the implementation, but in the
    /// IR AST we attach them to the AST node so that all metadata associated with an IRnode can be
    /// accessed from that node, rather than through a different IR node.
    #[serde(with = "indexmap::map::serde_seq")]
    overrides: IndexMap<(FunctionId, ImplementationId), IndexMap<String, Expression>>,
}

impl NodeAttributes {
    pub fn get(&self, key: &str) -> Option<&Expression> {
        self.meta.get(key)
    }
}

fn to_ir_attributes(
    db: &ParserDatabase,
    maybe_ast_attributes: Option<&ToStringAttributes>,
) -> IndexMap<String, Expression> {
    let mut attributes = IndexMap::new();

    if let Some(ast_attributes) = maybe_ast_attributes {
        match ast_attributes {
            ToStringAttributes::Static(s) => {
                if s.skip().is_some() {
                    attributes.insert("skip".to_string(), Expression::String("".to_string()));
                }
                if let Some(v) = s.alias() {
                    attributes.insert("alias".to_string(), Expression::String(db[*v].to_string()));
                }
                for (&k, &v) in s.meta().into_iter() {
                    attributes.insert(db[k].to_string(), Expression::String(db[v].to_string()));
                }
            }
            ToStringAttributes::Dynamic(d) => {
                for (&lang, &lang_code) in d.code.iter() {
                    attributes.insert(
                        format!("get/{}", db[lang].to_string()),
                        Expression::String(db[lang_code].to_string()),
                    );
                }
            }
        }
    }

    attributes
}

/// Nodes allow attaching metadata to a given IR entity: attributes, source location, etc
#[derive(serde::Serialize)]
pub struct Node<T> {
    pub attributes: NodeAttributes,
    pub elem: T,
}

/// Implement this for every node in the IR AST, where T is the type of IR node
pub trait WithRepr<T> {
    /// Represents block or field attributes - @@ for enums and classes, @ for enum values and class fields
    fn attributes(&self, _: &ParserDatabase) -> NodeAttributes {
        NodeAttributes::default()
    }

    fn repr(&self, db: &ParserDatabase) -> Result<T>;

    fn node(&self, db: &ParserDatabase) -> Result<Node<T>> {
        Ok(Node {
            elem: self.repr(db)?,
            attributes: self.attributes(db),
        })
    }
}

/// FieldType represents the type of either a class field or a function arg.
#[derive(serde::Serialize)]
pub enum FieldType {
    Primitive(ast::TypeValue),
    Enum(EnumId),
    Class(ClassId),
    List(Box<FieldType>),
    Map(Box<FieldType>, Box<FieldType>),
    Union(Vec<FieldType>),
    Tuple(Vec<FieldType>),
    Optional(Box<FieldType>),
}

impl FieldType {
    fn with_arity(self, arity: &FieldArity) -> FieldType {
        match arity {
            FieldArity::Required => self,
            FieldArity::Optional => FieldType::Optional(Box::new(self)),
        }
    }
}

impl WithRepr<FieldType> for ast::FieldType {
    fn repr(&self, db: &ParserDatabase) -> Result<FieldType> {
        Ok(match self {
            ast::FieldType::Identifier(arity, idn) => (match idn {
                ast::Identifier::Primitive(t, ..) => FieldType::Primitive(*t),
                ast::Identifier::Local(name, _) => match db.find_type(idn) {
                    Some(Either::Left(_class_walker)) => Ok(FieldType::Class(name.clone())),
                    Some(Either::Right(_enum_walker)) => Ok(FieldType::Enum(name.clone())),
                    None => Err(anyhow!("Field type uses unresolvable local identifier")),
                }?,
                _ => bail!("Field type uses unsupported identifier type"),
            })
            .with_arity(arity),
            ast::FieldType::List(ft, dims, _) => {
                // NB: potential bug: this hands back a 1D list when dims == 0
                let mut repr = FieldType::List(Box::new(ft.repr(db)?));

                for _ in 1u32..*dims {
                    repr = FieldType::List(Box::new(repr));
                }

                repr
            }
            ast::FieldType::Dictionary(kv, _) => {
                // NB: we can't just unpack (*kv) into k, v because that would require a move/copy
                FieldType::Map(Box::new((*kv).0.repr(db)?), Box::new((*kv).1.repr(db)?))
            }
            ast::FieldType::Union(arity, t, _) => {
                // NB: preempt union flattening by mixing arity into union types
                let mut types = t.iter().map(|ft| ft.repr(db)).collect::<Result<Vec<_>>>()?;

                if arity.is_optional() {
                    types.push(FieldType::Primitive(ast::TypeValue::Null));
                }

                FieldType::Union(types)
            }
            ast::FieldType::Tuple(arity, t, _) => {
                FieldType::Tuple(t.iter().map(|ft| ft.repr(db)).collect::<Result<Vec<_>>>()?)
                    .with_arity(arity)
            }
        })
    }
}

#[derive(serde::Serialize)]
pub enum Identifier {
    /// Starts with env.*
    ENV(String),
    /// The path to a Local Identifer + the local identifer. Separated by '.'
    Ref(Vec<String>),
    /// A string without spaces or '.' Always starts with a letter. May contain numbers
    Local(String),
    /// Special types (always lowercase).
    Primitive(ast::TypeValue),
    /// A string without spaces, but contains '-'
    String(String),
}

#[derive(serde::Serialize)]
pub enum Expression {
    Identifier(Identifier),
    Numeric(String),
    String(String),
    RawString(String),
    List(Vec<Expression>),
    Map(Vec<(Expression, Expression)>),
}

impl WithRepr<Expression> for ast::Expression {
    fn repr(&self, db: &ParserDatabase) -> Result<Expression> {
        Ok(match self {
            ast::Expression::NumericValue(val, _) => Expression::Numeric(val.clone()),
            ast::Expression::StringValue(val, _) => Expression::String(val.clone()),
            ast::Expression::RawStringValue(val) => Expression::RawString(val.value().to_string()),
            ast::Expression::Identifier(idn) => match idn {
                ast::Identifier::ENV(k, _) => {
                    Ok(Expression::Identifier(Identifier::ENV(k.clone())))
                }
                ast::Identifier::String(s, _) => Ok(Expression::String(s.clone())),
                ast::Identifier::Local(l, _) => {
                    Ok(Expression::Identifier(Identifier::Local(l.clone())))
                }
                ast::Identifier::Ref(r, _) => {
                    Ok(Expression::Identifier(Identifier::Ref(r.path.clone())))
                }
                ast::Identifier::Primitive(p, _) => {
                    Ok(Expression::Identifier(Identifier::Primitive(*p)))
                }
                ast::Identifier::Invalid(_, _) => {
                    Err(anyhow!("Cannot represent an invalid parser-AST identifier"))
                }
            }?,
            ast::Expression::Array(arr, _) => {
                Expression::List(arr.iter().map(|e| e.repr(db)).collect::<Result<Vec<_>>>()?)
            }
            ast::Expression::Map(arr, _) => Expression::Map(
                arr.iter()
                    .map(|(k, v)| Ok((k.repr(db)?, v.repr(db)?)))
                    .collect::<Result<Vec<_>>>()?,
            ),
        })
    }
}

type EnumId = String;

#[derive(serde::Serialize)]
pub struct EnumValue(pub String);

#[derive(serde::Serialize)]
pub struct Enum {
    pub name: EnumId,
    pub values: Vec<Node<EnumValue>>,
}

impl WithRepr<EnumValue> for EnumValueWalker<'_> {
    fn attributes(&self, db: &ParserDatabase) -> NodeAttributes {
        let mut attributes = NodeAttributes::default();

        attributes.meta = to_ir_attributes(db, self.get_default_attributes());

        for r#fn in db.walk_functions() {
            for r#impl in r#fn.walk_variants() {
                let node_attributes = to_ir_attributes(db, self.get_override(&r#impl));
                // TODO
                if !node_attributes.is_empty() {
                    attributes.overrides.insert(
                        (r#fn.name().to_string(), r#impl.name().to_string()),
                        node_attributes,
                    );
                }
            }
        }

        attributes
    }

    fn repr(&self, _db: &ParserDatabase) -> Result<EnumValue> {
        Ok(EnumValue(self.name().to_string()))
    }
}

impl WithRepr<Enum> for EnumWalker<'_> {
    fn attributes(&self, db: &ParserDatabase) -> NodeAttributes {
        let mut attributes = NodeAttributes::default();

        attributes.meta = to_ir_attributes(db, self.get_default_attributes());

        for r#fn in db.walk_functions() {
            for r#impl in r#fn.walk_variants() {
                let node_attributes =
                    to_ir_attributes(db, r#impl.find_serializer_attributes(self.name()));
                if !node_attributes.is_empty() {
                    attributes.overrides.insert(
                        (r#fn.name().to_string(), r#impl.name().to_string()),
                        node_attributes,
                    );
                }
            }
        }

        attributes
    }

    fn repr(&self, db: &ParserDatabase) -> Result<Enum> {
        Ok(Enum {
            name: self.name().to_string(),
            values: self
                .values()
                .map(|v| v.node(db))
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

#[derive(serde::Serialize)]
pub struct Field {
    pub name: String,
    pub r#type: Node<FieldType>,
}

impl WithRepr<Field> for FieldWalker<'_> {
    fn attributes(&self, db: &ParserDatabase) -> NodeAttributes {
        let mut attributes = NodeAttributes::default();

        attributes.meta = to_ir_attributes(db, self.get_default_attributes());

        for r#fn in db.walk_functions() {
            for r#impl in r#fn.walk_variants() {
                let node_attributes = to_ir_attributes(db, self.get_override(&r#impl));
                if !node_attributes.is_empty() {
                    attributes.overrides.insert(
                        (r#fn.name().to_string(), r#impl.name().to_string()),
                        node_attributes,
                    );
                }
            }
        }

        attributes
    }

    fn repr(&self, db: &ParserDatabase) -> Result<Field> {
        Ok(Field {
            name: self.name().to_string(),
            r#type: self.ast_field().field_type.node(db)?,
        })
    }
}

type ClassId = String;

#[derive(serde::Serialize)]
pub struct Class {
    pub name: ClassId,
    pub static_fields: Vec<Node<Field>>,
    pub dynamic_fields: Vec<Node<Field>>,
}

impl WithRepr<Class> for ClassWalker<'_> {
    fn attributes(&self, db: &ParserDatabase) -> NodeAttributes {
        let mut attributes = NodeAttributes::default();

        attributes.meta = to_ir_attributes(db, self.get_default_attributes());

        for r#fn in db.walk_functions() {
            for r#impl in r#fn.walk_variants() {
                let node_attributes =
                    to_ir_attributes(db, r#impl.find_serializer_attributes(self.name()));
                if !node_attributes.is_empty() {
                    attributes.overrides.insert(
                        (r#fn.name().to_string(), r#impl.name().to_string()),
                        node_attributes,
                    );
                }
            }
        }

        attributes
    }

    fn repr(&self, db: &ParserDatabase) -> Result<Class> {
        Ok(Class {
            name: self.name().to_string(),
            static_fields: self
                .static_fields()
                .map(|e| e.node(db))
                .collect::<Result<Vec<_>>>()?,
            dynamic_fields: self
                .dynamic_fields()
                .map(|e| e.node(db))
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

#[derive(serde::Serialize)]
pub enum OracleType {
    LLM,
}

type ImplementationId = String;

#[derive(serde::Serialize)]
pub struct Implementation {
    r#type: OracleType,
    pub name: ImplementationId,

    pub prompt: String,

    #[serde(with = "indexmap::map::serde_seq")]
    pub input_replacers: IndexMap<String, String>,

    #[serde(with = "indexmap::map::serde_seq")]
    pub output_replacers: IndexMap<String, String>,

    pub client: ClientId,
}

/// BAML does not allow UnnamedArgList nor a lone NamedArg
#[derive(serde::Serialize)]
pub enum FunctionArgs {
    UnnamedArg(FieldType),
    NamedArgList(Vec<(String, FieldType)>),
}

type FunctionId = String;

#[derive(serde::Serialize)]
pub struct Function {
    pub name: FunctionId,
    pub inputs: FunctionArgs,
    pub output: Node<FieldType>,
    pub impls: Vec<Node<Implementation>>,
    pub default_impl: Option<ImplementationId>,
}

impl WithRepr<Implementation> for VariantWalker<'_> {
    fn attributes(&self, _db: &ParserDatabase) -> NodeAttributes {
        NodeAttributes::default()
    }

    fn repr(&self, _db: &ParserDatabase) -> Result<Implementation> {
        Ok(Implementation {
            r#type: OracleType::LLM,
            name: self.name().to_string(),
            prompt: self.properties().prompt.value.clone(),
            input_replacers: self
                .properties()
                .replacers
                // NB: .0 should really be .input
                .0
                .iter()
                .map(|r| (r.0.key(), r.1.clone()))
                .collect(),
            output_replacers: self
                .properties()
                .replacers
                // NB: .1 should really be .output
                .1
                .iter()
                .map(|r| (r.0.key(), r.1.clone()))
                .collect(),
            client: self.properties().client.value.clone(),
        })
    }
}

impl WithRepr<Function> for FunctionWalker<'_> {
    fn repr(&self, db: &ParserDatabase) -> Result<Function> {
        Ok(Function {
            name: self.name().to_string(),
            inputs: match self.ast_function().input() {
                ast::FunctionArgs::Named(arg_list) => FunctionArgs::NamedArgList(
                    arg_list
                        .args
                        .iter()
                        .map(|(id, arg)| Ok((id.name().to_string(), arg.field_type.repr(db)?)))
                        .collect::<Result<Vec<_>>>()?,
                ),
                ast::FunctionArgs::Unnamed(arg) => {
                    FunctionArgs::UnnamedArg(arg.field_type.node(db)?.elem)
                }
            },
            output: match self.ast_function().output() {
                ast::FunctionArgs::Named(_) => bail!("Functions may not return named args"),
                ast::FunctionArgs::Unnamed(arg) => arg.field_type.node(db),
            }?,
            default_impl: self.metadata().default_impl.as_ref().map(|f| f.0.clone()),
            impls: self
                .walk_variants()
                .map(|e| e.node(db))
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

type ClientId = String;

#[derive(serde::Serialize)]
pub struct Client {
    pub name: ClientId,
    pub provider: String,
    pub options: Vec<(String, Expression)>,
}

impl WithRepr<Client> for ClientWalker<'_> {
    fn attributes(&self, _: &ParserDatabase) -> NodeAttributes {
        NodeAttributes::default()
    }

    fn repr(&self, db: &ParserDatabase) -> Result<Client> {
        Ok(Client {
            name: self.name().to_string(),
            provider: self.properties().provider.0.clone(),
            options: self
                .properties()
                .options
                .iter()
                .map(|(k, v)| Ok((k.clone(), v.repr(db)?)))
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

#[derive(serde::Serialize)]
pub struct RetryPolicyId(String);

#[derive(serde::Serialize)]
pub struct RetryPolicy {
    name: RetryPolicyId,
    max_retries: u32,
    strategy: RetryPolicyStrategy,
    // NB: the parser DB has a notion of "empty options" vs "no options"; we collapse
    // those here into an empty vec
    options: Vec<(String, Expression)>,
}

impl WithRepr<RetryPolicy> for ConfigurationWalker<'_> {
    fn attributes(&self, _db: &ParserDatabase) -> NodeAttributes {
        NodeAttributes::default()
    }

    fn repr(&self, db: &ParserDatabase) -> Result<RetryPolicy> {
        Ok(RetryPolicy {
            name: RetryPolicyId(self.name().to_string()),
            max_retries: self.retry_policy().max_retries,
            strategy: self.retry_policy().strategy,
            options: match &self.retry_policy().options {
                Some(o) => o
                    .iter()
                    .map(|((k, _), v)| Ok((k.clone(), v.repr(db)?)))
                    .collect::<Result<Vec<_>>>()?,
                None => vec![],
            },
        })
    }
}
