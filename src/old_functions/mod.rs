mod help;
mod macros;

mod decorator_function;
mod std_function;
mod stdlib {
    use super::Function;
    use std::collections::HashMap;

    mod array;
    mod bitwise;
    mod dev;
    mod math;
    mod network;
    mod string;
    mod system;
    mod trig;

    mod decorators;

    pub fn register_all(map: &mut HashMap<String, Function>) {
        array::register_all(map);
        bitwise::register_all(map);
        dev::register_all(map);
        math::register_all(map);
        network::register_all(map);
        string::register_all(map);
        system::register_all(map);
        trig::register_all(map);

        decorators::register_all(map);
    }
}
pub use stdlib::register_all;

pub use help::{collect_help, help_to_string};
pub use std_function::*;

pub mod functions;
