"use strict";

// replace this with the name of your module
const MODULE_NAME = "crustacean";
const BUCKET_TO_COMPILE = 500;

// This provides the function `console.error` that wasm_bindgen sometimes expects to exist,
// especially with type checks in debug mode. An alternative is to have this be `function () {}`
// and let the exception handler log the thrown JS exceptions, but there is some additional
// information that wasm_bindgen only passes here.
//
// There is nothing special about this function and it may also be used by any JS/Rust code as a convenience.
function console_error() {
  const processedArgs = _.map(arguments, (arg) => {
    if (arg instanceof Error) {
      // On this version of Node, the `stack` property of errors contains
      // the message as well.
      return arg.stack;
    } else {
      return arg;
    }
  }).join(" ");
  console.log("[JS] ERROR:", processedArgs);
  Game.notify(processedArgs);
}

global.clear_scouting_data = function() {
  if (wasm_module) {
    wasm_module.wipe_scouting_data()
  }
}

global.hauler_rescan = function() {
  if (wasm_module) {
    wasm_module.hauler_rescan()
  }
}

global.pause_exec = function() {
  pause_exec = !pause_exec;
  return `[JS] pause_exec: ${pause_exec}`;
}

global.wipe_memory = function() {
  Memory = {};
  RawMemory.set(JSON.stringify(Memory));

  if (wasm_module) {
    wasm_module.wipe_memory();
  }

  return "[JS] Memory wiped";
}

// Set to true to have JS call Game.cpu.halt() on the next tick it processes.
// This is used so that console output from the end of the erroring tick
// will still be emitted, since calling halt destroys the environment instantly.
// The environment will be re-created with a fresh heap next tick automatically.
// We lose a tick of processing here, but it should be exceptional that code
// throws at all.
let halt_next_tick = false;
let pause_exec = false;

let wasm_module;
module.exports.loop = function () {
  if (pause_exec) {
    console.log("[JS] Skipping execution on tick: " + Game.time);
    return;
  }

  try {
    if (halt_next_tick) {
      // We encountered an error, skip execution in this tick and get
      // a new environment next tick.
      console.log("[JS] Resetting IVM...")

      Game.cpu.halt();
      return;
    }

    // need to freshly override the fake console object each tick
    console.error = console_error;

    // Decouple `Memory` from `RawMemory`, but give it `TempMemory` to persist to so that
    // `moveTo` can cache. This avoids issues where the game tries to insert data into `Memory`
    // that is not expected.
    delete global.Memory;
    global.TempMemory = global.TempMemory || Object.create(null);
    global.Memory = global.TempMemory;

    if (wasm_module) {
      wasm_module.game_loop();
    } else {
      console.log("[JS] Module not loaded... loading");

      // Only load the wasm module if there is enough bucket to complete it this tick.
      let bucket = Game.cpu.bucket;
      if (bucket < BUCKET_TO_COMPILE) {
        console.log(
          `[JS] ${bucket}/${BUCKET_TO_COMPILE} bucket to compile wasm`
        );
        return;
      }

      let cpu_before = Game.cpu.getUsed();

      console.log("[JS] Compiling...");
      // load the wasm module
      wasm_module = require(MODULE_NAME);
      // load the wasm instance!
      wasm_module.initialize_instance();

      let cpu_after = Game.cpu.getUsed();
      console.log(`[JS] ${cpu_after - cpu_before}cpu used to initialize wasm`);

      // I mean, hey, if we have double our execution time, fuck it.
      // Why not run it?
      if (Game.cpu.bucket > 1000) {
        // This used to be called on the JS side, but its
        // been moved to WASM to ensure it executes when the rust code executes.
        console.log(`[JS] We have ${Game.cpu.bucket} CPU in the bucket, so we are running it.`)
        //wasm_module.init();
        wasm_module.game_loop();
        console.log(`[JS] Successfully executed bot in the same tick that we loaded it. Huzzah!`)
      }

      console.log("[JS] Module loaded");
    }
  } catch (e) {
    if (
      e instanceof WebAssembly.CompileError ||
      e instanceof WebAssembly.LinkError
    ) {
      console.log(`[JS] exception during wasm compilation: ${e}`);
    } else if (e instanceof WebAssembly.RuntimeError) {
      console.log(`[JS] wasm aborted`);
    } else {
      console.log(`[JS] unexpected exception: ${e}`);
    }
    console.log(e.stack)
    console.log(`[JS] destroying environment...`);

    // reset everything
    halt_next_tick = true;
  }
};
