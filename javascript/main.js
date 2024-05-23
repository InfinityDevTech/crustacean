"use strict";

// replace this with the name of your module
const MODULE_NAME = "crustacean";
let EXECUTION_PAUSED = false;
let RED_BUTTON = false;

let ERROR = false;
let js_memory = {};

function console_error() {
  const processedArgs = _.map(arguments, (arg) => {
      if (arg instanceof Error) {
          // On this version of Node, the `stack` property of errors contains
          // the message as well.
          return arg.stack;
      } else {
          return arg;
      }
  }).join(' ');
  console.log("ERROR:", processedArgs);
  Game.notify(processedArgs);
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

global.wipe_memory = function () {
  console.log("Wiping memory");

  global.RawMemory._parsed = {};
  global.Memory = {};
  global.Memory.rooms = {}
}

global.toggle_exec = function () {
    EXECUTION_PAUSED = !EXECUTION_PAUSED
    return `Successfully toggled execution pause to: ${EXECUTION_PAUSED}`
}

global.suicide_all = function() {
  for (let creep in Game.creeps) {
    let c = Game.creeps[creep];
    c.suicide()
  }
}

function run_loop() {
  if (ERROR) {
    // Stops memory leak present in WASM if rust were to error out.
    // Forces our global JS VM restart, (Basically a global reset)
    // A 10/10 way to stop memory leaks.
    Game.cpu.halt();
  } else {
    ERROR = true;

    delete global.Memory;
    global.Memory = js_memory;

    console.error = console_error
    try {

      if (!EXECUTION_PAUSED) {
        wasm_module.loop();
      }
      if (RED_BUTTON) {
        wasm_module.red_button();
        global.wipe_memory();
        RED_BUTTON = false;
        EXECUTION_PAUSED = false;
      }

      // If the WASM module were to break, execution wouldnt get to this point
      // Meaning ERROR would still be true, causing the reset.
      ERROR = false;
    } catch (e) {
      console.error("ERROR: Found an error! Resetting VM next tick...", e);
    }
  }
}

let wasm_module;

module.exports.loop = function () {
      // Fixes a memory corruption issue.
      if (!global.Memory) {
        global.RawMemory._parsed = {}; global.Memory = {}; global.Memory.rooms = {}
      }

      // attempt to load the wasm only if there's enough bucket to do a bunch of work this tick
      if (Game.cpu.bucket < 500) {
        console.log("Not enough in the CPU bucket, not going to compile - CPU: " + JSON.stringify(Game.cpu));
        return;
      }

      if (!wasm_module) {
        wasm_module = require(MODULE_NAME);
        wasm_module.initialize_instance();
      }

      delete require.cache[MODULE_NAME];

      module.exports.loop = run_loop;
      console.log("WASM module loaded successfully! Used CPU: " + Game.cpu.getUsed())
};
