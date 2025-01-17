use internal_baml_parser_database::walkers::{ArgWalker, EnumWalker, Walker};
use internal_baml_schema_ast::ast::{
    EnumValue, FieldType, FunctionArg, FunctionId, Identifier, WithDocumentation, WithIdentifier,
    WithName,
};

use serde_json::json;

use crate::generate::generate_python_client::file::clean_file_name;

use super::{
    file::File,
    template::render_template,
    traits::{JsonHelper, WithPartial, WithToCode, WithWritePythonString},
    FileCollector,
};

impl JsonHelper for ArgWalker<'_> {
    fn json(&self, f: &mut File) -> serde_json::Value {
        let _ = self
            .required_classes()
            .map(|cls| {
                f.add_import(&format!("..types.classes.{}", cls.file_name()), cls.name());
                f.add_import(
                    &format!("..types.partial.classes.{}", cls.file_name()),
                    &format!("Partial{}", cls.name()),
                );
            })
            .collect::<Vec<_>>();
        let _ = self
            .required_enums()
            .map(|enm| f.add_import(&format!("..types.enums.{}", enm.file_name()), enm.name()))
            .collect::<Vec<_>>();

        match self.ast_arg() {
            (Some(idn), arg) => json!({
                "name": idn.to_py_string(f),
                "type": arg.to_py_string(f),
                // Hack, because there is no easy way to check for enums in types_partial.rs and we can't reuse that. Also the logic in that file only works for class fields.
                // It is currently hacked together to just make class fields optional for now to support streaming. It doesn't actually do anything recursive.
                // Here we want the function return type for primitive to say "str" and not "PartialStr"
                // To fix, move to the IR.
                "type_partial": match arg.clone().field_type {
                    // if the names match any of the required_enums, just return the same name, otherwise return Partial{name} unless it's a primitive
                    _ if self.required_enums().any(|enm| enm.name() == idn.name()) => idn.to_py_string(f),
                    FieldType::Identifier(arity, idn) => {
                        // check the identifier
                        match idn {

                            Identifier::Ref(idn, _) => {
                                // f.add_import(&idn.path.join("."), &format!("Partial{}", &idn.name));
                                format!("Partial{}", idn.name.clone())
                            }
                            Identifier::Local(idn, _) => format!("Partial{}", &idn.to_string()),
                            Identifier::String(str, _) => format!("Partial{}", str.to_string()),
                            Identifier::Primitive(p, _) => p.to_py_string(f),
                            _ => arg.to_py_string(f),
                        }
                    },
                    _ => arg.to_py_string(f),
                },
                "default": match arg.field_type.is_nullable() {
                    true => Some("None"),
                    false => None
                }
            }),
            (None, arg) => json!({
                "type": arg.to_py_string(f),
                "type_partial": match arg.clone().field_type {
                    FieldType::Identifier(arity, idn) => {
                        if is_enum(&idn, &arg, f, self.required_enums()) {
                            idn.to_py_string(f)
                        } else {
                            match idn {
                                Identifier::Ref(idn, _) => {
                                    format!("Partial{}", idn.name.clone())
                                }
                                Identifier::Local(idn, _) => format!("Partial{}", &idn.to_string()),
                                Identifier::String(str, _) => format!("Partial{}", str.to_string()),
                                Identifier::Primitive(p, _) => p.to_py_string(f),
                                _ => arg.to_py_string(f),
                            }
                        }
                    },
                    _ => arg.to_py_string(f),
                },
                "default": match arg.field_type.is_nullable() {
                    true => Some("None"),
                    false => None
                }
            }),
        }
    }
}

impl JsonHelper for Walker<'_, FunctionId> {
    fn json(&self, f: &mut File) -> serde_json::Value {
        let impls = self
            .walk_variants()
            .map(|v| v.name().to_string())
            .collect::<Vec<_>>();
        let mut inputs = self.walk_input_args().collect::<Vec<_>>();
        inputs.sort_by(|a, b| a.is_optional().cmp(&b.is_optional()));

        json!({
            "name": self.ast_function().name(),
            "unnamed_args": self.is_positional_args(),
            "args": inputs.iter().map(|a| a.json(f)).collect::<Vec<_>>(),
            "return": self.walk_output_args().map(|a| a.json(f)).collect::<Vec<_>>(),
            "doc_string": self.ast_function().documentation(),
            "impls": impls,
            "has_impls": impls.len() > 0,
            "default_impl": self.metadata().default_impl.as_ref().map(|f| f.0.clone())
        })
    }
}
fn is_enum<'a>(
    idn: &Identifier,
    arg: &FunctionArg,
    f: &mut File,
    required_enums: impl Iterator<Item = EnumWalker<'a>>,
) -> bool {
    let required_enums_vec: Vec<_> = required_enums.collect();
    if required_enums_vec
        .iter()
        .any(|enm| enm.name() == idn.name())
    {
        return true;
    }
    return false;
}

impl WithWritePythonString for Walker<'_, FunctionId> {
    fn file_name(&self) -> String {
        format!("fx_{}", clean_file_name(self.name()))
    }

    fn write_py_file(&self, fc: &mut FileCollector) {
        fc.start_py_file("functions", "__init__");
        fc.complete_file();

        fc.start_py_file("functions", self.file_name());
        let json = self.json(fc.last_file());
        render_template(super::template::HSTemplate::Function, fc.last_file(), json);
        fc.complete_file();

        fc.start_py_file("functions", format!("{}.pyi", self.file_name()));
        let json = self.json(fc.last_file());
        render_template(
            super::template::HSTemplate::FunctionPYI,
            fc.last_file(),
            json,
        );
        fc.complete_file();
    }
}
