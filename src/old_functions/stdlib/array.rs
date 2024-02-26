use crate::{
    error::{ErrorDetails, WrapExternalError},
    get_argument, oops, required_argument, static_function,
    std_functions::Function,
    State,
};
use polyvalue::{
    operations::IndexingMutationExt,
    types::{Array, Object, I64},
    InnerValue, Value, ValueTrait, ValueType,
};
use std::collections::HashMap;

pub fn register_all(map: &mut HashMap<String, Function>) {
    static_function!(
        registry = map,
        name = "len",
        description = "Returns the length of the given array or object",
        category = "arrays",
        arguments = [required_argument!("input", ValueType::Any)],
        returns = ValueType::Int,
        handler = |_state: &mut State, arguments, _token, _| {
            let len = get_argument!("input", arguments).len() as i64;
            Ok(Value::i64(len))
        }
    );

    static_function!(
        registry = map,
        name = "is_empty",
        description = "Returns true if the given array or object is empty",
        category = "arrays",
        arguments = [required_argument!("input", ValueType::Any)],
        returns = ValueType::Bool,
        handler = |_state: &mut State, arguments, _token, _| {
            let len = get_argument!("input", arguments).len() as i64;
            Ok(Value::from(len == 0))
        }
    );

    static_function!(
        registry = map,
        name = "first",
        description = "Returns the first element of the given array",
        category = "arrays",
        arguments = [required_argument!("input", ValueType::Array)],
        returns = ValueType::Any,
        handler = |_state: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Array>()
                .with_context(token)?;
            let first = input
                .inner()
                .first()
                .cloned()
                .ok_or(ErrorDetails::ArrayEmpty)
                .with_context(token)?;
            Ok(first)
        }
    );

    static_function!(
        registry = map,
        name = "last",
        description = "Returns the last element of the given array",
        category = "arrays",
        arguments = [required_argument!("input", ValueType::Array)],
        returns = ValueType::Any,
        handler = |_state: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Array>()
                .with_context(token)?;
            let last = input
                .inner()
                .last()
                .cloned()
                .ok_or(ErrorDetails::ArrayEmpty)
                .with_context(token)?;
            Ok(last)
        }
    );

    static_function!(
        registry = map,
        name = "pop",
        description = "Removes and returns the last element of the given array",
        category = "arrays",
        arguments = [required_argument!("input", ValueType::Array)],
        returns = ValueType::Any,
        handler = |state: &mut State, arguments, token, _| {
            let mut input = get_argument!("input", arguments)
                .as_a::<Array>()
                .with_context(token)?
                .inner()
                .clone();
            let last = input
                .pop()
                .ok_or(ErrorDetails::ArrayEmpty)
                .with_context(token)?;

            // Update the array if it references a variable
            let value = Value::array(input);
            if let Some(reference) = &token.references {
                state.set_variable(reference, value);
            }

            Ok(last)
        }
    );

    static_function!(
        registry = map,
        name = "push",
        description =
            "Adds the given element to the end of the given array, then returns the array",
        category = "arrays",
        arguments = [
            required_argument!("input", ValueType::Array),
            required_argument!("element", ValueType::Any)
        ],
        returns = ValueType::Array,
        handler = |state: &mut State, arguments, token, _| {
            let mut input = get_argument!("input", arguments)
                .as_a::<Array>()
                .with_context(token)?
                .inner()
                .clone();
            let element = get_argument!("element", arguments);
            input.push(element);

            // Update the array if it references a variable
            let value = Value::array(input);
            if let Some(reference) = &token.references {
                state.set_variable(reference, value.clone());
            }

            Ok(value)
        }
    );

    static_function!(
        registry = map,
        name = "deque",
        description = "Removes and returns the first element of the given array",
        category = "arrays",
        arguments = [required_argument!("input", ValueType::Array)],
        returns = ValueType::Any,
        handler = |state: &mut State, arguments, token, _| {
            let mut input = get_argument!("input", arguments)
                .as_a::<Array>()
                .with_context(token)?
                .inner()
                .clone();
            if input.is_empty() {
                return oops!(ArrayEmpty, token.clone());
            }
            let first = input.remove(0);

            // Update the array if it references a variable
            let value = Value::array(input);
            if let Some(reference) = &token.references {
                state.set_variable(reference, value);
            }

            Ok(first)
        }
    );

    static_function!(
        registry = map,
        name = "enque",
        description =
            "Adds the given element to the beginning of the given array, then returns the array",
        category = "arrays",
        arguments = [
            required_argument!("input", ValueType::Array),
            required_argument!("element", ValueType::Any)
        ],
        returns = ValueType::Array,
        handler = |state: &mut State, arguments, token, _| {
            let mut input = get_argument!("input", arguments)
                .as_a::<Array>()
                .with_context(token)?
                .inner()
                .clone();
            let element = get_argument!("element", arguments);
            input.insert(0, element);

            // Update the array if it references a variable
            let value = Value::array(input);
            if let Some(reference) = &token.references {
                state.set_variable(reference, value.clone());
            }

            Ok(value)
        }
    );

    static_function!(
        registry = map,
        name = "remove",
        description =
            "Removes the element at the given index from the given array or object and returns it",
        category = "arrays",
        arguments = [
            required_argument!("input", ValueType::Collection),
            required_argument!("index", ValueType::Int)
        ],
        returns = ValueType::Any,
        handler = |state: &mut State, arguments, token, _| {
            let mut input = get_argument!("input", arguments);
            let index = get_argument!("index", arguments);

            let value = input.delete_index(&index).with_context(token)?;

            // Update the array if it references a variable
            if let Some(reference) = &token.references {
                state.set_variable(reference, input);
            }

            Ok(value)
        }
    );

    // insert - insert element at index
    static_function!(
        registry = map,
        name = "insert",
        description = "Inserts the given element at the given index in the given array or object",
        category = "arrays",
        arguments = [
            required_argument!("input", ValueType::Collection),
            required_argument!("index", ValueType::Int),
            required_argument!("element", ValueType::Any)
        ],
        returns = ValueType::Any,
        handler = |state: &mut State, arguments, token, _| {
            let mut input = get_argument!("input", arguments);
            let index = get_argument!("index", arguments);
            let element = get_argument!("element", arguments);

            input.set_index(&index, element).with_context(token)?;

            // Update the array if it references a variable
            if let Some(reference) = &token.references {
                state.set_variable(reference, input.clone());
            }

            Ok(input)
        }
    );

    // merge - merge two arrays or objects
    static_function!(
        registry = map,
        name = "merge",
        description = "Merges the given arrays or objects",
        category = "arrays",
        arguments = [
            required_argument!("input1", ValueType::Collection),
            required_argument!("input2", ValueType::Collection)
        ],
        returns = ValueType::Collection,
        handler = |_state: &mut State, arguments, token, _| {
            let input1 = get_argument!("input1", arguments);
            let input2 = get_argument!("input2", arguments);

            let (left, right) = input1.resolve(&input2).with_context(token)?;
            match (left.inner(), right.inner()) {
                (InnerValue::Array(a), InnerValue::Array(b)) => {
                    let mut merged = a.inner().clone();
                    merged.extend(b.inner().clone());
                    Ok(Value::array(merged))
                }

                (InnerValue::Object(a), InnerValue::Object(b)) => {
                    let mut merged = a.inner().clone();
                    merged.extend(b.inner().clone());
                    Ok(Value::object(merged))
                }

                _ => oops!(
                    Internal {
                        msg: "Type mismatch in merge".to_string()
                    },
                    token.clone()
                ),
            }
        }
    );

    // split - split an array at index
    static_function!(
        registry = map,
        name = "split",
        description = "Splits the given array at the given index",
        category = "arrays",
        arguments = [
            required_argument!("input", ValueType::Array),
            required_argument!("index", ValueType::Int)
        ],
        returns = ValueType::Array,
        handler = |_state: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Array>()
                .with_context(token)?;
            let index = *get_argument!("index", arguments)
                .as_a::<I64>()
                .with_context(token)?
                .inner();

            let mut split = input.inner().clone();
            let index = index as usize;
            let right = split.split_off(index);
            let left = split;

            Ok(Value::array(vec![Value::array(left), Value::array(right)]))
        }
    );

    // chunks - split an array into chunks of a given size
    static_function!(
        registry = map,
        name = "chunks",
        description = "Splits the given array into chunks of the given size",
        category = "arrays",
        arguments = [
            required_argument!("input", ValueType::Array),
            required_argument!("size", ValueType::Int)
        ],
        returns = ValueType::Array,
        handler = |_state: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Array>()
                .with_context(token)?;
            let size = *get_argument!("size", arguments)
                .as_a::<I64>()
                .with_context(token)?
                .inner();

            let size = size as usize;
            let chunks = input.inner().chunks(size).map(|c| Value::from(c.to_vec()));
            Ok(Value::array(chunks.collect::<Vec<_>>()))
        }
    );

    // keys - return the keys of an object
    static_function!(
        registry = map,
        name = "keys",
        description = "Returns the keys of the given object",
        category = "arrays",
        arguments = [required_argument!("input", ValueType::Object)],
        returns = ValueType::Array,
        handler = |_state: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Object>()
                .with_context(token)?;
            let keys = input.keys().iter().cloned().cloned().collect::<Vec<_>>();
            Ok(Value::from(keys))
        }
    );

    // values - return the values of an object
    static_function!(
        registry = map,
        name = "values",
        description = "Returns the values of the given object",
        category = "arrays",
        arguments = [required_argument!("input", ValueType::Object)],
        returns = ValueType::Array,
        handler = |_state: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Object>()
                .with_context(token)?;
            let values = input.values().iter().cloned().cloned().collect::<Vec<_>>();
            Ok(Value::from(values))
        }
    );

    // all - return true if all elements in the array are true
    static_function!(
        registry = map,
        name = "all",
        description = "Returns true if all elements in the given array are true",
        category = "arrays",
        arguments = [required_argument!("input", ValueType::Array)],
        returns = ValueType::Bool,
        handler = |_state: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Array>()
                .with_context(token)?;
            let all = input.inner().iter().all(|v| v.is_truthy());
            Ok(Value::from(all))
        }
    );

    // any - return true if any elements in the array are true
    static_function!(
        registry = map,
        name = "any",
        description = "Returns true if any elements in the given array are true",
        category = "arrays",
        arguments = [required_argument!("input", ValueType::Array)],
        returns = ValueType::Bool,
        handler = |_state: &mut State, arguments, token, _| {
            let input = get_argument!("input", arguments)
                .as_a::<Array>()
                .with_context(token)?;
            let any = input.inner().iter().any(|v| v.is_truthy());
            Ok(Value::from(any))
        }
    );
}
