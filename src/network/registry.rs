use super::{defaults::default_apis, ApiDefinition};
use crate::State;
use polyvalue::{types::Object, Value};
use std::collections::HashMap;

pub struct ApiRegistry(HashMap<String, ApiDefinition>);
impl ApiRegistry {
    const STORE_NAME: &'static str = "__api_definitions";

    /// Create a new instance of the registry, loading the APIs
    /// from the state object
    pub fn new(state: &State) -> Self {
        let mut inst = Self(HashMap::new());
        inst.load(state);
        inst
    }

    /// Populate the state with the default APIs
    pub fn populate_defaults(state: &mut State) {
        Self(default_apis()).save(state);
    }

    /// Get the raw value of the registry from the state object
    pub fn raw(state: &State) -> Value {
        state
            .global_get_variable(Self::STORE_NAME)
            .cloned()
            .unwrap_or(Object::default().into())
    }

    /// Load the APIs from the state object
    fn load(&mut self, state: &State) {
        self.0.clear();
        let state = Self::raw(state).as_a::<Object>().unwrap_or_default();
        for (k, v) in state.iter() {
            if let Ok(api) = ApiDefinition::try_from(v.clone()) {
                self.0.insert(k.to_string(), api);
            }
        }
    }

    /// Save the APIs to the state object
    fn save(&self, state: &mut State) {
        let obj = self
            .0
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect::<Vec<(_, _)>>();
        state.global_assign_variable(Self::STORE_NAME, Value::try_from(obj).unwrap());
    }

    /// Add a new API to the registry
    pub fn add(&mut self, state: &mut State, name: &str, api: ApiDefinition) {
        self.0.insert(name.to_string(), api);
        self.save(state);
    }

    /// Remove an API from the registry
    pub fn remove(&mut self, state: &mut State, name: &str) {
        self.0.remove(name);
        self.save(state);
    }

    /// Get an API from the registry
    pub fn get(&self, name: &str) -> Option<&ApiDefinition> {
        self.0.get(name)
    }

    /// Get all APIs from the registry
    pub fn all(&self) -> &HashMap<String, ApiDefinition> {
        &self.0
    }
}
