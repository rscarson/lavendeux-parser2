// @ts-ignore
import {LavendeuxValue, Types} from 'ext:lavendeux/value.js';

export class LavendeuxFunctionStub {
    arguments: Types[];
    returns: Types;
}

export class LavendeuxExtensionStub {
    name: string;
    author: string;
    version: string;
    functions: Map<string, LavendeuxFunctionStub>;
    decorators: Map<string, LavendeuxFunctionStub>;
}

export class LavendeuxExtension {
    name: string;
    author: string;
    version: string;
    functions: Map<string, LavendeuxFunction>;
    decorators: Map<string, LavendeuxFunction>;

    constructor(properties:any) {
        this.name = properties.name ?? 'Unnamed Extension';
        this.author = properties.author ?? 'Anonymous';
        this.version = properties.version ?? '0.0.0';

        this.functions = new Map();
        this.decorators = new Map();
    }

    addFunction(name, function_definition) {
        this.functions[name] = function_definition;
        return this.functions[name];
    }

    addDecorator(name, decorator_definition) {
        this.decorators[name] = decorator_definition;
        return this.decorators[name];
    }

    export(): LavendeuxExtensionStub {
        let functions = new Map();
        for (const name of Object.keys(this.functions)) {
            let definition = this.functions[name];
            functions[name] = definition.export();
        }
        
        let decorators = new Map();
        for (const name of Object.keys(this.decorators)) {
            let definition = this.decorators[name];
            decorators[name] = definition.export();
        }

        return {
            name: this.name,
            author: this.author,
            version: this.version,
            functions: functions,
            decorators: decorators,
        }
    }

    addIntegerFunction(name, callback) {
        return this.addFunction(name, new LavendeuxFunction(callback, Types.Integer));
    }

    addFloatFunction(name, callback) {
        return this.addFunction(name, new LavendeuxFunction(callback, Types.Float));
    }

    addStringFunction(name, callback) {
        return this.addFunction(name, new LavendeuxFunction(callback, Types.String));
    }

    addBooleanFunction(name, callback) {
        return this.addFunction(name, new LavendeuxFunction(callback, Types.Boolean));
    }

    addIntegerDecorator(name, callback) {
        return this.addDecorator(name, new LavendeuxDecorator(callback, Types.Integer));
    }

    addFloatDecorator(name, callback) {
        return this.addDecorator(name, new LavendeuxDecorator(callback, Types.Float));
    }

    addStringDecorator(name, callback) {
        return this.addDecorator(name, new LavendeuxDecorator(callback, Types.String));
    }

    addBooleanDecorator(name, callback) {
        return this.addDecorator(name, new LavendeuxDecorator(callback, Types.Boolean));
    }
}

/// Functions accept aguments and return a value
export class LavendeuxFunction {
    callback: Function;
    returns: Types;
    arguments: Types[];

    constructor(callback, returnType = Types.Any) {
        this.callback = callback;
        this.returns = returnType;
        this.arguments = [];
    }

    requireArgument(type = Types.Any) {
        this.arguments.push(type);
        return this;
    }

    requireArguments(...types) {
        this.arguments = [...this.arguments, ...types];
        return this;
    }

    requireInteger() {
        return this.requireArgument(Types.Integer);
    }

    requireFloat() {
        return this.requireArgument(Types.Float);
    }

    requireString() {
        return this.requireArgument(Types.String);
    }

    requireBoolean() {
        return this.requireArgument(Types.Boolean);
    }
    
    requireAny() {
        return this.requireArgument(Types.Any);
    }

    export(): LavendeuxFunctionStub {
        return {
            arguments: this.arguments,
            returns: this.returns,
        }
    }
}

/// Decorators are functions that return a string
/// and take in a single argument
export class LavendeuxDecorator extends LavendeuxFunction {
    constructor(callback, requiredType = Types.Any) {
        super(callback, Types.String);
        this.requireArgument(requiredType); 
    }
}