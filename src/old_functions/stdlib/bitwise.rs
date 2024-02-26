use crate::{
    error::WrapExternalError, get_argument, required_argument, static_function,
    std_functions::Function, State,
};
use polyvalue::{types::I64, ValueType};
use std::collections::HashMap;

pub fn register_all(map: &mut HashMap<String, Function>) {
    static_function!(
        registry = map,
        name = "unsized_not",
        description = "Bitwise NOT that attempts to remove the effect of the size of the integer (i.e. ~0xF0 == 0x0F)",
        category = "bitwise",
        arguments = [required_argument!("input", ValueType::Int)],
        returns = ValueType::Int,
        handler = |_: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments).as_a::<I64>().with_context(token)?;
            Ok(input.unsized_bitwise_not().into())
        }
    );
}
