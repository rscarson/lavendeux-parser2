use crate::std_functions::Function;
use std::collections::HashMap;

pub fn collect_help(
    function_map: HashMap<String, Function>,
    filter: Option<String>,
) -> HashMap<String, Vec<String>> {
    let mut help_map: HashMap<String, Vec<String>> = HashMap::new();
    for (name, function) in function_map.iter() {
        let category = function.category().to_string();

        if let Some(filter) = &filter {
            if !category.contains(filter) && !name.contains(filter) {
                continue;
            }
        }

        if !help_map.contains_key(&category) {
            help_map.insert(category.clone(), Vec::new());
        }

        let help = help_map.get_mut(&category).unwrap();
        help.push(function_help(function));
        help.sort();
    }
    help_map
}

fn function_help(function: &Function) -> String {
    if function.description().len() > 0 {
        format!("{} : {}", function.signature(), function.description())
    } else {
        function.signature()
    }
}

pub fn help_to_string(string_map: HashMap<String, Vec<String>>) -> String {
    let mut keys = string_map.keys().cloned().collect::<Vec<String>>();
    keys.sort();
    keys.iter()
        .map(|category| {
            draw_cool_box(category, string_map.get(category).unwrap())
            /*format!(
                "## {}\n\n{}\n",
                category,
                string_map.get(category).unwrap().join("\n")
            )*/
        })
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn draw_cool_box(title: &str, lines: &Vec<String>) -> String {
    let mut max_length = 0;
    for line in lines.iter() {
        if line.len() > max_length {
            max_length = line.len();
        }
    }
    if title.len() > max_length {
        max_length = title.len();
    }

    // capitalize title[0] and pad to max_length+1
    let title = format!("{}{}", title[0..1].to_uppercase(), &title[1..]);
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
