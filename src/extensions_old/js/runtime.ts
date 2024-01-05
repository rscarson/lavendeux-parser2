// @ts-ignore
import {LavendeuxValue, Types} from 'ext:lavendeux/value.js';
// @ts-ignore
import {LavendeuxExtension, LavendeuxFunction} from 'ext:lavendeux/extension.ts';

export class LavendeuxRuntime {
    extension: LavendeuxExtension;
    state: any;

    constructor() {
        this.extension = undefined;
        this.state = {};
    }

    types() {
        return Types;
    }

    extend(properties: any) {
        return new LavendeuxExtension(properties);
    }

    register(extension: LavendeuxExtension) {
        this.extension = extension;
        rustyscript.register_entrypoint(() => {
            let extension = globalThis.lavendeux.extension.export();
            console.log(JSON.stringify(extension));
            return extension;
        });
    }

    getState() {
        return this.state;
    }

    setState(state: any) {
        this.state = state;
    }

    callFunction(name: string, ...args: LavendeuxValue[]) {
        let definition: LavendeuxFunction = this.extension.functions[name];
        if (!definition) {
            throw new Error(`Function ${name} not found`);
        }
    
        // Unwrap state sent by rust
        let state = this.getState();
        for (const key of Object.keys(state)) {
            state[key] = LavendeuxValue.unwrap(state[key]);
        }
    
        let unwrappedArgs = LavendeuxValue.unwrap_all(definition.arguments, args);
        let value = LavendeuxValue.wrap(
            definition.callback(...unwrappedArgs, state),
            definition.returns
        );
    
        // Wrap up state to send to rust
        for (const key of Object.keys(state)) {
            state[key] = LavendeuxValue.wrap(state[key]);
        }
        this.setState(state);
        
        return value;
    }
}