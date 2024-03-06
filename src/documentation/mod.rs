use crate::{functions::ParserFunction, State};
use std::collections::HashMap;

mod plain;
pub use plain::PlaintextFormatter;

mod markdown;
pub use markdown::MarkdownFormatter;

#[macro_use]
mod operator_documentation;
pub use operator_documentation::OperatorDocumentation;

mod static_docs;
pub use static_docs::DocumentationTemplate;

pub trait FunctionsByCategory {
    fn functions_by_category(&self) -> HashMap<String, Vec<&dyn ParserFunction>>;
}

impl FunctionsByCategory for State {
    fn functions_by_category(&self) -> HashMap<String, Vec<&dyn ParserFunction>> {
        let mut categories: HashMap<String, Vec<&dyn ParserFunction>> = HashMap::new();
        for (_, function) in self.all_functions().iter() {
            if function.name().starts_with("__") {
                // Skip hidden functions
                continue;
            }

            if !categories.contains_key(function.documentation().category()) {
                categories.insert(function.documentation().category().to_string(), Vec::new());
            }
            categories
                .get_mut(function.documentation().category())
                .unwrap()
                .push(function.as_ref());
        }

        for (_, functions) in categories.iter_mut() {
            functions.sort_by(|f1, f2| f1.name().cmp(f2.name()));
        }
        categories
    }
}

pub trait DocumentationFormatter {
    /// A single function including extended descriptions and examples
    fn format_function(&self, state: &State, name: &str) -> Option<String>;

    /// A more focussed list of functions including extended descriptions
    fn format_function_category(&self, state: &State, category: &str) -> Option<String>;

    /// A general list of function signatures and short descriptions
    fn format_function_list(&self, state: &State) -> String;

    //
    // Section Documentation
    //

    fn format_operators(&self) -> String;

    fn format_functions(&self, state: &State, search: Option<&str>) -> String {
        if let Some(search) = search {
            if let Some(s) = self.format_function(state, search) {
                s
            } else if let Some(s) = self.format_function_category(state, search) {
                s
            } else {
                format!("No function or category found for '{}'", search)
            }
        } else {
            self.format_function_list(state)
        }
    }

    //
    // Global
    //

    fn format_title(&self, title: &str) -> String;
    fn format_subtitle(&self, title: &str) -> String;
    fn format_subsubtitle(&self, title: &str) -> String;
    fn format_text(&self, text: &str) -> String;
}
