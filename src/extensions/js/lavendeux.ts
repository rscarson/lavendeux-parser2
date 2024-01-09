export enum Type {
    Int = 'Int', Float = 'Float',
    String = 'String', Bool = 'Bool',
    Array = 'Array', Object = 'Object',
    Numeric = 'Numeric', Compound = 'Compound', Any = 'Any',
}

interface FunctionDefinition {
    name: string;
    arguments: Type[];
    returns: Type;

    callback: Function;
}

interface FunctionMetadata {
    description?: string;
    arguments?: Type[];
    returns?: Type;
}

interface ExtensionDetails {
    name: string;
    author: string;
    version: string;
    functions: Object;
}

class Lavendeux {
    variables: Object;
    functions: Object;
    extension_name: string;
    extension_author: string;
    extension_version: string;

    constructor() {
        this.variables = {};
        this.functions = {};
        this.extension_name = "Unnamed Extension";
        this.extension_author = "@anonymous";
        this.extension_version = "0.0.1";
    }

    addFunction(name: string, callback: Function, metadata: FunctionMetadata = {}) {
        let fn = {
            name: name,
            description: metadata.description ?? "",
            arguments: metadata.arguments ?? [],
            returns: metadata.returns ?? Type.Any,
            callback: callback,
        };
        this.functions[name] = fn;
    }

    addDecorator(name: string, callback: Function, expects: Type) {
        this.addFunction(`@${name}`, callback, {
            arguments: [expects],
            returns: Type.Any,
        });
    }

    setExtensionName(name: string) {
        this.extension_name = name;
    }

    setExtensionAuthor(author: string) {
        this.extension_author = author;
    }

    setExtensionVersion(version: string) {
        this.extension_version = version;
    }

    export(): ExtensionDetails {
        return {
            name: this.extension_name,
            author: this.extension_author,
            version: this.extension_version,
            functions: this.functions,
        };
    }
}

//@ts-ignore
import { applyToGlobal, nonEnumerable } from 'ext:rustyscript/rustyscript.js';
applyToGlobal({
    lavendeux: nonEnumerable(
      new Lavendeux(),
    ),

    lavendeuxType: nonEnumerable(
        Type
    ),

    lavendeuxFunction: nonEnumerable(
        function(name: string, callback: Function, metadata: FunctionMetadata = {}) {
            globalThis.lavendeux.addFunction(name, callback, metadata);
        }
    ),

    lavendeuxDecorator: nonEnumerable(
        function(name: string, callback: Function, expects: Type) {
            globalThis.lavendeux.addDecorator(name, callback, expects);
        }
    ),

    lavendeuxExtensionName: nonEnumerable(
        function(name: string) {
            globalThis.lavendeux.setExtensionName(name);
        }
    ),

    lavendeuxExtensionAuthor: nonEnumerable(
        function(author: string) {
            globalThis.lavendeux.setExtensionAuthor(author);
        }
    ),

    lavendeuxExtensionVersion: nonEnumerable(
        function(version: string) {
            globalThis.lavendeux.setExtensionVersion(version);
        }
    ),

    lavendeuxExport: nonEnumerable(
        function() {
            return globalThis.lavendeux.export();
        }
    ),

    saveState: nonEnumerable(
        function(variables: Object) {
            Object.assign(globalThis.lavendeux.variables, variables);
        }
    ),

    loadState: nonEnumerable(
        function() {
            return globalThis.lavendeux.variables;
        }
    ),

    callLavendeuxFunction: nonEnumerable(
        function(name: string, ...args: any[]) {
            let fn = globalThis.lavendeux.functions[name];
            if (fn) {
                return fn.callback(...args);
            } else {
                throw new Error(`Function ${name} does not exist.`);
            }
        }
    ),
});