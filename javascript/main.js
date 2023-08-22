"use strict";
let wasm_module;

// replace this with the name of your module
const MODULE_NAME = "crustacean";
let EXECUTION_PAUSED = false;
let RED_BUTTON = false;

function console_error(...args) {
  console.log(...args);
  Game.notify(args.join(" "));
}

global.big_red_button = function (input) {
  EXECUTION_PAUSED = true;
  console.log("The big red button has been pressed. Are you sure you want to do this?");
  console.log("This will suicide EVERY room, and EVERY creep. Are you still sure?");
  console.log("If you are sure. Then rerun this command like big_red_button(\"yes\") or big_red_button(\"no\")");

  if (input == undefined) {
    EXECUTION_PAUSED = true;
    return "Suicide? [y/n]"
  } else if (input.toLowerCase() == "n" || input.toLowerCase() == "no") {
    EXECUTION_PAUSED = false;
    return "The bot will live to see another day..."
  } else if (input.toLowerCase() == "y" || input.toLowerCase() == "yes") {
    RED_BUTTON = true;
    return "The bot will suicide on the next tick. Look at what you have done..."
  }
};

global.toggle_exec = function () {
    EXECUTION_PAUSED = !EXECUTION_PAUSED
    return `Successfully toggled execution pause to: ${EXECUTION_PAUSED}`
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
  delete global.Memory;
  global.Memory = {};
  try {
    if (wasm_module) {
      if (RED_BUTTON) {
        wasm_module.red_button();
        RED_BUTTON = false;
        EXECUTION_PAUSED = false;
      }
      if (!EXECUTION_PAUSED) {
        wasm_module.loop();
      }
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
      } else {
        console.log("Wasm module is undefined, is the name correct?");
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
};
