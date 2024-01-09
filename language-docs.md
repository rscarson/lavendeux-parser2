# Lavendeux - Language Documentation

This document will provide information on lavendish, a language focused on short, single-line expressions designed to manipulate values.
It was created for use in [Lavendeux](https://rscarson.github.io/lavendeux/).

## Chapter 1 - Values

All expressions in Lavendeux will return a value, in one of the following types, which include a few categories:

The first group of types are classified as numeric; they can all freely be converted between one another
But expressions will always upgrade both values to the highest-order in this list (currency being the highest, bool, the lowest):
- Bool: a single-bit truth value
- Int: A 64bit integer (1, 2, 3, ...)
- Float: A 64bit floating-point number (1.0, .2, 3e+7, ...)
- Fixed: A fixed-point decimal value (1.22D, 4D, ...)
- Currency: A fixed-point decimal value with a currency symbol ($1, $2.00, 3￥, ...)

The 2nd group are compound types; attempting to convert non-compound types into one of these will result in a single-value array or object.
For example, `1 as array` would result in `[1]`, and `1 as object` would be the equivalent to `[1] as object`, which is `{0: 1}`.

Attemting to convert a compound value into a non-compound type will only work if the length of the compound value is 1

Compound types include:
- Array: An ordered set of values
- Object: A set of values indexed by a non-compound value - it is a syntax error to use a compound type as an object key
- Range: A special value which cannot be indexed into directly, and will always evaluate as an array in comparisions and operations. All ranges are inclusive

The last value is string, which can be freely created from any other type

### Formats and Examples

Here are the formats supported when using the above types:

**Integers**
- Base-10, such a `10`, with optional commas for thousands-seperators: `10,000`
- Other bases, such as binary (`0b101010101`), hex (`0xFFA`), or octal (`0777` or `0o6`)

**Floats**
- Regular notation, leading number is optional: `5.22` or `.3`
- Sci notation: `5e+1`, `5E-2`, `6.2e3`

**Decimal**
- Fixed-point literal: `1D`, `2.3323d`
- Currency value: `$1.00`, `3￥`

Supported Currency symbols:
```
$ | ¢ | £ | ¤ | ¥ | ֏ | ؋ | ߾ | ߿ | ৲ | ৳ | ৻ | ૱ | ௹ | ฿ | ៛ | ₠ | ₡ |
₢ | ₣ | ₤ | ₥ | ₦ | ₧ | ₨ | ₩ | ₪ | ₫ | € | ₭ | ₮ | ₯ | ₰ | ₱ | ₲ | ₳ |
₴ | ₵ | ₶ | ₷ | ₸ | ₹ | ₺ | ₻ | ₼ | ₽ | ₾ | ₿ | ꠸ | ﷼ | ﹩ | ＄ | ￠ |
￡ | ￥ | ￦
```

**Bool**
`true` or `false`, case-insensitive

**String**
Single or double quote enclosed; `'test'` or `"test"`
With the following supported escape sequences:
- `\'` Single-quote
- `\"` Dboule-quote
- `\n` Newline
- `\r` Carriage-return
- `\t` Tab
- `\\` Literal backslash

**Array**
Square bracket enclosed, comma separated; `[2, 3]`

**Object**
Curly-brace enclosed comma seperated pairs of `k:v`
Where key can be any type except array, object or range
`{0: 1, true: 'test', 1.2: 'no'}`

**Range**
Pair of integers or characters split by `..`
`0..10`; 0 to 10 inclusive
`'a'..'c'`; The array `['a', 'b', 'c']`

### Converting between types

You can manually convert between types using `<value> as <type>`, so long as that conversion is allowed:
- Numeric values can always convert to other numeric values
- Non-compound non-numeric values cannot convert into numeric values
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