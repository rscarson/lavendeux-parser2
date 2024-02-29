use lazy_static::lazy_static;
use serde_json::{json, Value};

use super::DocumentationFormatter;

pub struct DocumentationTemplate(Box<dyn DocumentationFormatter>);
impl DocumentationTemplate {
    const FUNCTION_TITLE: &'static str = "Functions";
    const OPERATOR_TITLE: &'static str = "Operators and Syntax";

    pub fn new(formatter: impl DocumentationFormatter + 'static) -> Self {
        Self(Box::new(formatter))
    }

    pub fn render_functions(&self, state: &crate::State, search: Option<&str>) -> String {
        self.0.format_functions(state, search)
    }

    pub fn render_operators(&self) -> String {
        self.0.format_operators()
    }

    pub fn render_values(&self) -> String {
        let mut output = String::new();

        let sections = VALUE_SECTION_DATA["contents"].as_array().unwrap().iter();
        for section in sections {
            output += &self.0.format_title(section["section"].as_str().unwrap());
            output += &section["text"]
                .as_str()
                .unwrap()
                .split("\n")
                .map(|s| self.0.format_text(s.trim()))
                .collect::<String>();
        }

        output
    }

    pub fn render(&self, state: &crate::State) -> String {
        let mut output = String::new();
        output += &self.render_values();
        output += "\n\n------------\n\n";

        output += &self.0.format_title(Self::OPERATOR_TITLE);
        output += &self.render_operators();
        output += "\n\n------------\n\n";

        output += &self.0.format_title(Self::FUNCTION_TITLE);
        output += &self.render_functions(state, None);

        output
    }
}

