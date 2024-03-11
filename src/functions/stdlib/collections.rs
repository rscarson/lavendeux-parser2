use crate::{
    define_stdfunction,
    error::{ErrorDetails, WrapExternalError},
};
use polyvalue::{
    operations::{IndexingMutationExt, IndexingOperationExt},
    types::{Array, Object},
    Value, ValueType,
};

/**********************************************
 *
 * Array Metadata Functions
 *
 *********************************************/

define_stdfunction!(
    len { input: Standard::Any },
    returns = Int,
    docs = {
        category: "Collections",
        description: "Returns the length of the given array or object",
        ext_description: "
            For arrays and objects, this function returns the number of elements in the array or object.
            For strings, it returns the number of characters.
            For all other types it will return 1
        ",
        examples: "
            assert_eq(len('test'),       4);
            assert_eq(len([1, 2, 3]),    3);
            assert_eq(len({'a': 1, 'b': 2}), 2);
            assert_eq(len(38),           1);
        ",
    },
    handler = (state, _reference) {
        let input = required_arg!(state::input);
        Ok(Value::i64(input.len() as i64))
    },
);

define_stdfunction!(
    is_empty { input: Standard::Any },
    returns = Bool,
    docs = {
    category: "Collections",
    description: "Returns true if the given array or object is empty",
    ext_description: "
        For arrays and objects, this function returns true if the array or object has no elements.
        For strings, it returns true if the string is empty.
        For all other types it will return false
    ",
    examples: "
        assert_eq(is_empty([]),     true);
        assert_eq(is_empty({}),     true);
        assert_eq(is_empty('test'), false);
        assert_eq(is_empty(38),     false);
        ",
    },
    handler = (state, _reference) {
        let input = required_arg!(state::input);
        Ok(Value::bool(input.len() == 0))
    },
);

define_stdfunction!(
    first { input: Standard::Array },
    returns = Any,
    docs = {
        category: "Collections",
        description: "Returns the first element of the given array",
        ext_description: "
            Coerces its argument to an array and returns the first element.
            If the resulting array is empty, an error is returned.
        ",
        examples: "
            assert_eq(first([1, 2, 3]), 1);
            assert_eq(first(3),         3); // equivalent to first([3])
            
            would_err('first([])'); // Array is empty, so an error is returned
        ",
    },
    handler = (state, _reference) {
        let input = required_arg!(state::input).as_a::<Array>()?;
        input.first().cloned().ok_or(ErrorDetails::ArrayEmpty).without_context()
    },
);

define_stdfunction!(
    last { input: Standard::Array },
    returns = Any,
    docs = {
        category: "Collections",
        description: "Returns the last element of the given array",
        ext_description: "
            Coerces its argument to an array and returns the last element.
            If the resulting array is empty, an error is returned.
        ",
        examples: "
            assert_eq(last([1, 2, 3]), 3);
            assert_eq(last(3),         3); // equivalent to last([3])

            would_err('last([])'); // Array is empty, so an error is returned
        ",
    },
    handler = (state, _reference) {
        let input = required_arg!(state::input).as_a::<Array>()?;
        input.last().cloned().ok_or(ErrorDetails::ArrayEmpty).without_context()
    },
);

/**********************************************
 *
 * Array Manipulation Functions
 *
 *********************************************/

define_stdfunction!(
    pop { input: Standard::Array },
    returns = Any,
    docs = {
        category: "Collections",
        description: "Removes and returns the last element of the given array",
        ext_description: "
            Removes the last element from the given array and returns it.
            If the array is empty, an error is returned.
            If the input is a reference to an array in a variable, the variable is updated.
        ",
        examples: "
            assert_eq(pop([1, 2, 3]), 3);
            would_err('pop([]') // Array is empty, so an error is returned
            
            a = [1];
            assert_eq(pop(a), 1);
            assert_eq(a, []);
        ",
    },
    handler = (state, reference) {
        let input = required_arg!(state::input);
        let input_type = input.own_type();
        let mut input = input.as_a::<Array>()?.clone();
        let value = input.pop().ok_or(ErrorDetails::ArrayEmpty).without_context()?;

        // Update the array if it references a variable containing an array
        if let Some(reference) = reference {
            if input_type == ValueType::Array {
                reference.update_value_in_parent(state, input.clone().into())?;
            }
        };

        Ok(value)
    },
);

