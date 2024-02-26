# Lavendeux Documentation

This document will provide information on lavendish, a language focused on short, single-line expressions designed to manipulate values.
It was created for use in Lavendeux (https://rscarson.github.io/lavendeux/).

Inputs are a series of expressions separated by a newline, or a `;`.
Lines can optionally end with an @decorator to format the output as a string (see `section 3.2`)

Comments are either `//`, which turns the rest of the line into a comment
Or a block comment bounded by /* and */
## Values and Literals

All expressions in Lavendeux will return a value of a specific type.
These types can be broadly categorized as `numeric`, `collection`, or `string`.

## Numeric Types

The first group of types are classified as numeric; they can all freely be cast to each other.
If a type is downcast to a smaller numeric type, it will be truncated to fit (1.6 becomes 1 for example)

Note; Bool is an outlier here, since any type can be cast to bool:
Truth is determined by equivalence to 0, or by emptiness, depending on the type.

But expressions will always upgrade both values to the highest-order in this list (currency being the highest, bool, the lowest):
- Bool: a single-bit truth value [`true`, `false`]
- Int: One of U8/I8 / U16/I16 / U32/I32 / U64/I64
- Float: A 64bit floating-point number [`1.0`, `.2`, `3e+7`]
- Fixed: A fixed-point decimal value [`1.22D`, `4D`]
- Currency: A fixed-point decimal value with a currency symbol [`$1`, `$2.00`, `3￥`]

Integers can be written in decimal, binary, octal, or hexadecimal, with an optional suffix to specify the type.
If no suffix is provided, the type defaults to i64.

Examples:
- `38_000i32`
- `0xFFu8`
- `0o77i16`
- `077`
- `0b1010_1010_1010_1010i32`
Supported Currency symbols:
$ | ¢ | £ | ¤ | ¥ | ֏ | ؋ | ߾ | ߿ | ৲ | ৳ | ৻ | ૱ | ௹ | ฿ | ៛ | ₠ | ₡ |
₢ | ₣ | ₤ | ₥ | ₦ | ₧ | ₨ | ₩ | ₪ | ₫ | € | ₭ | ₮ | ₯ | ₰ | ₱ | ₲ | ₳ |
₴ | ₵ | ₶ | ₷ | ₸ | ₹ | ₺ | ₻ | ₼ | ₽ | ₾ | ₿ | ꠸ | ﷼ | ﹩ | ＄ | ￠ |
￡ | ￥ | ￦

## Collection Types

The second group are collections, which encapsulate a set of values:
- Array: An ordered collection of values, indexed by integers. Values can be a mix of types.
- Object: An unordered collection of key-value pairs, where keys are any non-collection type and values can be a mix of types.
- Range: An inclusive range of integers, with a start and end value. Start and end can be single characters, or any numeric type.

Attempting to convert non-compound types into one of these will result in a single-value array or object.
Range is the exception - no type can be converted into range, and range can only be converted into an array.

For example, `1 as array` would result in `[1]`,
and `1 as object` would be the equivalent to `[1] as object`, which is `{0: 1}`
(Non-compound types are first cast to array before being transformed into objects)

Attemting to convert a compound value into a non-compound type will only work if the length of the compound value is 1, and will simply extract that value:
- For example, `['a'] as string` would result in `'a'`

## Strings

The last value is string, which any value can be cast to in order to get a string representation.

They are Single or double quote enclosed
With the following supported escape sequences:
- `\'` Single-quote
- `\"` Double-quote
- `\n` Newline
- `\r` Carriage-return
- `\t` Tab
- `\\` Literal backslash

## Type Conversion

You can manually convert between types using `<value> as <type>`, so long as that conversion is allowed:
- Numeric values can always convert to other numeric values [`1 as float`, `true as fixed` or `$1.00 as int` are all valid]
- Non-compound non-numeric values cannot convert into numeric values [`'test' as int` would be an error]
- Any type `T` can be converted to an array as `[T]`, or an object as `{0: T}`
- Conversely, single-element compound values such as `[T]` or `{0: T}` can be freely converted to any type valid for `T`
- All types can be converted to string or bool
- Range can become string, bool or array, but no type can become range

Comparisons and expressions will always try and cooerce both values to the same type, using these rules, in this order:
- If either value is a range, compare the values as arrays
- If either value is an object, compare the values as objects
- If either value is an array, compare the values as arrays
- If either value is an string, compare the values as strings
- If either value is an currency, compare the values as currencies
- If either value is a fixed-point, compare the values as fixed-points
- If either value is an float, compare the values as floats
- If either value is an int, compare the values as ints
- If either value is a bool, compare the values as bools
## Operators and Syntax
### Arithmetic
#### \[+, -, *, /, %, **\]
Performs arithmetic operations on two values.
All but exponentiation are left-associative.

##### Examples
```lavendeux
1 + 2 / 3
2 ** 3
```
### Array Literals
#### \[[ a, b, ... ]\]
A collection of values.
Arrays can contain any type of value, including other arrays.
Arrays are 0-indexed, meaning the first element is at index 0.
The indexing operator (a[b]) can be used to access elements of an array.

##### Examples
```lavendeux
[1, 2, 3, 4, 5]
["Hello", "World"]
[1, [2, 3], 4]
```
### Assignment Operator
#### \[=, +=, -=, *=, /=, %=, **=, &=, |=, ^=, <<=, >>=\]
Assigns a value to a variable, index, or destructuring assignment
Target is either a literal with optional indices, or a destructuring assignment
If an index is empty, a new value will be appended to the array
If the target is a destructuring assignment, the value must be a collection of the same length
If the operator is present, the value will be transformed before assignment

##### Examples
```lavendeux
[a, b] = [1, 2]
a = 1; a += 1
a = [1]; a[] = 2
```
### Bitwise
#### \[|, ^, &, <<, >>\]
A left-associative infix operator that performs bitwise operations on two values.
Values are first converted to integers.
Shifts are arithmetic for signed integers and logical for unsigned integers.
A larger set of bitwise operations are available in the 'bitwise' category of the standard library.

##### Examples
```lavendeux
5 | 3 & 3
5 ^ 3
5 << 3 >> 3
```
### Boolean
#### \[or, and, ==, !=, <=, >=, <, >\]
Performs an infix boolean comparison between two values.
Comparisons are weak, meaning that the types of the values are not checked.
Result are always a boolean value.
And and Or are short-circuiting.
All are left-associative.

##### Examples
```lavendeux
true || false
1 < 2
```
### Cast
#### \[as\]
Casts a value to a different type.
The type can be a string or an identifier.
The operator is right-associative

##### Examples
```lavendeux
5 as float
5 as 'float'
5 as i8
```
### Constant
#### \[pi, e, tau\]
A constant value.
A predefined set of values that are always available.

##### Examples
```lavendeux
pi; e; tau
```
### Decorator
#### \[@name\]
Converts a value to a formatted string.
It calls a function named '@name' with the value as an argument.

##### Examples
```lavendeux
assert_eq(
    5 @float,
    '5.0'
)
```
### Deletion Keyword
#### \[del, delete, unset\]
Deletes a value, function, @decorator, or index
Will return the value deleted (or the function signature if a function was deleted)
Index can be blank to delete the last value in an array, or negative to count from the end
Indices can also be a collection to delete multiple values at once

##### Examples
```lavendeux
a = 2; del a
a = [1]; del a[]
a = {'test': 1}; del a['test']

@dec(x) = 2
del @dec
```
### For
#### \[for <variable> in <iterable> { <block> }, for [<variable> in] <iterable> do <block>\]
A loop that iterates over a range, array, or object.
The variable is optional, and if not provided, the loop will not bind a variable.
The expression will return an array of the results of the block.
Break and skip/continue can be used to exit the loop or skip the current iteration.

##### Examples
```lavendeux
for i in 0..10 { i }
for i in [1, 2, 3] { i }
for i in {'a': 1, 'b': 2} { i }

for 0..10 do '!'
```
### Function Assignment
#### \[name([arg1:type, arg2, ...]) = { ... }\]
Assigns a block of code to a function name.
The function can be called later in the code.
If the function name begins with `@`, it is a decorator and must take in one argument and return a string

Function body can be a block of code or a single expression. The last expression is returned, unless a return statement is used.
Return type or argued types can be specified with `: type`, but are optional.

Arguments will be cooerced to the specified type if provided, as will the return value.
Valid type names are: `u[8-64]`, `i[8-64]`, `float`, `int`, `numeric`, `string`, `array`, `object`, `bool`, `any`.

##### Examples
```lavendeux
// Decorator taking in a number and returning a string
@double(x:numeric) = 2*x
5 @double

// Takes in any 2 numeric values, and returns an integer (i64 by default)
add(a:numeric, b:numeric): int = {
    a + b
}
add(3, 4.5)
```
### Function Call
#### \[name(arg1, arg2, ...), arg1.func(arg2, arg3, ...)\]
Calls a function with the given arguments.
The help() will list all available functions, and can filter by category or function name.

##### Examples
```lavendeux
arr = []
push(arr, 3)
arr.push(3)
help(push)
help(collections)
```
### Identifier
#### \[a, b, c\]
A variable name.
The value of the variable is looked up in the state.

##### Examples
```lavendeux
[a, b, c] = [1, 2, 3]
a; b; c
```
### If
#### \[if <condition> then <block> else <block>, if <condition> {block} else {block}\]
A conditional expression that evaluates a condition and then one of two branches.
body can be either a block or a single expression. The last expression is returned from a block.
Since all expressions in lavendeux return a value, the if expression will return the value of the branch that is executed.
As such, all if expressions must have both a then and an else branch.

##### Examples
```lavendeux
a = 6
if a > 5 { a } else { 5 }
if a == 4 {
    a
} else if a == 5 {
    5
} else {
    6
}
```
### Indexing
#### \[a[b], a[]\]
Accessing elements of a collection.
The indexing operator can be used to access elements of a collection or string.
If the index is a collection, it is used to access multiple elements.
If the index is a string, it is used to access a character.
If the index is blank, it is used to access the last element of the collection.
Negative indices can be used to access elements from the end of the collection.

##### Examples
```lavendeux
[1, 2, 3][0]
[1, 2, 3][0..1]
{ "name": "John", "age": 25 }["name"]
```
### Matching
#### \[contains, matches, is, starts_with, ends_with\]
A set of left-associative boolean operators comparing a collection with a pattern
'is' is a special case that compares type (`value is string` is equivalent `typeof(value) == 'string'`)
starts/ends with are not applicable to objects, which are not ordered

##### Examples
```lavendeux
{'name': 'test'} contains 'name'
'hello' matches 'ell'
'hello' is string
'hello' starts_with 'hel'
[1, 2] endswith 2
```
### Object Literals
#### \[{ key: value, ... }\]
A collection of key-value pairs.
Values can contain any type, including other objects.
Keys can be any non-collection type
The indexing operator (a[b]) can be used to access elements of an object.

##### Examples
```lavendeux
{ "name": "John", "age": 25 }
{ "name": "John", "address": { "city": "New York", "state": "NY" } }
```
### Range Literals
#### \[first..last\]
A range of values.
Ranges can be used to create arrays of numbers or characters.
Ranges are inclusive, meaning the start and end values are included in the array.
Start and end values must be of the same type, and start must be <= end.
Character ranges are inclusive and can only be used with single-character strings.

##### Examples
```lavendeux
1..5
'a'..'z'
```
### Ternary
#### \[condition ? then : else\]
A right-associative ternary operator.
The condition is evaluated first, then either the then or else branch is evaluated.

##### Examples
```lavendeux
true ? 1 : 2
```
### Unary Bitwise Not
#### \[~\]
A prefix operator that performs bitwise NOT on a value.
The value is first converted to an integer.
A larger set of bitwise operations are available in the 'bitwise' category of the standard library.

##### Examples
```lavendeux
~5
```
### Unary Boolean Not
#### \[not\]
Negates a boolean value.
If the value is not a boolean, it is cooerced to boolean first.

##### Examples
```lavendeux
!true == false
!'test' == false
!0 == true
```
### Unary Increment/Decrement
#### \[++, --\]
Increments or decrements a variable by 1.
##### Examples
```lavendeux
a = 0
assert_eq(a++, 0)
assert_eq(--a, 0)
```
### Unary Negation
#### \[-\]
Negates a value.
##### Examples
```lavendeux
-1
```
### match
#### \[match <value> { <condition> => <block>, _ => <block> }\]
A conditional expression that evaluates a value and then one of several cases.
match blocks must be exhaustive, and therefore must end in a default case

##### Examples
```lavendeux
a = 6
match a {
    5 => { 'five' },
    6 => { 'six' },
    _ => { 'other' }
}
```
## Functions
### API Functions
#### `api_add(name:string, endpoint) -> string`
Registers an API
This function registers an API with the system. The API can then be used to make requests to the specified endpoint.  
The endpoint can be a string, or an object with the properties [ base_url, headers, description, examples, auth_key]  
Use the 'api_get' and 'api_post' functions to make requests to the registered API  
  
##### Examples
```lavendeux
api_add('ipify', 'https://api.ipify.org')
assert( api_list() contains 'ipify' )
```

------------
#### `api_all() -> object`
Details all registered APIs
This function returns an object containing the names and endpoints of all registered APIs  
  
##### Examples
```lavendeux
api_all()['chatgpt']['base_url']
```

------------
#### `api_get(name:string, [path:string]) -> string`
Performs a GET request to a registered API
This function performs a GET request to the specified path of a registered API.  
The path is appended to the base URL of the API.  
  
##### Examples
```lavendeux
api_get('ipify')
api_get('ipify', '/?format=json')
```

------------
#### `api_key(name:string, auth_key:string) -> string`
Sets an authentication key for a registered API
This function sets an authentication key for a registered API.  
The key will be used in the 'Authorization' header of requests to the API.  
  
##### Examples
```lavendeux
api_key('chatgpt', 'my_super_secret_api_key')
assert_eq( api_all()['chatgpt']['auth_key'], 'my_super_secret_api_key' )
```

------------
#### `api_list() -> object`
Lists all registered APIs
This function returns an array containing the names of all registered APIs  
  
##### Examples
```lavendeux
assert( api_list() contains 'chatgpt' )
```

------------
#### `api_post(name:string, body:string, [path:string]) -> string`
Performs a POST request to a registered API
This function performs a POST request to the specified path of a registered API.  
The path is appended to the base URL of the API.  
  
##### Examples
```lavendeux
api_post('ipify', '{"name"="john"}', 'format=json')
```

------------
#### `api_rem(name:string) -> string`
Unregisters an API
This function unregisters an API with the system, and returns its name  
The API can no longer be used to make requests  
  
##### Examples
```lavendeux
api_rem('ipify')
assert( !(api_list() contains 'ipify') )
```

------------
#### `chatgpt(prompt:string) -> string`
Performs a request to the ChatGPT API
This function performs a request to the ChatGPT 3.5 API, using the specified prompt.  
  
##### Examples
```lavendeux
api_key('chatgpt', 'my_super_secret_api_key')
chatgpt('What is the meaning of life?')
```

### Bitwise Functions
#### `and(left:int, right:int) -> int`
Performs a bitwise and operation on two integers
Floats and Fixed-point numbers will be truncated to integers before the operation is performed.  
  
##### Examples
```lavendeux
assert_eq(0b0100, and(0b1100, 0b0110))
```

------------
#### `llshift(value:int, shift:int) -> int`
Performs a logical bitwise left shift operation on an integer
Floats and Fixed-point numbers will be truncated to integers before the operation is performed.  
Will always ignore the sign bit.  
  
##### Examples
```lavendeux
assert_eq(
    0b1000_0010i8,
    llshift(0b0100_0001i8, 1)
)
```

------------
#### `lrshift(value:int, shift:int) -> int`
Performs a logical bitwise right shift operation on an integer
Floats and Fixed-point numbers will be truncated to integers before the operation is performed.  
Will always ignore the sign bit.  
  
##### Examples
```lavendeux
assert_eq(
    0b0100_0000i8,
    lrshift(0b1000_0001i8, 1)
)
```

------------
#### `not(value:int) -> int`
Performs a bitwise NOT operation on an integer
Floats and Fixed-point numbers will be truncated to integers before the operation is performed.  
  
##### Examples
```lavendeux
assert_eq(0b1111_1111u8, not(0b0000_0000u8))
```

------------
#### `or(left:int, right:int) -> int`
Performs a bitwise or operation on two integers
Floats and Fixed-point numbers will be truncated to integers before the operation is performed.  
  
##### Examples
```lavendeux
assert_eq(0b1110, or(0b1100, 0b0110))
```

------------
#### `xor(left:int, right:int) -> int`
Performs a bitwise xor operation on two integers
Floats and Fixed-point numbers will be truncated to integers before the operation is performed.  
  
##### Examples
```lavendeux
assert_eq(0b1010, xor(0b1100, 0b0110))
```

### Collections Functions
#### `all(input:array) -> bool`
Returns true if all elements of the given array are truthy
Returns true if all elements of the given array evaluate to true.  
If the array is empty, true is returned.  
  
##### Examples
```lavendeux
assert_eq(all([true, true, true]), true);
assert_eq(all([0, 1, 2]), false);
assert_eq(all([]), true);
```

------------
#### `any(input:array) -> bool`
Returns true if any element of the given array is truthy
Returns true if any element of the given array evaluates to true.  
If the array is empty, false is returned.  
  
##### Examples
```lavendeux
assert_eq(any([true, true, true]), true);
assert_eq(any([0, 1, 2]), true);
assert_eq(any([]), false);
```

------------
#### `chunks(input:array, size:int) -> array`
Splits the given array into chunks of the given size, and returns the resulting array of arrays
Splits the given array into chunks of the given size.  
The last chunk may be smaller than the given size.  
  
##### Examples
```lavendeux
assert_eq(chunks([1, 2, 3, 4, 5], 2), [[1, 2], [3, 4], [5]]);
assert_eq(chunks([1, 2, 3, 4, 5], 3), [[1, 2, 3], [4, 5]]);
assert_eq(chunks([1, 2, 3, 4, 5], 5), [[1, 2, 3, 4, 5]]);
```

------------
#### `dequeue(input:array) -> array`
Removes and returns the first element of the given array
Removes the first element from the given array and returns it.  
If the array is empty, an error is returned.  
If the input is a reference to an array in a variable, the variable is updated.  
This function is less performant than `pop` for large arrays, as it requires shifting all elements by one position.  
  
##### Examples
```lavendeux
assert_eq(dequeue([1, 2, 3]), 1);
would_err('dequeue([]') // Array is empty, so an error is returned

a = [1, 2];
assert_eq(dequeue(a), 1);
assert_eq(a, [2]);
```

------------
#### `enqueue(input:array, value) -> array`
Appends the given value to the start of the given array, and returns the result
Appends the given value to the start of the given array.  
If the input is a reference to an array in a variable, the variable is updated.  
This function is less performant than `push` for large arrays, as it requires shifting all elements by one position.  
  
##### Examples
```lavendeux
assert_eq(enqueue([1, 2], 3), [3, 1, 2])
assert_eq(enqueue([], 3), [3])

a = [1]
assert_eq(enqueue(a, 2), [2, 1])
assert_eq(a, [2, 1])
```

------------
#### `extend(left:array, right:array) -> array`
Appends the elements of the second array to the first array, and returns the result
The elements of the second array are appended to the first array.  
The first array is updated.  
  
##### Examples
```lavendeux
assert_eq(extend([1, 2], [3, 4]), [1, 2, 3, 4]);
assert_eq(extend([], [3, 4]), [3, 4]);
assert_eq(extend([1, 2], []), [1, 2]);

a = [1, 2];
extend(a, [3, 4])
assert_eq(a, [1, 2, 3, 4]);
```

------------
#### `first(input:array) -> any`
Returns the first element of the given array
Coerces its argument to an array and returns the first element.  
If the resulting array is empty, an error is returned.  
  
##### Examples
```lavendeux
assert_eq(first([1, 2, 3]), 1);
assert_eq(first(3),         3); // equivalent to first([3])

would_err('first([])'); // Array is empty, so an error is returned
```

------------
#### `flatten(input:array) -> array`
Flattens the given array of arrays into a single array, and returns the result
Flattens the given array of arrays into a single array.  
The input array is not updated.  
  
##### Examples
```lavendeux
assert_eq(flatten([[1, 2], [3, 4]]), [1, 2, 3, 4]);
assert_eq(flatten([[1, 2], []]), [1, 2]);
assert_eq(flatten([[], []]), []);
```

------------
#### `insert(input:array, index:int, value) -> array`
Inserts the given value at the given index in the given array, and returns the result
Inserts the given value at the given index in the given array.  
If the input is a reference to an array in a variable, the variable is updated.  
If the index is out of bounds, an error is returned.  
  
##### Examples
```lavendeux
assert_eq(insert([1, 2, 3], 1, 4), [1, 4, 2, 3]);
assert_eq(insert([1, 2, 3], 3, 4), [1, 2, 3, 4]);
assert_eq(insert([1, 2, 3], 0, 4), [4, 1, 2, 3]);

would_err('insert([1, 2, 3], 4, 4)') // Index out of bounds

a = [1, 2, 3];
assert_eq(insert(a, 1, 4), [1, 4, 2, 3]);
assert_eq(a, [1, 4, 2, 3]);
```

------------
#### `is_empty(input) -> bool`
Returns true if the given array or object is empty
For arrays and objects, this function returns true if the array or object has no elements.  
For strings, it returns true if the string is empty.  
For all other types it will return false  
  
##### Examples
```lavendeux
assert_eq(is_empty([]),     true);
assert_eq(is_empty({}),     true);
assert_eq(is_empty('test'), false);
assert_eq(is_empty(38),     false);
```

------------
#### `keys(input:object) -> array`
Returns an array of the keys of the given object
Returns an array of the keys of the given object.  
The order of the keys is not guaranteed.  
  
##### Examples
```lavendeux
assert_eq(keys({'a': 1, 'b': 2}), ['a', 'b']);
assert_eq(keys({}), []);
```

------------
#### `last(input:array) -> any`
Returns the last element of the given array
Coerces its argument to an array and returns the last element.  
If the resulting array is empty, an error is returned.  
  
##### Examples
```lavendeux
assert_eq(last([1, 2, 3]), 3);
assert_eq(last(3),         3); // equivalent to last([3])

would_err('last([])'); // Array is empty, so an error is returned
```

------------
#### `len(input) -> int`
Returns the length of the given array or object
For arrays and objects, this function returns the number of elements in the array or object.  
For strings, it returns the number of characters.  
For all other types it will return 1  
  
##### Examples
```lavendeux
assert_eq(len('test'),       4);
assert_eq(len([1, 2, 3]),    3);
assert_eq(len({'a': 1, 'b': 2}), 2);
assert_eq(len(38),           1);
```

------------
#### `merge(left:array, right:array) -> array`
Merges the two given arrays into a single array, and returns the result
The two input arrays are concatenated into a single new array.  
The input arrays are not updated.  
  
##### Examples
```lavendeux
assert_eq(merge([1, 2], [3, 4]), [1, 2, 3, 4]);
assert_eq(merge([], [3, 4]), [3, 4]);
assert_eq(merge([1, 2], []), [1, 2]);
```

------------
#### `pop(input:array) -> any`
Removes and returns the last element of the given array
Removes the last element from the given array and returns it.  
If the array is empty, an error is returned.  
If the input is a reference to an array in a variable, the variable is updated.  
  
##### Examples
```lavendeux
assert_eq(pop([1, 2, 3]), 3);
would_err('pop([]') // Array is empty, so an error is returned

a = [1];
assert_eq(pop(a), 1);
assert_eq(a, []);
```

------------
#### `push(input:array, value) -> array`
Appends the given value to the end of the given array, and returns the result
Appends the given value to the end of the given array.  
If the input is a reference to an array in a variable, the variable is updated.  
  
##### Examples
```lavendeux
assert_eq(push([1, 2], 3), [1, 2, 3]);
assert_eq(push([], 3), [3]);

a = [1];
assert_eq(push(a, 2), [1, 2]);
assert_eq(a, [1, 2]);
```

------------
#### `remove(input:array, index:int) -> array`
Removes the element at the given index in the given array, and returns value
Removes the element at the given index in the given array.  
If the input is a reference to an array in a variable, the variable is updated.  
If the index is out of bounds, an error is returned.  
  
##### Examples
```lavendeux
assert_eq(remove([1, 2, 3], 1), 2);
assert_eq(remove([1, 2, 3], 2), 3);
assert_eq(remove([1, 2, 3], 0), 1);

would_err('remove([1, 2, 3], 3)') // Index out of bounds

a = [1, 2, 3];
assert_eq(remove(a, 1), 2);
assert_eq(a, [1, 3]);
```

------------
#### `reverse(input:array) -> array`
Reverses the given array, and returns the result
The resulting array is the reverse of the input array.  
The original array is not updated.  
  
##### Examples
```lavendeux
assert_eq(reverse([1, 2, 3]), [3, 2, 1]);
assert_eq(reverse(['a', 'b', 'c']), ['c', 'b', 'a']);
```

------------
#### `sort(input:array) -> array`
Sorts the given array, and returns the result
The resulting array is sorted in ascending order by value.  
The original array is not updated.  
  
##### Examples
```lavendeux
assert_eq(sort([3, 1, 2]), [1, 2, 3]);
assert_eq(sort(['c', 'a', 'b']), ['a', 'b', 'c']);
```

------------
#### `split(input:array, index:int) -> array`
Splits the given array at the given index, and returns the two resulting arrays
If the index is out of bounds, an error is returned.  
Returns start-to-index (excluding index) and index-to-end (including index) arrays.  
  
##### Examples
```lavendeux
assert_eq(split([1, 2, 3, 4], 2), [[1, 2], [3, 4]]);
assert_eq(split([1, 2, 3, 4], 0), [[], [1, 2, 3, 4]]);
assert_eq(split([1, 2, 3, 4], 4), [[1, 2, 3, 4], []]);

would_err('split([1, 2, 3, 4], 5)') // Index out of bounds
```

------------
#### `values(input:object) -> array`
Returns an array of the values of the given object
Returns an array of the values of the given object.  
The order of the values is not guaranteed.  
  
##### Examples
```lavendeux
assert_eq(values({'a': 1, 'b': 2}), [1, 2]);
assert_eq(values({}), []);
```

------------
#### `zip(left:array, right:array) -> array`
Zips the two given arrays into an array of pairs, and returns the result
Zips the two given arrays into an array of pairs.  
If the input arrays are of different lengths, the resulting array will have the length of the shortest input array.  
  
##### Examples
```lavendeux
assert_eq(zip([1, 2, 3], [4, 5, 6]), [[1, 4], [2, 5], [3, 6]]);
assert_eq(zip([1, 2], [4, 5, 6]), [[1, 4], [2, 5]]);
assert_eq(zip([1, 2, 3], [4, 5]), [[1, 4], [2, 5]]);
```

------------
#### `zop(left:array, right:array) -> array`
Zips the two given arrays into an array of pairs, and converts in to an object
Zips the two given arrays into an array of pairs, then converts the result to object  
If the input arrays are of different lengths, the result will have the length of the shortest input array.  
Will fail if any resulting keys would be invalid (collections cannot be used as object keys)  
  
##### Examples
```lavendeux
assert_eq(zop(['a', 'b', 'c'], [1, 2, 3]), {'a': 1, 'b': 2, 'c': 3});
```

### Cryptographic Functions
#### `md5(input:string) -> string`
Returns the md5 hash of a given string
Will return an unsalted md5 hash of the input string.  
##### Examples
```lavendeux
assert_eq(
    md5('hello'),
    '5D41402ABC4B2A76B9719D911017C592'
)
```

------------
#### `sha256(input:string) -> string`
Returns the sha256 hash of a given string
Will return an unsalted sha256 hash of the input string.  
##### Examples
```lavendeux
assert_eq(
    sha256('hello'),
    '2CF24DBA5FB0A30E26E83B2AC5B9E29E1B161E5C1FA7425E73043362938B9824'
)
```

------------
#### `sha512(input:string) -> string`
Returns the sha512 hash of a given string
Will return an unsalted sha512 hash of the input string.  
##### Examples
```lavendeux
assert_eq(
    sha512('hello'),
    '9B71D224BD62F3785D96D46AD3EA3D73319BFBC2890CAADAE2DFF72519673CA72323C3D99BA5C11D7C7ACC6E14B8C5DA0C4663475C2E5C3ADEF46F73BCDEC043'
)
```

### Decorators Functions
#### `@aud(input:numeric) -> string`
Interprets a number as a AUD amount
Includes a dollar sign and two decimal places.  
##### Examples
```lavendeux
assert_eq(
    100 @aud,
    '$100.00'
)
```

------------
#### `@bin(input:numeric) -> string`
Base 2 number formatting, such as 0b101
Converts a number to a binary string. The output will be prefixed with '0b' with a length based on the input type.  
##### Examples
```lavendeux
assert_eq(
    255 @bin,
    '0b11111111'
)
```

------------
#### `@bool(input) -> string`
Boolean formatting
Converts a number to a boolean string.  
##### Examples
```lavendeux
assert_eq(
    1 @bool,
    'true'

)
```

------------
#### `@cad(input:numeric) -> string`
Interprets a number as a CAD amount
Includes a dollar sign and two decimal places.  
##### Examples
```lavendeux
assert_eq(
    100 @cad,
    '$100.00'
)
```

------------
#### `@cny(input:numeric) -> string`
Interprets a number as a CNY amount
Includes a yuan sign and two decimal places.  
##### Examples
```lavendeux
assert_eq(
    100 @cny,
    '¥100.00'
)
```

------------
#### `@eur(input:numeric) -> string`
Interprets a number as a Euro amount
Includes a euro sign and two decimal places.  
##### Examples
```lavendeux
assert_eq(
    100 @eur,
    '€100.00'
)
```

------------
#### `@float(input:numeric) -> string`
Floating point number formatting
Converts a number to a floating point string.  
##### Examples
```lavendeux
assert_eq(
    1.0 @float,
    '1.0'
)
```

------------
#### `@gbp(input:numeric) -> string`
Interprets a number as a GBP amount
Includes a pound sign and two decimal places.  
##### Examples
```lavendeux
assert_eq(
    100 @gbp,
    '£100.00'
)
```

------------
#### `@hex(input:numeric) -> string`
Base 16 number formatting, such as 0xFF
Converts a number to a hexadecimal string. The output will be prefixed with '0x' with a length based on the input type.  
##### Examples
```lavendeux
assert_eq(
    255 @hex,
    '0xff'
)
```

------------
#### `@inr(input:numeric) -> string`
Interprets a number as a INR amount
Includes a rupee sign and two decimal places.  
##### Examples
```lavendeux
assert_eq(
    100 @inr,
    '₹100.00'
)
```

------------
#### `@int(input:numeric) -> string`
Integer number formatting
Converts a number to an integer string.  
##### Examples
```lavendeux
assert_eq(
    1000000 @int,
    '1000000'
)
```

------------
#### `@jpy(input:numeric) -> string`
Interprets a number as a JPY amount
Includes a yen sign and no decimal places.  
##### Examples
```lavendeux
assert_eq(
    100 @jpy,
    '¥100'
)
```

------------
#### `@oct(input:numeric) -> string`
Base 8 number formatting, such as 0o77
Converts a number to an octal string. The output will be prefixed with '0o' with a length based on the input type.  
##### Examples
```lavendeux
assert_eq(
    255 @oct,
    '0o377'
)
```

------------
#### `@ord(input:numeric) -> string`
Interprets an integer as an ordinal number
This function will append the appropriate suffix to the input number.  
##### Examples
```lavendeux
assert_eq(
    123 @ord,
    '123rd'
)
```

------------
#### `@percent(input:numeric) -> string`
Interprets a number as a percentage
This function will append a percentage sign to the input number times 100  
##### Examples
```lavendeux
assert_eq(
    0.123 @percent,
    '12.3%'

)
```

------------
#### `@roman(input:numeric) -> string`
Interprets an integer as a roman numeral
Like the roman system before it; this function only supports numbers up to 3999.  
##### Examples
```lavendeux
assert_eq(
    123 @roman,
    'CXXIII'
)
```

------------
#### `@rub(input:numeric) -> string`
Interprets a number as a RUB amount
Includes a ruble sign and two decimal places.  
##### Examples
```lavendeux
assert_eq(
    100 @rub,
    '₽100.00'
)
```

------------
#### `@sci(input:numeric) -> string`
Scientific notation
Converts a floating point number to sci notation.  
##### Examples
```lavendeux
assert_eq(
    1000000.0 @sci,
    '1e6'
)
```

------------
#### `@usd(input:numeric) -> string`
Interprets a number as a USD amount
Includes a dollar sign and two decimal places.  
##### Examples
```lavendeux
assert_eq(
    100 @usd,
    '$100.00'
)
```

------------
#### `@utc(input:numeric) -> string`
Interprets an integer as a timestamp, and formats it in UTC standard
This function will convert the input number to a UTC timestamp.  
##### Examples
```lavendeux
assert_eq(
    123 @utc,
    '1970-01-01T00:02:03Z'
)
```

### Development Functions
#### `tail(file:string, [lines:int]) -> array`
Returns the last <lines> lines from a given file
If <lines> is not specified, the function will return the last line of the file.  
##### Examples
```lavendeux
lines = tail('.gitignore')
assert_eq(
    lines,
    ['/Cargo.lock']
)
```

------------
#### `time() -> float`
Returns a unix timestamp for the current system time
Returns a unix timestamp for the current system time.  
The timestamp is a floating point number representing the number of seconds since the Unix epoch.  
##### Examples
```lavendeux
assert(
    time() > 0
)
```

### Math Functions
#### `abs(value:numeric) -> numeric`
Returns the absolute value of a number
The function will return the absolute value of the input number.  
  
##### Examples
```lavendeux
assert_eq(
    abs(-5),
    5
)
```

------------
#### `ceil(value:numeric) -> numeric`
Rounds a number up to the nearest whole number
The function will round the input number up to the nearest whole number.  
If the input number is already a whole number, the function will return the input number.  
  
##### Examples
```lavendeux
assert_eq(
    ceil(1.5),
    2.0
)
```

------------
#### `floor(value:numeric) -> numeric`
Rounds a number down to the nearest whole number
The function will round the input number down to the nearest whole number.  
If the input number is already a whole number, the function will return the input number.  
  
##### Examples
```lavendeux
assert_eq(
    floor(1.5),
    1.0
)
```

------------
#### `ilog2(value:int) -> numeric`
Returns the base-2 logarithm of a number, rounded down to the nearest whole number
  
##### Examples
```lavendeux
assert_eq(
    ilog2(8),
    3
)
```

------------
#### `ln(value:numeric) -> numeric`
Returns the natural logarithm of a number
  
##### Examples
```lavendeux
assert_eq(
    ln(2.718281828459045),
    1.0
)
```

------------
#### `log(value:numeric, [base:numeric]) -> numeric`
Returns the logarithm of a number to a given base
  
##### Examples
```lavendeux
assert_eq(
    log(8, 2),
    3.0
)
```

------------
#### `log10(value:numeric) -> numeric`
Returns the base-10 logarithm of a number
  
##### Examples
```lavendeux
assert_eq(
    log10(100),
    2
)
```

------------
#### `log2(value:numeric) -> numeric`
Returns the base-2 logarithm of a number
  
##### Examples
```lavendeux
assert_eq(
    log2(8),
    3
)
```

------------
#### `max(options:array) -> numeric`
Returns the largest value in the given array
The array can contain any number of elements, and they can be of any type.  
Since all values in lavendeux are comparable, the function will work with any type of array.  
  
##### Examples
```lavendeux
assert_eq(
    max([1, 2, 3, 4, 5]),
    5
)
```

------------
#### `min(options:array) -> numeric`
Returns the smallest value in the given array
The array can contain any number of elements, and they can be of any type.  
Since all values in lavendeux are comparable, the function will work with any type of array.  
  
##### Examples
```lavendeux
assert_eq(
    min([1, 2, 3, 4, 5]),
    1
)
```

------------
#### `root(value:numeric, root:numeric) -> numeric`
Returns the nth root of a number
  
##### Examples
```lavendeux
assert_eq(
    root(8, 3),
    2.0
)
```

------------
#### `round(value:numeric, [precision:int]) -> numeric`
Rounds a number to the nearest whole number
The function will round the input number to the nearest whole number.  
If the input number is already a whole number, the function will return the input number.  
  
##### Examples
```lavendeux
assert_eq(
    round(1.5),
    2.0
)
```

------------
#### `sqrt(value:numeric) -> numeric`
Returns the square root of a number
  
##### Examples
```lavendeux
assert_eq(
    sqrt(9),
    3.0
)
```

### Network Functions
#### `get(url:string, [headers:object]) -> string`
Performs an HTTP GET request
This function performs an HTTP GET request to the specified URL.  
If the request fails, this function will return an error or time out  
  
##### Examples
```lavendeux
str_out = get('https://jsonplaceholder.typicode.com/users')
obj_out = get('https://jsonplaceholder.typicode.com/users', {
    'Content-Type': 'application/json'
})
assert(str_out is string)
assert(obj_out is array)
```

------------
#### `post(url:string, body:string, [headers:object]) -> string`
Performs an HTTP POST request
This function performs an HTTP POST request to the specified URL.  
If the request fails, this function will return an error or time out  
  
##### Examples
```lavendeux
obj_out = post(
    'https://jsonplaceholder.typicode.com/users', 
    '{"name": "John Doe"}',
    {'Content-Type': 'application/json'}
)
```

------------
#### `resolve(hostname:string) -> string`
Resolves a hostname to an IP address
This function uses the system's DNS resolver to resolve a hostname to an IP address.  
If the hostname cannot be resolved, this function will return an error, or time out  
  
##### Examples
```lavendeux
resolve('example.com')
```

### Random Functions
#### `choose(options:array) -> string`
Returns a random element from a given array
Uses a uniform distribution to select a random element from the input array.  
##### Examples
```lavendeux
s = ['a', 'b', 'c']
assert(
    s contains choose(s)
)
```

------------
#### `rand([range:range]) -> numeric`
Returns a random number within a given range, or between 0 and 1 if no range is specified.
If no range is specified, the function will return a random number between 0 and 1.  
If a range is specified, the function will return a random number within that range.  
  
##### Examples
```lavendeux
r = rand(0..10)
assert(
    r >= 0 && r <= 10
)
```

### String Functions
#### `base64_decode(s:string) -> string`
Decodes a base64 string into a string.
This function will handle all Unicode characters.  
##### Examples
```lavendeux
assert_eq('hello world', base64_decode('aGVsbG8gd29ybGQ='))
```

------------
#### `base64_encode(s:string) -> string`
Encodes a string into base64
This function will handle all Unicode characters.  
##### Examples
```lavendeux
assert_eq('aGVsbG8gd29ybGQ=', base64_encode('hello world'))
```

------------
#### `chr(i:i64) -> string`
Returns a string containing the character represented by the Unicode code point.
This is the complement of ord(); Output from one is valid input for the other.  
  
##### Examples
```lavendeux
assert_eq('a', chr(97))
```

------------
#### `concat(parts:array, [joiner:string]) -> string`
Concatenates an array of values into a single string.
Converts all its arguments to strings and then concatenates them.  
If a joiner is provided, it will be used to separate the parts.  
  
##### Examples
```lavendeux
assert_eq('hello world', concat(['hello', ' ', 'world']))
```

------------
#### `format(s:string, args:array) -> string`
Formats a string using positional arguments.
The 2nd argument is an array of values to be consumed in order  
##### Examples
```lavendeux
assert_eq('hello world', format('hello {}', ['world']))
```

------------
#### `lowercase(s:string) -> string`
Converts a string to lowercase.
This function is locale-insensitive and will handle all Unicode characters.  
##### Examples
```lavendeux
assert_eq('hello', lowercase('HELLO'))
```

------------
#### `ord(c:string) -> i64`
Returns the Unicode code point of the character at the specified index.
Will always return a 32bit value, regardless of the width of the character.  
This is the complement of chr(); Output from one is valid input for the other.  
  
##### Examples
```lavendeux
assert_eq(97u32, ord('a'))
```

------------
#### `prettyjson(s:object) -> string`
Formats a JSON string for human readability.
This function will handle all Unicode characters.  
##### Examples
```lavendeux
assert_eq(
    '{
  "hello": "world"
}',
    prettyjson({"hello": "world"})
)
```

------------
#### `repeat(s:string, n:i64) -> string`
Repeats a string a specified number of times.
This function is locale-insensitive and will handle all Unicode characters.  
##### Examples
```lavendeux
assert_eq('hellohellohello', repeat('hello', 3))
```

------------
#### `replace(s:string, from:string, to:string) -> string`
Replaces all occurrences of a substring within a string with another string.
This function is locale-insensitive and will handle all Unicode characters.  
##### Examples
```lavendeux
assert_eq('hello world', replace('hello there', 'there', 'world'))
```

------------
#### `trim(s:string) -> string`
Removes leading and trailing whitespace from a string.
This function is locale-insensitive and will handle all Unicode characters.  
##### Examples
```lavendeux
assert_eq('hello', trim('  hello  '))
```

------------
#### `trim_end(s:string) -> string`
Removes trailing whitespace from a string.
This function is locale-insensitive and will handle all Unicode characters.  
##### Examples
```lavendeux
assert_eq('  hello', trim_end('  hello  '))
```

------------
#### `trim_start(s:string) -> string`
Removes leading whitespace from a string.
This function is locale-insensitive and will handle all Unicode characters.  
##### Examples
```lavendeux
assert_eq('hello  ', trim_start('  hello  '))
```

------------
#### `uppercase(s:string) -> string`
Converts a string to uppercase.
This function is locale-insensitive and will handle all Unicode characters.  
##### Examples
```lavendeux
assert_eq('HELLO', uppercase('hello'))
```

------------
#### `url_decode(s:string) -> string`
Decodes a URL-safe string into a normal string.
This function will handle all Unicode characters.  
##### Examples
```lavendeux
assert_eq('hello world', url_decode('hello%20world'))
```

------------
#### `url_encode(s:string) -> string`
Encodes a string as a URL-safe string.
This function will handle all Unicode characters.  
##### Examples
```lavendeux
assert_eq('hello%20world', url_encode('hello world'))
```

### System Functions
#### `assert(condition) -> any`
Throws an error if the condition is false
Does a weak-comparison to boolean, so 0, '', [], etc. are all considered false.  
  
##### Examples
```lavendeux
assert(true)
assert( would_err('assert(false)') )
```

------------
#### `assert_eq(condition, expected) -> any`
Asserts that 2 values are equal
Raises an error if the condition is not equal to the expected value.  
Also verifies type, as opposed to the `==` operator, which uses weak typing.  
use assert(a == b) if you want to compare values without checking their types.  
  
##### Examples
```lavendeux
assert_eq(true, true)
assert_eq( true, would_err('assert_eq(1, true)') )
```

------------
#### `assign(name:string, value) -> any`
Assigns a variable in the current scope
Writes a value to the current scope, leaving other scopes unchanged.  
  
##### Examples
```lavendeux
x = 5
if true then {
    assign('x', 6)
    assert_eq(6, x)
} else { 0 }
assert_eq(5, x)
```

------------
#### `assign_global(name:string, value) -> any`
Assigns a variable in the top-level scope
Writes a value to the top-level scope, leaving other scopes unchanged.  
  
##### Examples
```lavendeux
x = 5
if true then {
    assign_global('x', 6)
    assert_eq(6, x)
} else { 0 }
assert_eq(6, x)
```

------------
#### `call_function(name:string, args:array) -> any`
Calls a function or @decorator by name with the given arguments
If the name begins with '@', it will be treated as a decorator.  
Maps the given object to the function's arguments and calls the function.  
Important note: Functions that take in a reference, such as pop/push etc, will act by-value and not modify the original object.  
  
##### Examples
```lavendeux
@test(x) = x
assert_eq('5', call_function('@test', {'x': 5}))
```

------------
#### `debug(message:string) -> any`
Prints a debug message to the console
The message will be both written to stdout, and returned as a string.  
If the parser is not attached to a console, it will not be visible.  
  
##### Examples
```lavendeux
debug("This is a debug message")
```

------------
#### `error(message:string) -> any`
Throws an error with the given message
Throws an exception with a custom message. The error's source will be the line where the error was thrown.  
  
##### Examples
```lavendeux
would_err('error("This is an error")')
```

------------
#### `eval(expression:string) -> any`
Evaluates a string as a Lavendeux expression and returns the result
The string will be interpreted as a script and evaluated in it's own scope.  
If there are multiple lines, an array of values will be returned.  
  
##### Examples
```lavendeux
assert_eq(5, eval('2 + 3'))
assert_eq([6, 6], eval('x = 6; x'))
assert_eq([1, 2, 3], eval('1\n2\n3'))
```

------------
#### `generate_documentation() -> string`
Generates documentation for all standard library functions
Returns a markdown-formatted string containing documentation for all standard library functions.  
  
##### Examples
```lavendeux
generate_documentation()
```

------------
#### `include(filename:string) -> any`
Evaluates a file as a Lavendeux expression and returns the result
The file will be interpreted as a script and evaluated in it's own scope.  
Returns an empty string in all cases.  
  
##### Examples
```lavendeux
include('examples/stdlib_example.lav')
```

------------
#### `typeof(value) -> string`
Returns the type of its input
Returns the type of the given value as a string.  
  
##### Examples
```lavendeux
assert_eq('string', typeof('hello'))
assert_eq('i64', typeof(5))
assert_eq('object', typeof({}))
```

------------
#### `variables() -> object`
Returns the currently defined variables
Returns a map of all the variables currently defined in the current scope.  
  
##### Examples
```lavendeux
x = 5; y = 6
state = variables()
assert_eq(5, state['x'])
assert_eq(6, state['y'])
```

------------
#### `would_err(expression:string) -> bool`
Returns true if the given expression would raise an error
Returns true if expression given by the string would raise an error, false otherwise.  
This is useful for testing error messages.  
  
##### Examples
```lavendeux
assert_eq( false, would_err('1 + 1') )
assert_eq( true, would_err('1 + asparagus') )
```

### Trigonometry Functions
#### `acos(n:numeric) -> float`
Calculate the acos of n
Returns a result for the angle n (in radians).  
You can use the `to_degrees` and `to_radians` functions to convert between degrees and radians.  
  
##### Examples
```lavendeux
assert_eq( 0.0, acos(1) )
```

------------
#### `acosh(n:numeric) -> float`
Calculate the acosh of n
Returns a result for the angle n (in radians).  
You can use the `to_degrees` and `to_radians` functions to convert between degrees and radians.  
  
##### Examples
```lavendeux
assert_eq( 0.0, acosh(1) )
```

------------
#### `asin(n:numeric) -> float`
Calculate the asin of n
Returns a result for the angle n (in radians).  
You can use the `to_degrees` and `to_radians` functions to convert between degrees and radians.  
  
##### Examples
```lavendeux
assert_eq( 0.0, asin(0) )
```

------------
#### `asinh(n:numeric) -> float`
Calculate the asinh of n
Returns a result for the angle n (in radians).  
You can use the `to_degrees` and `to_radians` functions to convert between degrees and radians.  
  
##### Examples
```lavendeux
assert_eq( 0.0, asinh(0) )
```

------------
#### `atan(n:numeric) -> float`
Calculate the atan of n
Returns a result for the angle n (in radians).  
You can use the `to_degrees` and `to_radians` functions to convert between degrees and radians.  
  
##### Examples
```lavendeux
assert_eq( 0.0, atan(0) )
```

------------
#### `atanh(n:numeric) -> float`
Calculate the atanh of n
Returns a result for the angle n (in radians).  
You can use the `to_degrees` and `to_radians` functions to convert between degrees and radians.  
  
##### Examples
```lavendeux
assert_eq( 0.0, atanh(0) )
```

------------
#### `cos(n:numeric) -> float`
Calculate the cos of n
Returns a result for the angle n (in radians).  
You can use the `to_degrees` and `to_radians` functions to convert between degrees and radians.  
  
##### Examples
```lavendeux
assert_eq( 1.0, cos(0) )
```

------------
#### `cosh(n:numeric) -> float`
Calculate the cosh of n
Returns a result for the angle n (in radians).  
You can use the `to_degrees` and `to_radians` functions to convert between degrees and radians.  
  
##### Examples
```lavendeux
assert_eq( 1.0, cosh(0) )
```

------------
#### `sin(n:numeric) -> float`
Calculate the sin of n
Returns a result for the angle n (in radians).  
You can use the `to_degrees` and `to_radians` functions to convert between degrees and radians.  
  
##### Examples
```lavendeux
assert_eq( 0.0, sin(0) )
```

------------
#### `sinh(n:numeric) -> float`
Calculate the sinh of n
Returns a result for the angle n (in radians).  
You can use the `to_degrees` and `to_radians` functions to convert between degrees and radians.  
  
##### Examples
```lavendeux
assert_eq( 0.0, sinh(0) )
```

------------
#### `tan(n:numeric) -> float`
Calculate the tan of n
Returns a result for the angle n (in radians).  
You can use the `to_degrees` and `to_radians` functions to convert between degrees and radians.  
  
##### Examples
```lavendeux
assert_eq( 0.0, tan(0) )
```

------------
#### `tanh(n:numeric) -> float`
Calculate the tanh of n
Returns a result for the angle n (in radians).  
You can use the `to_degrees` and `to_radians` functions to convert between degrees and radians.  
  
##### Examples
```lavendeux
assert_eq( 0.0, tanh(0) )
```


