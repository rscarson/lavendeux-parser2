# Lavendeux - Language Documentation

This document will provide information on lavendish, a language focused on short, single-line expressions designed to manipulate values.
It was created for use in [Lavendeux](https://rscarson.github.io/lavendeux/).

## Chapter 1 - Values

All expressions in Lavendeux will return a value, in one of the following types, which include a few categories:

The first group of types are classified as numeric; they can all freely be converted between one another
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

The last value is string, which can be freely created from any other type