define_stdfunction!(
    push { input: Standard::Collection, value: Standard::Any },
    returns = Array,
    docs = {
        category: "Collections",
        description: "Appends the given value to the end of the given collection, and returns the result",
        ext_description: "
            Appends the given value to the end of the given collection.
            If the input is a reference to a collection in a variable, the variable is updated.
        ",
        examples: "
        assert_eq(push([1, 2], 3), [1, 2, 3]);
        assert_eq(push([], 3), [3]);
        
        a = [1];
        assert_eq(push(a, 2), [1, 2]);
        assert_eq(a, [1, 2]);
    ",
    },
    handler = (state, reference) {
        let input = required_arg!(state::input);
        let input_type = input.own_type();
        let value = required_arg!(state::value);

        match input_type {
            ValueType::Array => {
                let mut input = input.as_a::<Array>()?;
                input.push(value.clone());

                // Update the array if it references a variable containing an array
                if let Some(reference) = reference {
                    if let Some(target) = reference.get_target_mut_in_parent(state)? {
                        *target = input.clone().into();
                    }
                };

                Ok(input.into())
            }

            ValueType::String => {
                let mut input = input.as_a::<String>()?;
                input.push_str(&value.to_string());

                // Update the array if it references a variable containing an array
                if let Some(reference) = reference {
                    reference.update_value_in_parent(state, input.clone().into())?;
                };

                Ok(input.into())
            }

            _ => oops!(Custom {
                msg: format!("cannot push to `{input_type}`")
            })
        }
    },
);

define_stdfunction!(
    enqueue { input: Standard::Array, value: Standard::Any },
    returns = Array,
    docs = {
        category: "Collections",
        description: "Appends the given value to the start of the given array, and returns the result",
        ext_description: "
            Appends the given value to the start of the given array.
            If the input is a reference to an array in a variable, the variable is updated.
            This function is less performant than `push` for large arrays, as it requires shifting all elements by one position.
        ",
        examples: "
            assert_eq(enqueue([1, 2], 3), [3, 1, 2])
            assert_eq(enqueue([], 3), [3])
            
            a = [1]
            assert_eq(enqueue(a, 2), [2, 1])
            assert_eq(a, [2, 1])
        ",
    },
    handler = (state, reference) {
        let input = required_arg!(state::input);
        let input_type = input.own_type();
        let mut input = input.as_a::<Array>()?.clone();
        let value = required_arg!(state::value).clone();

        input.insert(0, value);

        // Update the array if it references a variable containing an array
        if let Some(reference) = reference {
            if input_type == ValueType::Array {
                reference.update_value_in_parent(state, input.clone().into())?;
            }
        };

        Ok(input.into())
    },
);

define_stdfunction!(
    dequeue { input: Standard::Array },
    returns = Array,
    docs = {
        category: "Collections",
        description: "Removes and returns the first element of the given array",
        ext_description: "
            Removes the first element from the given array and returns it.
            If the array is empty, an error is returned.
            If the input is a reference to an array in a variable, the variable is updated.
            This function is less performant than `pop` for large arrays, as it requires shifting all elements by one position.
        ",
        examples: "
            assert_eq(dequeue([1, 2, 3]), 1);
            would_err('dequeue([]') // Array is empty, so an error is returned
            
            a = [1, 2];
            assert_eq(dequeue(a), 1);
            assert_eq(a, [2]);
        ",
    },
    handler = (state, reference) {
        let input = required_arg!(state::input);
        let input_type = input.own_type();
        let mut input = input.as_a::<Array>()?.clone();
        let value = input.remove(0).ok_or(ErrorDetails::ArrayEmpty).without_context()?;

        // Update the array if it references a variable containing an array
        if let Some(reference) = reference {
            if input_type == ValueType::Array {
                reference.update_value_in_parent(state, input.clone().into())?;
            }
        };

        Ok(value)
    },
);

define_stdfunction!(
    insert {
        input: Standard::Array,
        index: Standard::Int,
        value: Standard::Any
    },
    returns = Array,
    docs = {
        category: "Collections",
        description: "Inserts the given value at the given index in the given array, and returns the result",
        ext_description: "
            Inserts the given value at the given index in the given array.
            If the input is a reference to an array in a variable, the variable is updated.
            If the index is out of bounds, an error is returned.
        ",
        examples: "
            assert_eq(insert([1, 2, 3], 1, 4), [1, 4, 2, 3]);
            assert_eq(insert([1, 2, 3], 3, 4), [1, 2, 3, 4]);
            assert_eq(insert([1, 2, 3], 0, 4), [4, 1, 2, 3]);

            would_err('insert([1, 2, 3], 4, 4)') // Index out of bounds
            
            a = [1, 2, 3];
            assert_eq(insert(a, 1, 4), [1, 4, 2, 3]);
            assert_eq(a, [1, 4, 2, 3]);
        ",
    },
    handler = (state, reference) {
        let input = required_arg!(state::input);
        let input_type = input.own_type();
        let mut input = input.as_a::<Array>()?.clone();

        let index = required_arg!(state::index);
        let value = required_arg!(state::value);

        input.insert_at(&index, value.clone())?;

        // Update the array if it references a variable containing an array
        if let Some(reference) = reference {
            if input_type == ValueType::Array {
                reference.update_value_in_parent(state, input.clone().into())?;
            }
        };

        Ok(input.into())
    },
);

