use crate::{
    get_argument, required_argument, static_function, std_functions::Function, Error, State,
};
use polyvalue::{
    operations::IndexingOperationExt,
    types::{Array, Int, Object},
    Value, ValueTrait, ValueType,
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
            let input = get_argument!("input", arguments).as_a::<Array>()?;
            let len = Int::from(input.inner().len());
            Ok(Value::Int(len))
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
            let input = get_argument!("input", arguments).as_a::<Array>()?;
            let is_empty = input.inner().is_empty();
            Ok(Value::from(is_empty))
        }
    );

    static_function!(
        registry = map,
        name = "first",
        description = "Returns the first element of the given array",
        category = "arrays",
        arguments = [required_argument!("input", ValueType::Array)],
        returns = ValueType::Any,
        handler = |_state: &mut State, arguments, _token, _| {
            let input = get_argument!("input", arguments).as_a::<Array>()?;
            let first = input.inner().first().cloned().ok_or(Error::ArrayEmpty)?;
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
        handler = |_state: &mut State, arguments, _token, _| {
            let input = get_argument!("input", arguments).as_a::<Array>()?;
            let last = input.inner().last().cloned().ok_or(Error::ArrayEmpty)?;
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
                .as_a::<Array>()?
                .inner()
                .clone();
            let last = input.pop().ok_or(Error::ArrayEmpty)?;

            // Update the array if it references a variable
            let value = Value::Array(input.into());
            if let Some(reference) = &token.references {
                state.set_variable(reference, value)?;
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
                .as_a::<Array>()?
                .inner()
                .clone();
            let element = get_argument!("element", arguments);
            input.push(element);

            // Update the array if it references a variable
            let value = Value::Array(input.into());
            if let Some(reference) = &token.references {
                state.set_variable(reference, value.clone())?;
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
                .as_a::<Array>()?
                .inner()
                .clone();
            if input.is_empty() {
                return Err(Error::ArrayEmpty);
            }
            let first = input.remove(0);

            // Update the array if it references a variable
            let value = Value::Array(input.into());
            if let Some(reference) = &token.references {
                state.set_variable(reference, value)?;
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
                .as_a::<Array>()?
                .inner()
                .clone();
            let element = get_argument!("element", arguments);
            input.insert(0, element);

            // Update the array if it references a variable
            let value = Value::Array(input.into());
            if let Some(reference) = &token.references {
                state.set_variable(reference, value.clone())?;
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
            required_argument!("input", ValueType::Compound),
            required_argument!("index", ValueType::Int)
        ],
        returns = ValueType::Any,
        handler = |state: &mut State, arguments, token, _| {
            let mut input = get_argument!("input", arguments);
            let index = get_argument!("index", arguments);

            let value = input.delete_index(&index)?;

            // Update the array if it references a variable
            if let Some(reference) = &token.references {
                state.set_variable(reference, input)?;
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
            required_argument!("input", ValueType::Compound),
            required_argument!("index", ValueType::Int),
            required_argument!("element", ValueType::Any)
        ],
        returns = ValueType::Any,
        handler = |state: &mut State, arguments, token, _| {
            let mut input = get_argument!("input", arguments);
            let index = get_argument!("index", arguments);
            let element = get_argument!("element", arguments);

            input.set_index(&index, element)?;

            // Update the array if it references a variable
            if let Some(reference) = &token.references {
                state.set_variable(reference, input.clone())?;
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
            required_argument!("input1", ValueType::Compound),
            required_argument!("input2", ValueType::Compound)
        ],
        returns = ValueType::Compound,
        handler = |_state: &mut State, arguments, _token, _| {
            let input1 = get_argument!("input1", arguments);
            let input2 = get_argument!("input2", arguments);

            match input1.resolve(&input2)? {
                (Value::Array(a), Value::Array(b)) => {
                    let mut merged = a.inner().clone();
                    merged.extend(b.inner().clone());
                    Ok(Value::Array(merged.into()))
                }

                (Value::Object(a), Value::Object(b)) => {
                    let mut merged = a.inner().clone();
                    merged.extend(b.inner().clone());
                    Ok(Value::Object(merged.into()))
                }

                _ => Err(Error::Internal("Type mismatch in merge".to_string())),
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
        handler = |_state: &mut State, arguments, _token, _| {
            let input = get_argument!("input", arguments).as_a::<Array>()?;
            let index = *get_argument!("index", arguments).as_a::<Int>()?.inner();

            let mut split = input.inner().clone();
            let index = index as usize;
            let right = split.split_off(index);
            let left = split;

            Ok(Value::Array(
                vec![Value::Array(left.into()), Value::Array(right.into())].into(),
            ))
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
        handler = |_state: &mut State, arguments, _token, _| {
            let input = get_argument!("input", arguments).as_a::<Array>()?;
            let size = *get_argument!("size", arguments).as_a::<Int>()?.inner();

            let size = size as usize;
            let chunks = input.inner().chunks(size).map(|c| Value::from(c.to_vec()));
            Ok(Value::Array(chunks.collect::<Vec<_>>().into()))
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
        handler = |_state: &mut State, arguments, _token, _| {
            let input = get_argument!("input", arguments).as_a::<Object>()?;
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
        handler = |_state: &mut State, arguments, _token, _| {
            let input = get_argument!("input", arguments).as_a::<Object>()?;
            let values = input.values().iter().cloned().cloned().collect::<Vec<_>>();
            Ok(Value::from(values))
        }
    );
}
