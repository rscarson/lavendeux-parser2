use super::{operator_documentation, DocumentationFormatter, FunctionsByCategory};

#[allow(dead_code)]
enum MarkdownSnippet {
    H1(String),
    H2(String),
    H3(String),
    H4(String),

    CodeBlock(String),
    CodeInline(String),

    Text(String),
}
impl std::fmt::Display for MarkdownSnippet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarkdownSnippet::H1(s) => write!(f, "# {}", s.trim()),
            MarkdownSnippet::H2(s) => write!(f, "## {}", s.trim()),
            MarkdownSnippet::H3(s) => write!(f, "### {}", s.trim()),
            MarkdownSnippet::H4(s) => write!(f, "#### {}", s.trim()),
            MarkdownSnippet::CodeBlock(s) => write!(f, "```lavendeux\n{}\n```", s.trim()),
            MarkdownSnippet::CodeInline(s) => write!(f, "`{}`", s.trim()),
            MarkdownSnippet::Text(s) => write!(f, "{}", s),
        }?;
        write!(f, "\n")
    }
}

pub struct MarkdownFormatter;
impl DocumentationFormatter for MarkdownFormatter {
    //
    // Functions
    //

    fn format_function(&self, state: &crate::State, name: &str) -> Option<String> {
        let function = state.get_function(name)?;
        let mut pieces = Vec::new();

        pieces.push(MarkdownSnippet::H3(function.name().to_string()));
        pieces.push(MarkdownSnippet::CodeBlock(function.signature().to_string()));

        if let Some(desc) = function.documentation().description {
            pieces.push(MarkdownSnippet::Text(desc.to_string()));
        }
        if let Some(ext_desc) = function.documentation().ext_description {
            for line in ext_desc.split("\n") {
                pieces.push(MarkdownSnippet::Text(line.to_string() + "  "));
            }
        }
        if let Some(examples) = function.documentation().examples {
            let examples = examples.trim_start_matches("#skip").trim();
            if !examples.is_empty() {
                pieces.push(MarkdownSnippet::Text(format!("**Examples:**  ",)));
                pieces.push(MarkdownSnippet::CodeBlock(examples.to_string()));
            }
        }

        Some(
            pieces
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(""),
        )
    }

    fn format_function_category(&self, state: &crate::State, category: &str) -> Option<String> {
        let functions = state.functions_by_category();
        let key = functions
            .keys()
            .find(|k| k.to_lowercase() == category.to_lowercase())?;
        let functions = functions.get(key)?;

        let mut output = Vec::new();

        for f in functions {
            output.push(self.format_function(state, f.name())?);
        }

        Some(output.join("\n------------\n"))
    }

    fn format_function_list(&self, state: &crate::State) -> String {
        let categories = state.functions_by_category();
        let mut output = vec![];

        let mut sorted_categories: Vec<_> = categories.keys().collect();
        sorted_categories.sort();

        for category in sorted_categories {
            output.push(MarkdownSnippet::H2(category.to_string() + " Functions"));
            output.push(MarkdownSnippet::Text(
                self.format_function_category(state, category)
                    .unwrap_or_default(),
            ));
        }

        output
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join("")
    }

    //
    // Section Loaders
    //

    fn format_operators(&self) -> String {
        let mut output = vec![];
        let mut operators = operator_documentation::all();
        operators.sort_by(|a, b| a.name.cmp(&b.name));

        for operator in operators {
            output.push(MarkdownSnippet::H2(operator.name.to_string()));

            let symbols = operator.symbols.join(", ");
            output.push(MarkdownSnippet::Text(format!("**[{}]**  ", symbols)));

            output.push(MarkdownSnippet::Text(operator.description.to_string()));

            output.push(MarkdownSnippet::H3("Examples".to_string()));
            output.push(MarkdownSnippet::CodeBlock(operator.examples.to_string()));
        }

        output
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join("")
    }

    fn format_title(&self, title: &str) -> String {
        MarkdownSnippet::H1(title.to_string()).to_string()
    }

    fn format_subtitle(&self, title: &str) -> String {
        MarkdownSnippet::H2(title.to_string()).to_string()
    }

    fn format_subsubtitle(&self, title: &str) -> String {
        MarkdownSnippet::H3(title.to_string()).to_string()
    }

    fn format_text(&self, text: &str) -> String {
        MarkdownSnippet::Text(text.to_string()).to_string()
    }
}