define_stdfunction!(
    remove {
        input: Standard::Array,
        index: Standard::Int
    },
    returns = Array,
    docs = {
        category: "Collections",
        description: "Removes the element at the given index in the given array, and returns value",
        ext_description: "
            Removes the element at the given index in the given array.
            If the input is a reference to an array in a variable, the variable is updated.
            If the index is out of bounds, an error is returned.
        ",
        examples: "
            assert_eq(remove([1, 2, 3], 1), 2);
            assert_eq(remove([1, 2, 3], 2), 3);
            assert_eq(remove([1, 2, 3], 0), 1);

            would_err('remove([1, 2, 3], 3)') // Index out of bounds
            
            a = [1, 2, 3];
            assert_eq(remove(a, 1), 2);
            assert_eq(a, [1, 3]);
        ",
    },
    handler = (state, reference) {
        let input = required_arg!(state::input);
        let input_type = input.own_type();
        let mut input = input.as_a::<Array>()?.clone();

        let index = required_arg!(state::index);

        let removed = input.delete_index(&index)?;

        // Update the array if it references a variable containing an array
        if let Some(reference) = reference {
            if input_type == ValueType::Array {
                reference.update_value_in_parent(state, input.clone().into())?;
            }
        };

        Ok(removed)
    },
);

/**********************************************
 *
 * Object Manipulation Functions
 *
 *********************************************/

define_stdfunction!(
    keys { input: Standard::Object },
    returns = Array,
    docs = {
        category: "Collections",
        description: "Returns an array of the keys of the given object",
        ext_description: "
            Returns an array of the keys of the given object.
            The order of the keys is not guaranteed.
        ",
        examples: "
            assert_eq(
                keys({'a': 1, 'b': 2}).sort(),
                ['a', 'b']
            );
            assert_eq(keys({}), []);
        ",
    },
    handler = (state, _reference) {
        let input = required_arg!(state::input).as_a::<Object>()?;
        Ok(Value::from(
            input.keys().iter().cloned().cloned().collect::<Vec<_>>()
        ))
    },
);

define_stdfunction!(
    values { input: Standard::Object },
    returns = Array,
    docs = {
        category: "Collections",
        description: "Returns an array of the values of the given object",
        ext_description: "
            Returns an array of the values of the given object.
            The order of the values is not guaranteed.
        ",
        examples: "
            assert_eq(
                values({'a': 1, 'b': 2}).sort(), 
                [1, 2]
            );
            assert_eq(values({}), []);
        ",
    },
    handler = (state, _reference) {
        let input = required_arg!(state::input).as_a::<Object>()?;
        Ok(Value::from(
            input.values().iter().cloned().cloned().collect::<Vec<_>>()
        ))
    },
);

/**********************************************
 *
 * Array Query Functions
 *
 *********************************************/

define_stdfunction!(
    all { input: Standard::Array },
    returns = Bool,
    docs = {
        category: "Collections",
        description: "Returns true if all elements of the given array are truthy",
        ext_description: "
            Returns true if all elements of the given array evaluate to true.
            If the array is empty, true is returned.
        ",
        examples: "
            assert_eq(all([true, true, true]), true);
            assert_eq(all([0, 1, 2]), false);
            assert_eq(all([]), true);
        ",
    },
    handler = (state, _reference) {
        let input = required_arg!(state::input).as_a::<Array>()?;
        Ok(Value::bool(input.iter().all(|v| v.is_truthy())))
    },
);

define_stdfunction!(
    any { input: Standard::Array },
    returns = Bool,
    docs = {
        category: "Collections",
        description: "Returns true if any element of the given array is truthy",
        ext_description: "
            Returns true if any element of the given array evaluates to true.
            If the array is empty, false is returned.
        ",
        examples: "
            assert_eq(any([true, true, true]), true);
            assert_eq(any([0, 1, 2]), true);
            assert_eq(any([]), false);
        ",
    },
    handler = (state, _reference) {
        let input = required_arg!(state::input).as_a::<Array>()?;
        Ok(Value::bool(input.iter().any(|v| v.is_truthy())))
    },
);

