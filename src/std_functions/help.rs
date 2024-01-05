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
