/*!
 * 
 * This file is an extension for the Lavendeux parser
 * https://rscarson.github.io/lavendeux/
 * 
 */

lavendeuxExtensionName('Stateful Extension');
lavendeuxExtensionAuthor('@rscarson');
lavendeuxExtensionVersion('1.0.0');

/**
 * Function stores a variable in the parser state
 *  Usage: put(<name>, <value>)
 * Can be called from the lavendeux parser
 */
lavendeuxFunction('put', (name, value) => {
    let state = loadState();
    state[name] = value;

    saveState(state);
    return value;
}, {
    arguments: [lavendeuxType.String, lavendeuxType.Any],
    returns: lavendeuxType.Any
});

/**
 * Function gets a variable from the parser state
 *  Usage: get(<name>)
 * Can be called from the lavendeux parser
 */
lavendeuxFunction('get', (name) => {
    let state = loadState();
    return state[name];
}
, {
    arguments: [lavendeuxType.String],
    returns: lavendeuxType.Any
});