/**********************************************
 *
 * Array Combinators
 *
 *********************************************/

// sort, reverse

define_stdfunction!(
    split {
        input: Standard::Array,
        index: Standard::Int
    },
    returns = Array,
    docs = {
        category: "Collections",
        description: "Splits the given array at the given index, and returns the two resulting arrays",
        ext_description: "
            If the index is out of bounds, an error is returned.
            Returns start-to-index (excluding index) and index-to-end (including index) arrays.
        ",
        examples: "
            assert_eq(split([1, 2, 3, 4], 2), [[1, 2], [3, 4]]);
            assert_eq(split([1, 2, 3, 4], 0), [[], [1, 2, 3, 4]]);
            assert_eq(split([1, 2, 3, 4], 4), [[1, 2, 3, 4], []]);

            would_err('split([1, 2, 3, 4], 5)') // Index out of bounds
        ",
    },
    handler = (state, _reference) {
        let input = required_arg!(state::input).as_a::<Array>()?.clone();
        let index = required_arg!(state::index).as_a::<i64>()?;

        let left = Value::range(0i64 ..= (index - 1i64));
        let left = input.get_indices(&left)?;

        let right = Value::range(index ..= (input.len() as i64 - 1));
        let right = input.get_indices(&right)?;

        Ok(Value::from(vec![left, right]))
    },
);

define_stdfunction!(
    merge {
        left: Standard::Array,
        right: Standard::Array
    },
    returns = Array,
    docs = {
        category: "Collections",
        description: "Merges the two given arrays into a single array, and returns the result",
        ext_description: "
            The two input arrays are concatenated into a single new array.
            The input arrays are not updated.
        ",
        examples: "
            assert_eq(merge([1, 2], [3, 4]), [1, 2, 3, 4]);
            assert_eq(merge([], [3, 4]), [3, 4]);
            assert_eq(merge([1, 2], []), [1, 2]);
        ",
    },
    handler = (state, _reference) {
        let left = required_arg!(state::left).as_a::<Array>()?.clone();
        let right = required_arg!(state::right).as_a::<Array>()?.clone();
        Ok(Value::from(left.iter().chain(right.iter()).cloned().collect::<Vec<_>>()))
    },
);

define_stdfunction!(
    extend {
        left: Standard::Array,
        right: Standard::Array
    },
    returns = Array,
    docs = {
        category: "Collections",
        description: "Appends the elements of the second array to the first array, and returns the result",
        ext_description: "
            The elements of the second array are appended to the first array.
            The first array is updated.
        ",
        examples: "
            assert_eq(extend([1, 2], [3, 4]), [1, 2, 3, 4]);
            assert_eq(extend([], [3, 4]), [3, 4]);
            assert_eq(extend([1, 2], []), [1, 2]);

            a = [1, 2];
            extend(a, [3, 4])
            assert_eq(a, [1, 2, 3, 4]);
        ",
    },
    handler = (state, reference) {
        let left = required_arg!(state::left);
        let input_type = left.own_type();
        let mut left = left.as_a::<Array>()?.clone();
        let right = required_arg!(state::right).as_a::<Array>()?.clone();

        left.extend(right.iter().cloned());

        // Update the array if it references a variable containing an array
        if let Some(reference) = reference {
            if input_type == ValueType::Array {
                reference.update_value_in_parent(state, left.clone().into())?;
            }
        };

        Ok(left.into())
    },
);

define_stdfunction!(
    chunks {
        input: Standard::Array,
        size: Standard::Int
    },
    returns = Array,
    docs = {
        category: "Collections",
        description: "Splits the given array into chunks of the given size, and returns the resulting array of arrays",
        ext_description: "
            Splits the given array into chunks of the given size.
            The last chunk may be smaller than the given size.
        ",
        examples: "
            assert_eq(chunks([1, 2, 3, 4, 5], 2), [[1, 2], [3, 4], [5]]);
            assert_eq(chunks([1, 2, 3, 4, 5], 3), [[1, 2, 3], [4, 5]]);
            assert_eq(chunks([1, 2, 3, 4, 5], 5), [[1, 2, 3, 4, 5]]);
        ",
    },
    handler = (state, _reference) {
        let input = required_arg!(state::input).as_a::<Array>()?.clone();
        let size = required_arg!(state::size).as_a::<i64>()?;

        let result = input.chunks(size as usize).map(|c| Value::from(c.to_vec())).collect::<Vec<_>>();
        Ok(Value::from(result))
    },
);