lazy_static! {
    pub static ref VALUE_SECTION_DATA: Value = json!({
        "contents": [
            {
                "section": "Lavendeux Documentation",
                "text": "
                    Lavendish a language designed to manipulate values with concise, single-line expressions.
                    It was created for use in Lavendeux (<https://rscarson.github.io/lavendeux/>).

                    Inputs are a series of expressions separated by a newline, or a `;`.
                    Expressions can optionally end with an @decorator to format the output as a string

                    Key features:
                    - Functions ([reference](<#function-assignment>))
                    - @Decorators ([reference](<#decorator>))
                    - Seamless type conversion ([reference](<#the-type-system>))
                "
            },

            {
                "section": "The Type System",
                "text": "
                    All expressions in Lavendeux must return a value of a specific type.
                    These types can be broadly categorized as `numeric`, `collection`, or `string`.

                    The type of a given expression is calculated on a hierarchy, based on the 'specificity' of the types involved.
                    The hierarchy is as follows (from lowest to highest priority):  
                    - Bool
                    - Int (in order, `u8, i8, u16, i16, u32, i32, u64, i64`)
                    - Float
                    - Fixed, then Currency
                    - String
                    - Array, then Object

                    Note that range is not included, since it can only be compared to itself, or to arrays.

                    So for example, if you add an int to a float, the result will be a float.
                    Some types can also be grouped together, like `int` (see aboove), or `numeric` (which includes all numeric types), 
                    and `collection` (which includes range, string, array and object). 'any' is the implicit type that includes everything.

                    The names of the types, for the most part, are self-explanatory, but here's a quick rundown:
                    `bool` - A single-bit truth value
                    `int` - A signed or unsigned integer of various sizes (`u8` to `i64`)
                    `float` - A 64bit floating-point number
                    `fixed` - A fixed-point decimal value
                    `currency` - A fixed-point decimal value with a currency symbol and set precision
                    `array` - An ordered collection of values, indexed by integers
                    `object` - An unordered collection of key-value pairs
                    `range` - An inclusive range of integers
                    `string` - A sequence of characters
                    `numeric` - The implicit type that includes all numeric types, and `bool`
                    `collection` - The implicit type that includes all collection types
                    `any` - The implicit type that includes everything

                    --------

                    ## bool
                    The `bool` type is a single-bit truth value, and can be either `true` or `false`.
                    In arithmetic operations, `true` is equivalent to `1`, and `false` is equivalent to `0`; operations are performed as if on a wrapping 1-bit integer.
                    
                    **Casting:**  
                    It can be cast from any type; and truth is determined by equivalence to 0, or by emptiness, depending on the type.

                    **Formatting:**  
                    When cast to a string, it will be formatted as `true` or `false`.

                    **Examples:**
                    ```lavendeux
                    1 as bool; // true
                    [] as bool; // false
                    ```

                    ## int
                    Covering the types from `u8` to `i64`, `int` is a signed or unsigned integer of various sizes.
                    Integers can be written in decimal, binary, octal, or hexadecimal, with an optional suffix to specify the type.
                    If no suffix is provided, the type defaults to i64.

                    **Casting:**
                    It can be cast from and to any type, and from any numeric type.

                    **Formatting:**  
                    When cast to a string, it will be formatted as a decimal number. For other formatting options, see [decorators](<#decorators-functions>).

                    **Examples:**
                    ```lavendeux
                    0xFFu8; // 255
                    0o77i16; // 63
                    077; // 63
                    0b1010_1010_1010_1010i32; // 43690
                    38_000; // 38,000
                    ```

                    ## float
                    A 64bit floating-point number, `float` can be written in decimal, or in scientific notation.

                    **Casting:**
                    It can be cast from and to any type, and from any numeric type.

                    **Formatting:**  
                    When cast to a string, it will be formatted as a decimal number. For other formatting options, see [decorators](<#decorators-functions>).

                    **Examples:**
                    ```lavendeux
                    1.0; // 1.0
                    .2; // 0.2
                    3e+7; // 30,000,000
                    ```

                    ## fixed
                    A fixed-point decimal value. Fixed literals are suffixed with `D`.

                    Note that fixed-point exponentiation is not supported, and will result in an implicit cast to float.

                    **Casting:**
                    It can be cast from and to any type, and from any numeric type.

                    **Formatting:**  
                    When cast to a string, it will appear similar to int or float, depending on the decimal precision of the value.

                    **Examples:**
                    ```lavendeux
                    1.22D; // 1.22
                    4D; // 4
                    ```

                    ## currency
                    A fixed-point decimal value with a currency symbol and set precision. Arithmetic operations will maintain the currency symbol only if both operands share the same one, and will use the larger precision.

                    **Casting:**
                    It can be cast from and to any type, and from any numeric type.

                    **Formatting:**  
                    When cast to a string, it will appear similar to fixed, but with the currency symbol.

                    **Examples:**
                    ```lavendeux
                    $1.00 + 1; // $1.00
                    $2.00 + £1.000; // 3.000
                    3￥; // 3
                    ```

                    **Supported currency symbols:**

                    ```
                    $ | ¢ | £ | ¤ | ¥ | ֏ | ؋ | ߾ | ߿ | ৲ | ৳ | ৻ | ૱ | ௹ | ฿ | ៛ | ₠ | ₡ |
                    ₢ | ₣ | ₤ | ₥ | ₦ | ₧ | ₨ | ₩ | ₪ | ₫ | € | ₭ | ₮ | ₯ | ₰ | ₱ | ₲ | ₳ |
                    ₴ | ₵ | ₶ | ₷ | ₸ | ₹ | ₺ | ₻ | ₼ | ₽ | ₾ | ₿ | ꠸ | ﷼ | ﹩ | ＄ | ￠ |
                    ￡ | ￥ | ￦
                    ```

                    ## array
                    An ordered collection of values, indexed by integers. Values can be a mix of types.

                    **Casting:**
                    It can be cast from any type; and will result in a single-value array - `A as array` is equivalent to `[A]`.
                    When cast to any non-collection type, it will extract the single value, if the array has a length of 1 - `['a'] as int` would result in `a`.
                    Casting a larger array to a non-collection type will result in an error.
                    Casting array to object will result in an object with integer keys, and the values of the array.
                    Casting range to array will result in an array of the range values (very large ranges may result in an error).

                    **Formatting:**  
                    When cast to a string, it will be formatted as a comma-separated list of values, enclosed in square brackets.

                    **Examples:**
                    ```lavendeux
                    [1, 2, 3]; // [1, 2, 3]
                    [1] as int; // 1
                    [1, 2, 3] as object; // {0: 1, 1: 2, 2: 3}
                    1..3 as array; // [1, 2, 3]
                    ```

                    ## object
                    An unordered collection of key-value pairs, where keys are any non-collection type and values can be a mix of types.
                    Unlike boolean comparison operators (see <#boolean>), key comparison is type-sensitive - `{1: 2} == {'1': 2}` would be false.

                    **Casting:**
                    It can be cast from any type; and will result in a single-value object - `A as object` is equivalent to `{0: A}`.
                    This is the same as casting to array, then to object.
                    When cast to any non-collection type, it will extract the single value, if the object has a length of 1 - `{0: 'a'} as int` would result in `a`.
                    Casting a larger object to a non-collection type will result in an error.
                    Casting object to array will result in an array of the object values.

                    **Formatting:**  
                    When cast to a string, it will be formatted as a comma-separated list of key-value pairs, enclosed in curly brackets.

                    **Examples:**
                    ```lavendeux
                    {1: 2}; // {1: 2}
                    {1: 2} as array; // [2]
                    {1: 2} as int; // 2
                    {1: 2} as string; // \"{1: 2}\"
                    ```

                    ## range
                    An inclusive range of integers, with a start and end value. Start and end can be single characters, or any numeric type.
                    
                    **Casting:**
                    Ranges cannot be cast from any type, and can only be cast to an array.

                    **Formatting:**  
                    When cast to a string, it will be formatted in the form `start..end`.

                    **Examples:**
                    ```lavendeux
                    1..3 as array; // [1, 2, 3]
                    'a'..'c' as array // ['a', 'b', 'c']
                    ```

                    ## string
                    Any value can be cast to a string, to get a string representation of that value. It is encoding as a UTF-8 string.
                    It is enclosed in single or double quotes, and supports the following supported escape sequences:
                    - `\\\'` Single-quote
                    - `\\\"` Double-quote
                    - `\\n` Newline
                    - `\\r` Carriage-return
                    - `\\t` Tab
                    - `\\\\` Literal backslash

                    **Casting:**
                    It can be cast from any type, and cast to array or object.
                    Casting to array will result in a character array, and casting to object will in a single-value object with the key `0i64`

                    **Formatting:**  
                    When cast to a string, it will be formatted as a string literal, without enclosing quotes.
                    Inside a, array or object, it will be formatted as a string literal, with enclosing quotes.

                    **Examples:**
                    ```lavendeux
                    \"test \\\"\"
                    1 as string // \"1\"
                    [1] as string // \"[1]\"
                    {0: 1} as string // \"{0: 1}\"
                    ```
                "
            },
        ]
    });
}
