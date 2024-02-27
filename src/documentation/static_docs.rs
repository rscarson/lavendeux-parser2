use lazy_static::lazy_static;
use serde_json::{json, Value};

use super::DocumentationFormatter;
const INTRO_TEXT: &str = "
This document will provide information on lavendish, a language focused on short, single-line expressions designed to manipulate values.
It was created for use in Lavendeux (<https://rscarson.github.io/lavendeux/>).

Inputs are a series of expressions separated by a newline, or a `;`.
Lines can optionally end with an @decorator to format the output as a string (see `section 3.2`)

Comments are either `//`, which turns the rest of the line into a comment
Or a block comment bounded by /* and */
";

pub struct DocumentationTemplate(Box<dyn DocumentationFormatter>);
impl DocumentationTemplate {
    const DOCUMENT_TITLE: &str = "Lavendeux Documentation";
    const FUNCTION_TITLE: &str = "Functions";
    const OPERATOR_TITLE: &str = "Operators and Syntax";
    const VALUES_TITLE: &str = "Values and Literals";

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

        let intro = VALUE_SECTION_DATA["intro"].as_str().unwrap();
        output += &intro
            .split("\n")
            .map(|s| self.0.format_text(s.trim()))
            .collect::<String>();

        for section in VALUE_SECTION_DATA["contents"].as_array().unwrap() {
            let heading = section["section"].as_str().unwrap();
            output += &self.0.format_subtitle(heading);

            let text = section["text"].as_str().unwrap();
            output += &text
                .split("\n")
                .map(|s| self.0.format_text(s.trim()))
                .collect::<String>();
        }

        output
    }

    pub fn render(&self, state: &crate::State) -> String {
        let mut output = String::new();
        output += &self.0.format_title(Self::DOCUMENT_TITLE);
        output += INTRO_TEXT;

        output += &self.0.format_title(Self::VALUES_TITLE);
        output += &self.render_values();

        output += &self.0.format_title(Self::OPERATOR_TITLE);
        output += &self.render_operators();

        output += &self.0.format_title(Self::FUNCTION_TITLE);
        output += &self.render_functions(state, None);

        output
    }
}

lazy_static! {
    pub static ref VALUE_SECTION_DATA: Value = json!({
        "intro": "
            All expressions in Lavendeux will return a value of a specific type.
            These types can be broadly categorized as `numeric`, `collection`, or `string`.
        ",

        "contents": [
            {
                "section": "Numeric Types",
                "text": "
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
                "
            },

            {
                "section": "Collection Types",
                "text": "
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
                "
            },

            {
                "section": "Strings",
                "text": "
                    The last value is string, which any value can be cast to in order to get a string representation.

                    They are Single or double quote enclosed
                    With the following supported escape sequences:
                    - `\\'` Single-quote
                    - `\\\"` Double-quote
                    - `\\n` Newline
                    - `\\r` Carriage-return
                    - `\\t` Tab
                    - `\\\\` Literal backslash
                "
            },

            {
                "section": "Type Conversion",
                "text": "
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
                    - If either value is a bool, compare the values as bools"
            }
        ]
    });
}