define_stdfunction!(
    flatten { input: Standard::Array },
    returns = Array,
    docs = {
        category: "Collections",
        description: "Flattens the given array of arrays into a single array, and returns the result",
        ext_description: "
            Flattens the given array of arrays into a single array.
            The input array is not updated.
        ",
        examples: "
            assert_eq(flatten([[1, 2], [3, 4]]), [1, 2, 3, 4]);
            assert_eq(flatten([[1, 2], []]), [1, 2]);
            assert_eq(flatten([[], []]), []);
        ",
    },
    handler = (state, _reference) {
        let input = required_arg!(state::input).as_a::<Array>()?.clone();
        let result = input.iter().flat_map(|v| v.clone().as_a::<Array>().unwrap().iter().cloned().collect::<Vec<_>>()).collect::<Vec<_>>();
        Ok(Value::from(result))
    },
);

define_stdfunction!(
    zip {
        left: Standard::Array,
        right: Standard::Array
    },
    returns = Array,
    docs = {
        category: "Collections",
        description: "Zips the two given arrays into an array of pairs, and returns the result",
        ext_description: "
            Zips the two given arrays into an array of pairs.
            If the input arrays are of different lengths, the resulting array will have the length of the shortest input array.
        ",
        examples: "
            assert_eq(zip([1, 2, 3], [4, 5, 6]), [[1, 4], [2, 5], [3, 6]]);
            assert_eq(zip([1, 2], [4, 5, 6]), [[1, 4], [2, 5]]);
            assert_eq(zip([1, 2, 3], [4, 5]), [[1, 4], [2, 5]]);
        ",
    },
    handler = (state, _reference) {
        let left = required_arg!(state::left).as_a::<Array>()?.clone();
        let right = required_arg!(state::right).as_a::<Array>()?.clone();

        let result = left.iter().zip(right.iter()).map(|(l, r)| Value::from(vec![l.clone(), r.clone()])).collect::<Vec<_>>();
        Ok(Value::from(result))
    },
);

define_stdfunction!(
    zop {
        left: Standard::Array,
        right: Standard::Array
    },
    returns = Array,
    docs = {
        category: "Collections",
        description: "Zips the two given arrays into an array of pairs, and converts in to an object",
        ext_description: "
            Zips the two given arrays into an array of pairs, then converts the result to object
            If the input arrays are of different lengths, the result will have the length of the shortest input array.
            Will fail if any resulting keys would be invalid (collections cannot be used as object keys)
        ",
        examples: "
            assert_eq(zop(['a', 'b', 'c'], [1, 2, 3]), {'a': 1, 'b': 2, 'c': 3});
        ",
    },
    handler = (state, _reference) {
        let left = required_arg!(state::left).as_a::<Array>()?.clone();
        let right = required_arg!(state::right).as_a::<Array>()?.clone();

        let result = left.iter().zip(right.iter()).map(|(l, r)| (l.clone(), r.clone())).collect::<Vec<(_,_)>>();
        let result = Object::try_from(result)?;
        Ok(Value::from(result))
    },
);

define_stdfunction!(
    sort { input: Standard::Array },
    returns = Array,
    docs = {
        category: "Collections",
        description: "Sorts the given array, and returns the result",
        ext_description: "
            The resulting array is sorted in ascending order by value.
            The original array is not updated.
        ",
        examples: "
            assert_eq(sort([3, 1, 2]), [1, 2, 3]);
            assert_eq(sort(['c', 'a', 'b']), ['a', 'b', 'c']);
        ",
    },
    handler = (state, _reference) {
        let input = required_arg!(state::input).as_a::<Array>()?.clone();
        let mut result = input.clone();
        result.sort();
        Ok(result.into())
    },
);

define_stdfunction!(
    reverse { input: Standard::Array },
    returns = Array,
    docs = {
        category: "Collections",
        description: "Reverses the given array, and returns the result",
        ext_description: "
            The resulting array is the reverse of the input array.
            The original array is not updated.
        ",
        examples: "
            assert_eq(reverse([1, 2, 3]), [3, 2, 1]);
            assert_eq(reverse(['a', 'b', 'c']), ['c', 'b', 'a']);
        ",
    },
    handler = (state, _reference) {
        let input = required_arg!(state::input).as_a::<Array>()?.clone();
        let mut result = input.clone();
        result.reverse();
        Ok(result.into())
    },
);
