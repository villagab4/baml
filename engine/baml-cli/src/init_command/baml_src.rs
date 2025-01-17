/// File to convert types to baml code.
use std::path::PathBuf;

use crate::errors::CliError;

use super::{
    interact::get_multi_selection_or_default,
    py::PythonConfig,
    traits::{ToBamlSrc, WithLanguage, WithLoader, Writer},
    ts::TypeScriptConfig,
};

pub(super) enum LanguageConfig {
    Python(PythonConfig),
    TypeScript(TypeScriptConfig),
}

impl LanguageConfig {
    pub fn project_root(&self) -> &PathBuf {
        match self {
            LanguageConfig::Python(py) => py.project_root(),
            LanguageConfig::TypeScript(ts) => ts.project_root(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            LanguageConfig::Python(_) => "python".into(),
            LanguageConfig::TypeScript(_) => "typeScript".into(),
        }
    }
}

impl WithLanguage for LanguageConfig {
    fn install_command(&self) -> String {
        match self {
            LanguageConfig::Python(py) => py.install_command(),
            LanguageConfig::TypeScript(ts) => ts.install_command(),
        }
    }

    fn test_command<T: AsRef<str>>(&self, prefix: Option<T>) -> String {
        match self {
            LanguageConfig::Python(py) => py.test_command(prefix),
            LanguageConfig::TypeScript(ts) => ts.test_command(prefix),
        }
    }

    fn package_version_command(&self) -> String {
        match self {
            LanguageConfig::Python(py) => py.package_version_command(),
            LanguageConfig::TypeScript(ts) => ts.package_version_command(),
        }
    }
}

impl WithLoader<Vec<LanguageConfig>> for LanguageConfig {
    fn from_dialoguer(
        no_prompt: bool,
        project_root: &PathBuf,
        writer: &mut Writer,
    ) -> Result<Vec<LanguageConfig>, CliError> {
        let languages = get_multi_selection_or_default(
            "What language do you want to use with BAML?",
            &["Python", "TypeScript"],
            &[true, false],
            no_prompt,
        )?;

        languages
            .iter()
            .map(|&lang| match lang {
                0 => PythonConfig::from_dialoguer(no_prompt, project_root, writer)
                    .map(LanguageConfig::Python),
                1 => TypeScriptConfig::from_dialoguer(no_prompt, project_root, writer)
                    .map(LanguageConfig::TypeScript),
                _ => unreachable!(),
            })
            .collect()
    }
}

pub(super) struct Generator {
    language: String,
    project_root: PathBuf,
    test_command: String,
    install_command: String,
    package_version_command: String,
}

impl Generator {
    pub fn new(
        language: String,
        project_root: PathBuf,
        test_command: String,
        install_command: String,
        package_version_command: String,
    ) -> Self {
        Self {
            language,
            project_root,
            test_command,
            install_command,
            package_version_command,
        }
    }
}

impl ToBamlSrc for Generator {
    fn to_baml(&self) -> String {
        format!(
            r#"
generator lang_{} {{
  language {}
  // This is where your non-baml source code located
  // (relative directory where pyproject.toml, package.json, etc. lives)
  project_root "{}"
  // This command is used by "baml test" to run tests
  // defined in the playground
  test_command "{}"
  // This command is used by "baml update-client" to install
  // dependencies to your language environment
  install_command "{}"
  package_version_command "{}"
}}
        "#,
            self.language,
            self.language,
            self.project_root.to_string_lossy(),
            self.test_command,
            self.install_command,
            self.package_version_command
        )
        .trim()
        .into()
    }
}
