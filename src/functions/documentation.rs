/// Documentation for a function
pub trait FunctionDocumentation: core::fmt::Debug {
    /// Clone the documentation
    fn clone_self(&self) -> Box<dyn FunctionDocumentation>;

    /// The category of the function
    fn category(&self) -> &str;

    /// Set the category of the function
    fn set_category(&mut self, category: &str);

    /// The description of the function
    fn description(&self) -> Option<&str>;

    /// Set the description of the function
    fn set_description(&mut self, description: Option<&str>);

    /// The extended description of the function
    fn ext_description(&self) -> Option<&str>;

    /// Set the extended description of the function
    fn set_ext_description(&mut self, ext_description: Option<&str>);

    /// The examples of the function
    fn examples(&self) -> Option<&str>;

    /// Set the examples of the function
    fn set_examples(&mut self, examples: Option<&str>);
}

/// Documentation for a function
#[derive(Debug, Clone, Copy)]
pub struct StaticFunctionDocumentation {
    /// The category of the function
    pub category: &'static str,

    /// The description of the function
    pub description: Option<&'static str>,

    /// The extended description of the function
    pub ext_description: Option<&'static str>,

    /// The examples for the function
    pub examples: Option<&'static str>,
}
impl FunctionDocumentation for StaticFunctionDocumentation {
    fn clone_self(&self) -> Box<dyn FunctionDocumentation> {
        Box::new(*self)
    }

    fn category(&self) -> &str {
        self.category
    }
    fn description(&self) -> Option<&str> {
        self.description
    }
    fn ext_description(&self) -> Option<&str> {
        self.ext_description
    }
    fn examples(&self) -> Option<&str> {
        self.examples
    }

    fn set_category(&mut self, _: &str) {}
    fn set_description(&mut self, _: Option<&str>) {}
    fn set_ext_description(&mut self, _: Option<&str>) {}
    fn set_examples(&mut self, _: Option<&str>) {}
}

/// Documentation for a function
#[derive(Debug, Clone)]
pub struct UserFunctionDocumentation {
    /// The category of the function
    pub category: String,

    /// The description of the function
    pub description: Option<String>,

    /// The extended description of the function
    pub ext_description: Option<String>,

    /// The examples for the function
    pub examples: Option<String>,
}
impl FunctionDocumentation for UserFunctionDocumentation {
    fn clone_self(&self) -> Box<dyn FunctionDocumentation> {
        Box::new(self.clone())
    }

    fn category(&self) -> &str {
        &self.category
    }
    fn set_category(&mut self, category: &str) {
        self.category = category.to_string();
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    fn set_description(&mut self, description: Option<&str>) {
        self.description = description.map(|s| s.to_string());
    }

    fn ext_description(&self) -> Option<&str> {
        self.ext_description.as_deref()
    }
    fn set_ext_description(&mut self, ext_description: Option<&str>) {
        self.ext_description = ext_description.map(|s| s.to_string());
    }

    fn examples(&self) -> Option<&str> {
        self.examples.as_deref()
    }
    fn set_examples(&mut self, examples: Option<&str>) {
        self.examples = examples.map(|s| s.to_string());
    }
}
