use super::{DocumentationFormatter, FunctionsByCategory};
use crate::State;

pub struct PlaintextFormatter;
impl PlaintextFormatter {
    fn draw_cool_box(title: &str, lines: &Vec<String>) -> String {
        let mut max_length = 0;
        for line in lines.iter() {
            if line.len() > max_length {
                max_length = line.len();
            }
        }
        if title.len() > max_length {
            max_length = title.len();
        }

        // pad to max_length+1
        let title = format!("{: <width$}", title, width = max_length + 1);

        let mut output = format!("╔{}╗\n", "═".repeat(max_length + 2));
        output += &format!("║ {}║\n", title);
        output += &format!("╟{}╢\n", "─".repeat(max_length + 2));
        for line in lines {
            output += &format!("║ {: <width$}║\n", line, width = max_length + 1);
        }
        output += &format!("╚{}╝\n", "═".repeat(max_length + 2));

        output
    }
}
impl DocumentationFormatter for PlaintextFormatter {
    //
    // Functions
    //

    fn format_function(&self, state: &State, name: &str) -> Option<String> {
        let function = state.get_function(name)?;
        let mut lines = Vec::new();
        if let Some(desc) = function.documentation().description {
            lines.push(desc.to_string());
        }
        if let Some(ext_desc) = function.documentation().ext_description {
            for line in ext_desc.split("\n") {
                lines.push(line.to_string());
            }
        }
        if let Some(examples) = function.documentation().examples {
            let examples = examples.trim_start_matches("#skip").trim();
            if !examples.is_empty() {
                lines.push("Examples:".to_string());
                for line in examples.split("\n") {
                    lines.push(format!("  {}", line));
                }
            }
        }

        Some(Self::draw_cool_box(&function.signature(), &lines))
    }

    fn format_function_category(&self, state: &State, category: &str) -> Option<String> {
        let functions = state.functions_by_category();
        let key = functions
            .keys()
            .find(|k| k.to_lowercase() == category.to_lowercase())?;
        let functions = functions.get(key)?;

        let mut output = Vec::new();

        for f in functions {
            let mut lines = Vec::new();
            if let Some(desc) = f.documentation().description {
                lines.push(desc.to_string());
            }
            if let Some(ext_desc) = f.documentation().ext_description {
                for line in ext_desc.split("\n") {
                    lines.push(line.to_string());
                }
            }

            if lines.is_empty() {
                output.push(f.signature());
            } else {
                output.push(Self::draw_cool_box(&f.signature(), &lines));
            }
        }

        Some(output.join("\n"))
    }

    fn format_function_list(&self, state: &State) -> String {
        let categories = state.functions_by_category();
        let mut output = Vec::new();

        let mut sorted_categories: Vec<_> = categories.keys().collect();
        sorted_categories.sort();

        for category in sorted_categories {
            let functions = categories.get(category).unwrap();
            let lines = functions
                .iter()
                .map(|f| match f.documentation().description {
                    Some(desc) => format!("{} : {}", f.signature(), desc),
                    None => f.signature(),
                })
                .collect();
            output.push(Self::draw_cool_box(category, &lines));
        }

        output.join("\n")
    }

    //
    // Section Loaders
    //

    fn format_operators(&self) -> String {
        todo!()
    }

    fn format_title(&self, title: &str) -> String {
        format!("{}\n{}", title, "=".repeat(title.len()))
    }

    fn format_subtitle(&self, title: &str) -> String {
        format!("{}\n{}", title, "-".repeat(title.len()))
    }

    fn format_subsubtitle(&self, title: &str) -> String {
        format!("{}\n\n", title)
    }

    fn format_text(&self, text: &str) -> String {
        text.to_string()
    }
}
