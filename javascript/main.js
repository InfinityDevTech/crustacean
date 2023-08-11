"use strict";
let wasm_module;

// replace this with the name of your module
const MODULE_NAME = "screeps";

function console_error(...args) {
    console.log(...args);
    Game.notify(args.join(' '));
}

module.exports.loop = function () {
     // Replace the Memory object (which gets populated into our global each tick) with an empty
        // object, so that accesses to it from within the driver that we can't prevent (such as
        // when a creep is spawned) won't trigger an attempt to parse RawMemory. Replace the object
        // with one unattached to memory magic - game functions will access the `Memory` object and
        // can throw data in here, and it'll go away at the end of tick.

        // Because it's in place, RawMemory's string won't be thrown to JSON.parse to deserialize -
        // and because that didn't happen, RawMemory._parsed isn't set and won't trigger a
        // post-tick serialize.
    //delete global.Memory;
    //global.Memory = {};
    try {
        if (wasm_module) {
            wasm_module.loop();
        } else {
            // attempt to load the wasm only if there's enough bucket to do a bunch of work this tick
            if (Game.cpu.bucket < 500) {
                console.log("Not enough in the CPU bucket, not going to compile - CPU: " + JSON.stringify(Game.cpu));
                return;
            }

            // delect the module from the cache, so we can reload it
            if (MODULE_NAME in require.cache) {
                delete require.cache[MODULE_NAME];
            }
            // load the wasm module
            wasm_module = require(MODULE_NAME);
            if (wasm_module != undefined) {
            // load the wasm instance!
            wasm_module.initialize_instance();
            // run the setup function, which configures logging
            wasm_module.setup();
            // go ahead and run the loop for its first tick
            wasm_module.loop();
            } else {
                console.log("Wasm module is undefined, is the name correct?")
            }
        }
    } catch (error) {
        console_error("Found error: ", error);
        if (error.stack) {
            console_error("Stack trace: ", error.stack);
        }
        console_error("Reloading wasm module");
        wasm_module = null;
    }
}
