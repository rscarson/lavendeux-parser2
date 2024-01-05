// @ts-ignore
import {LavendeuxRuntime} from 'ext:lavendeux/runtime.ts';
// @ts-ignore
import 'ext:lavendeux/extension.ts';
// @ts-ignore
import { applyToGlobal, nonEnumerable } from 'ext:rustyscript/rustyscript.js'; 

applyToGlobal({
    lavendeux: nonEnumerable(
        new LavendeuxRuntime()
    ),

    setLavendeuxState: nonEnumerable(
        (s) => globalThis.lavendeux.setState(s)
    ),

    getLavendeuxState: nonEnumerable(
        () => globalThis.lavendeux.getState()
    ),

    callLavendeuxFunction: nonEnumerable(
        (name, ...args) => globalThis.lavendeux.callFunction(name, ...args)
    ),
});