/*!
 * 
 * This file is an extension for the Lavendeux parser
 * https://rscarson.github.io/lavendeux/
 * 
 */

lavendeuxExtensionName('Simple Extension');
lavendeuxExtensionAuthor('@rscarson');
lavendeuxExtensionVersion('1.0.0');

/**
 * Function adds the 2 operands and returns the result as an integer
 * Usage: add(<number>, <number>)
 * Can be called from the lavendeux parser
 * 
 * Accepts any numeric type, but the return type will always be cooerced to an integer
 */
lavendeuxFunction('add', (l, r) => l + r, {
    arguments: [lavendeuxType.Numeric, lavendeuxType.Numeric],
    returns: lavendeuxType.Int
});

/**
 * Formats an integer as a hex color code
 *  Usage: <number> @usd
 * Can be called from the lavendeux parser
 */
lavendeuxDecorator(
    'colour',
    (input) => `#${(input & 0x00FFFFFF).toString(16).padEnd(6, '0')}`,
    lavendeuxType.Int
);