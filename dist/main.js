'use strict';

var commonjsGlobal = typeof globalThis !== 'undefined' ? globalThis : typeof window !== 'undefined' ? window : typeof global !== 'undefined' ? global : typeof self !== 'undefined' ? self : {};

(function (q) {
  function y() {}
  function C(b) {
    var c = b.charCodeAt(0) | 0;
    if (55296 <= c) if (56319 >= c) {
      if (b = b.charCodeAt(1) | 0, 56320 <= b && 57343 >= b) {
        if (c = (c << 10) + b - 56613888 | 0, 65535 < c) return v(240 | c >> 18, 128 | c >> 12 & 63, 128 | c >> 6 & 63, 128 | c & 63);
      } else c = 65533;
    } else 57343 >= c && (c = 65533);
    return 2047 >= c ? v(192 | c >> 6, 128 | c & 63) : v(224 | c >> 12, 128 | c >> 6 & 63, 128 | c & 63);
  }
  function z() {}
  function A(b, c) {
    var g = void 0 === b ? "" : ("" + b).replace(D, C),
      d = g.length | 0,
      a = 0,
      k = 0,
      f = c.length | 0,
      h = b.length | 0;
    f < d && (d = f);
    a: for (; a < d; a = a + 1 | 0) {
      b = g.charCodeAt(a) | 0;
      switch (b >> 4) {
        case 0:
        case 1:
        case 2:
        case 3:
        case 4:
        case 5:
        case 6:
        case 7:
          k = k + 1 | 0;
        case 8:
        case 9:
        case 10:
        case 11:
          break;
        case 12:
        case 13:
          if ((a + 1 | 0) < f) {
            k = k + 1 | 0;
            break;
          }
        case 14:
          if ((a + 2 | 0) < f) {
            k = k + 1 | 0;
            break;
          }
        case 15:
          if ((a + 3 | 0) < f) {
            k = k + 1 | 0;
            break;
          }
        default:
          break a;
      }
      c[a] = b;
    }
    return {
      written: a,
      read: h < k ? h : k
    };
  }
  var v = String.fromCharCode,
    x = {}.toString,
    E = x.call(q.SharedArrayBuffer),
    F = x(),
    t = q.Uint8Array,
    w = t || Array,
    u = t ? ArrayBuffer : w,
    G = u.isView || function (b) {
      return b && "length" in b;
    },
    H = x.call(u.prototype),
    B = z.prototype;
  u = q.TextEncoder;
  var D = /[\x80-\uD7ff\uDC00-\uFFFF]|[\uD800-\uDBFF][\uDC00-\uDFFF]?/g,
    e = new (t ? Uint16Array : w)(32);
  y.prototype.decode = function (b) {
    if (!G(b)) {
      var c = x.call(b);
      if (c !== H && c !== E && c !== F) throw TypeError("Failed to execute 'decode' on 'TextDecoder': The provided value is not of type '(ArrayBuffer or ArrayBufferView)'");
      b = t ? new w(b) : b || [];
    }
    for (var g = c = "", d = 0, a = b.length | 0, k = a - 32 | 0, f, h, l = 0, r = 0, n, m = 0, p = -1; d < a;) {
      for (f = d <= k ? 32 : a - d | 0; m < f; d = d + 1 | 0, m = m + 1 | 0) {
        h = b[d] & 255;
        switch (h >> 4) {
          case 15:
            n = b[d = d + 1 | 0] & 255;
            if (2 !== n >> 6 || 247 < h) {
              d = d - 1 | 0;
              break;
            }
            l = (h & 7) << 6 | n & 63;
            r = 5;
            h = 256;
          case 14:
            n = b[d = d + 1 | 0] & 255, l <<= 6, l |= (h & 15) << 6 | n & 63, r = 2 === n >> 6 ? r + 4 | 0 : 24, h = h + 256 & 768;
          case 13:
          case 12:
            n = b[d = d + 1 | 0] & 255, l <<= 6, l |= (h & 31) << 6 | n & 63, r = r + 7 | 0, d < a && 2 === n >> 6 && l >> r && 1114112 > l ? (h = l, l = l - 65536 | 0, 0 <= l && (p = (l >> 10) + 55296 | 0, h = (l & 1023) + 56320 | 0, 31 > m ? (e[m] = p, m = m + 1 | 0, p = -1) : (n = p, p = h, h = n))) : (h >>= 8, d = d - h - 1 | 0, h = 65533), l = r = 0, f = d <= k ? 32 : a - d | 0;
          default:
            e[m] = h;
            continue;
          case 11:
          case 10:
          case 9:
          case 8:
        }
        e[m] = 65533;
      }
      g += v(e[0], e[1], e[2], e[3], e[4], e[5], e[6], e[7], e[8], e[9], e[10], e[11], e[12], e[13], e[14], e[15], e[16], e[17], e[18], e[19], e[20], e[21], e[22], e[23], e[24], e[25], e[26], e[27], e[28], e[29], e[30], e[31]);
      32 > m && (g = g.slice(0, m - 32 | 0));
      if (d < a) {
        if (e[0] = p, m = ~p >>> 31, p = -1, g.length < c.length) continue;
      } else -1 !== p && (g += v(p));
      c += g;
      g = "";
    }
    return c;
  };
  B.encode = function (b) {
    b = void 0 === b ? "" : "" + b;
    var c = b.length | 0,
      g = new w((c << 1) + 8 | 0),
      d,
      a = 0,
      k = !t;
    for (d = 0; d < c; d = d + 1 | 0, a = a + 1 | 0) {
      var f = b.charCodeAt(d) | 0;
      if (127 >= f) g[a] = f;else {
        if (2047 >= f) g[a] = 192 | f >> 6;else {
          a: {
            if (55296 <= f) if (56319 >= f) {
              var h = b.charCodeAt(d = d + 1 | 0) | 0;
              if (56320 <= h && 57343 >= h) {
                f = (f << 10) + h - 56613888 | 0;
                if (65535 < f) {
                  g[a] = 240 | f >> 18;
                  g[a = a + 1 | 0] = 128 | f >> 12 & 63;
                  g[a = a + 1 | 0] = 128 | f >> 6 & 63;
                  g[a = a + 1 | 0] = 128 | f & 63;
                  continue;
                }
                break a;
              }
              f = 65533;
            } else 57343 >= f && (f = 65533);
            !k && d << 1 < a && d << 1 < (a - 7 | 0) && (k = !0, h = new w(3 * c), h.set(g), g = h);
          }
          g[a] = 224 | f >> 12;
          g[a = a + 1 | 0] = 128 | f >> 6 & 63;
        }
        g[a = a + 1 | 0] = 128 | f & 63;
      }
    }
    return t ? g.subarray(0, a) : g.slice(0, a);
  };
  B.encodeInto = A;
  if (!u) q.TextDecoder = y, q.TextEncoder = z;else if (!(q = u.prototype).encodeInto) {
    var I = new u();
    q.encodeInto = function (b, c) {
      var g = b.length | 0,
        d = c.length | 0;
      if (g < d >> 1) {
        var a = I.encode(b);
        if ((a.length | 0) < d) return c.set(a), {
          read: g,
          written: a.length | 0
        };
      }
      return A(b, c);
    };
  }
})("undefined" == typeof commonjsGlobal ? "undefined" == typeof self ? commonjsGlobal : self : commonjsGlobal); //AnonyCo

let wasm;
const heap = new Array(128).fill(undefined);
heap.push(undefined, null, true, false);
function getObject(idx) {
  return heap[idx];
}
let heap_next = heap.length;
function dropObject(idx) {
  if (idx < 132) return;
  heap[idx] = heap_next;
  heap_next = idx;
}
function takeObject(idx) {
  const ret = getObject(idx);
  dropObject(idx);
  return ret;
}
function addHeapObject(obj) {
  if (heap_next === heap.length) heap.push(heap.length + 1);
  const idx = heap_next;
  heap_next = heap[idx];
  heap[idx] = obj;
  return idx;
}
let WASM_VECTOR_LEN = 0;
let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
  if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
    cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
  }
  return cachedUint8ArrayMemory0;
}
const cachedTextEncoder = typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : {
  encode: () => {
    throw Error('TextEncoder not available');
  }
};
const encodeString = typeof cachedTextEncoder.encodeInto === 'function' ? function (arg, view) {
  return cachedTextEncoder.encodeInto(arg, view);
} : function (arg, view) {
  const buf = cachedTextEncoder.encode(arg);
  view.set(buf);
  return {
    read: arg.length,
    written: buf.length
  };
};
function passStringToWasm0(arg, malloc, realloc) {
  if (realloc === undefined) {
    const buf = cachedTextEncoder.encode(arg);
    const ptr = malloc(buf.length, 1) >>> 0;
    getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
    WASM_VECTOR_LEN = buf.length;
    return ptr;
  }
  let len = arg.length;
  let ptr = malloc(len, 1) >>> 0;
  const mem = getUint8ArrayMemory0();
  let offset = 0;
  for (; offset < len; offset++) {
    const code = arg.charCodeAt(offset);
    if (code > 0x7F) break;
    mem[ptr + offset] = code;
  }
  if (offset !== len) {
    if (offset !== 0) {
      arg = arg.slice(offset);
    }
    ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
    const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
    const ret = encodeString(arg, view);
    offset += ret.written;
    ptr = realloc(ptr, len, offset, 1) >>> 0;
  }
  WASM_VECTOR_LEN = offset;
  return ptr;
}
function isLikeNone(x) {
  return x === undefined || x === null;
}
let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
  if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer) {
    cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
  }
  return cachedDataViewMemory0;
}
const cachedTextDecoder = typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', {
  ignoreBOM: true,
  fatal: true
}) : {
  decode: () => {
    throw Error('TextDecoder not available');
  }
};
if (typeof TextDecoder !== 'undefined') {
  cachedTextDecoder.decode();
}
function getStringFromWasm0(ptr, len) {
  ptr = ptr >>> 0;
  return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}
function debugString(val) {
  // primitive types
  const type = typeof val;
  if (type == 'number' || type == 'boolean' || val == null) {
    return `${val}`;
  }
  if (type == 'string') {
    return `"${val}"`;
  }
  if (type == 'symbol') {
    const description = val.description;
    if (description == null) {
      return 'Symbol';
    } else {
      return `Symbol(${description})`;
    }
  }
  if (type == 'function') {
    const name = val.name;
    if (typeof name == 'string' && name.length > 0) {
      return `Function(${name})`;
    } else {
      return 'Function';
    }
  }
  // objects
  if (Array.isArray(val)) {
    const length = val.length;
    let debug = '[';
    if (length > 0) {
      debug += debugString(val[0]);
    }
    for (let i = 1; i < length; i++) {
      debug += ', ' + debugString(val[i]);
    }
    debug += ']';
    return debug;
  }
  // Test for built-in
  const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
  let className;
  if (builtInMatches.length > 1) {
    className = builtInMatches[1];
  } else {
    // Failed to match the standard '[object ClassName]'
    return toString.call(val);
  }
  if (className == 'Object') {
    // we're a user defined class or Object
    // JSON.stringify avoids problems with cycles, and is generally much
    // easier than looping through ownProperties of `val`.
    try {
      return 'Object(' + JSON.stringify(val) + ')';
    } catch (_) {
      return 'Object';
    }
  }
  // errors
  if (val instanceof Error) {
    return `${val.name}: ${val.message}\n${val.stack}`;
  }
  // TODO we could test for more things here, like `Set`s and `Map`s.
  return className;
}
const CLOSURE_DTORS = typeof FinalizationRegistry === 'undefined' ? {
  register: () => {},
  unregister: () => {}
} : new FinalizationRegistry(state => {
  wasm.__wbindgen_export_2.get(state.dtor)(state.a, state.b);
});
function makeMutClosure(arg0, arg1, dtor, f) {
  const state = {
    a: arg0,
    b: arg1,
    cnt: 1,
    dtor
  };
  const real = (...args) => {
    // First up with a closure we increment the internal reference
    // count. This ensures that the Rust closure environment won't
    // be deallocated while we're invoking it.
    state.cnt++;
    const a = state.a;
    state.a = 0;
    try {
      return f(a, state.b, ...args);
    } finally {
      if (--state.cnt === 0) {
        wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);
        CLOSURE_DTORS.unregister(state);
      } else {
        state.a = a;
      }
    }
  };
  real.original = state;
  CLOSURE_DTORS.register(real, state, state);
  return real;
}
function __wbg_adapter_38(arg0, arg1, arg2) {
  const ret = wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h3758e8a939264c27(arg0, arg1, addHeapObject(arg2));
  return takeObject(ret);
}
function __wbg_adapter_41(arg0, arg1, arg2, arg3) {
  const ret = wasm._dyn_core__ops__function__FnMut__A_B___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h91cf2a64a4dfb7be(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
  return ret;
}

/**
*/
function game_loop() {
  wasm.game_loop();
}

/**
*/
function toggle_creepsay() {
  wasm.toggle_creepsay();
}

/**
*/
function toggle_intent_subtraction() {
  wasm.toggle_intent_subtraction();
}

/**
*/
function wipe_memory() {
  wasm.wipe_memory();
}

/**
*/
function hauler_rescan() {
  wasm.hauler_rescan();
}

/**
*/
function wipe_scouting_data() {
  wasm.wipe_scouting_data();
}
function handleError(f, args) {
  try {
    return f.apply(this, args);
  } catch (e) {
    wasm.__wbindgen_exn_store(addHeapObject(e));
  }
}
const SearchGoalFinalization = typeof FinalizationRegistry === 'undefined' ? {
  register: () => {},
  unregister: () => {}
} : new FinalizationRegistry(ptr => wasm.__wbg_searchgoal_free(ptr >>> 0, 1));
/**
*/
class SearchGoal {
  static __wrap(ptr) {
    ptr = ptr >>> 0;
    const obj = Object.create(SearchGoal.prototype);
    obj.__wbg_ptr = ptr;
    SearchGoalFinalization.register(obj, obj.__wbg_ptr, obj);
    return obj;
  }
  __destroy_into_raw() {
    const ptr = this.__wbg_ptr;
    this.__wbg_ptr = 0;
    SearchGoalFinalization.unregister(this);
    return ptr;
  }
  free() {
    const ptr = this.__destroy_into_raw();
    wasm.__wbg_searchgoal_free(ptr, 0);
  }
  /**
  * @returns {any}
  */
  get pos() {
    const ret = wasm.searchgoal_pos(this.__wbg_ptr);
    return takeObject(ret);
  }
  /**
  * @returns {number}
  */
  get range() {
    const ret = wasm.searchgoal_range(this.__wbg_ptr);
    return ret >>> 0;
  }
}
function __wbg_get_imports() {
  const imports = {};
  imports.wbg = {};
  imports.wbg.__wbindgen_object_drop_ref = function (arg0) {
    takeObject(arg0);
  };
  imports.wbg.__wbindgen_object_clone_ref = function (arg0) {
    const ret = getObject(arg0);
    return addHeapObject(ret);
  };
  imports.wbg.__wbindgen_is_null = function (arg0) {
    const ret = getObject(arg0) === null;
    return ret;
  };
  imports.wbg.__wbindgen_is_undefined = function (arg0) {
    const ret = getObject(arg0) === undefined;
    return ret;
  };
  imports.wbg.__wbindgen_cb_drop = function (arg0) {
    const obj = takeObject(arg0).original;
    if (obj.cnt-- == 1) {
      obj.a = 0;
      return true;
    }
    const ret = false;
    return ret;
  };
  imports.wbg.__wbindgen_string_get = function (arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof obj === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
  };
  imports.wbg.__wbindgen_error_new = function (arg0, arg1) {
    const ret = new Error(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
  };
  imports.wbg.__wbindgen_number_get = function (arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof obj === 'number' ? obj : undefined;
    getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
  };
  imports.wbg.__wbindgen_is_object = function (arg0) {
    const val = getObject(arg0);
    const ret = typeof val === 'object' && val !== null;
    return ret;
  };
  imports.wbg.__wbg_setstackTraceLimit_44320267158fa775 = function (arg0) {
    Error.stackTraceLimit = arg0;
  };
  imports.wbg.__wbg_new_c815f255befb3565 = function () {
    const ret = new Error();
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_stack_24d57cd339b35ff3 = function (arg0, arg1) {
    const ret = getObject(arg1).stack;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
  };
  imports.wbg.__wbindgen_in = function (arg0, arg1) {
    const ret = getObject(arg0) in getObject(arg1);
    return ret;
  };
  imports.wbg.__wbg_log_b103404cc5920657 = function (arg0) {
    console.log(getObject(arg0));
  };
  imports.wbg.__wbg_structuretype_f649797e27122f54 = function (arg0) {
    var _spawn$extension$road;
    const ret = getObject(arg0).structureType;
    return (_spawn$extension$road = {
      "spawn": 0,
      "extension": 1,
      "road": 2,
      "constructedWall": 3,
      "rampart": 4,
      "keeperLair": 5,
      "portal": 6,
      "controller": 7,
      "link": 8,
      "storage": 9,
      "tower": 10,
      "observer": 11,
      "powerBank": 12,
      "powerSpawn": 13,
      "extractor": 14,
      "lab": 15,
      "terminal": 16,
      "container": 17,
      "nuker": 18,
      "factory": 19,
      "invaderCore": 20
    }[ret]) !== null && _spawn$extension$road !== void 0 ? _spawn$extension$road : 21;
  };
  imports.wbg.__wbindgen_string_new = function (arg0, arg1) {
    const ret = getStringFromWasm0(arg0, arg1);
    return addHeapObject(ret);
  };
  imports.wbg.__wbindgen_number_new = function (arg0) {
    const ret = arg0;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_searchgoal_new = function (arg0) {
    const ret = SearchGoal.__wrap(arg0);
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_level_f9a4a2f58d4c66e8 = function () {
    const ret = Game.gpl.level;
    return ret;
  };
  imports.wbg.__wbg_setbits_27588fc8de9d17bc = function (arg0, arg1) {
    getObject(arg0)._bits = getObject(arg1);
  };
  imports.wbg.__wbg_addVisual_d6e1c4ae1678ee98 = function (arg0, arg1) {
    console.addVisual(getObject(arg0), getObject(arg1));
  };
  imports.wbg.__wbg_unclaim_30009f1a7b1113a0 = function (arg0) {
    const ret = getObject(arg0).unclaim();
    return ret;
  };
  imports.wbg.__wbg_store_f768071483cf6395 = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_set_b3f982a6ba3dd327 = function (arg0, arg1, arg2, arg3) {
    getObject(arg0)[getStringFromWasm0(arg1, arg2)] = getObject(arg3);
  };
  imports.wbg.__wbg_getvalue_b80bdd3552834f00 = function (arg0, arg1) {
    const ret = getObject(arg0)[getObject(arg1)];
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_level_f3dfa13f1ce06fa3 = function (arg0) {
    const ret = getObject(arg0).level;
    return ret;
  };
  imports.wbg.__wbg_progress_9645c4737aab11fc = function (arg0, arg1) {
    const ret = getObject(arg1).progress;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
  };
  imports.wbg.__wbg_progresstotal_7eb9ec365ca0c8a4 = function (arg0, arg1) {
    const ret = getObject(arg1).progressTotal;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
  };
  imports.wbg.__wbg_reservation_520c47f2c9e2d137 = function (arg0) {
    const ret = getObject(arg0).reservation;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_safemode_deabb6e903457c85 = function (arg0, arg1) {
    const ret = getObject(arg1).safeMode;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
  };
  imports.wbg.__wbg_sign_4f1dfad9671ee17d = function (arg0) {
    const ret = getObject(arg0).sign;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_tickstodowngrade_90fbc7f0e3afc9c4 = function (arg0, arg1) {
    const ret = getObject(arg1).ticksToDowngrade;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
  };
  imports.wbg.__wbg_username_f16cc1ca39a939cb = function (arg0, arg1) {
    const ret = getObject(arg1).username;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
  };
  imports.wbg.__wbg_tickstoend_4573556f04085798 = function (arg0) {
    const ret = getObject(arg0).ticksToEnd;
    return ret;
  };
  imports.wbg.__wbg_username_b83cbfb7286afb19 = function (arg0, arg1) {
    const ret = getObject(arg1).username;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
  };
  imports.wbg.__wbg_text_d039559dd15add04 = function (arg0, arg1) {
    const ret = getObject(arg1).text;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
  };
  imports.wbg.__wbg_static_accessor_ROOM_POSITION_PROTOTYPE_359d8a1531b99b4c = function () {
    const ret = RoomPosition.prototype;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_static_accessor_COST_MATRIX_PROTOTYPE_fe39bc3209f68ee5 = function () {
    const ret = PathFinder.CostMatrix.prototype;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_limit_59cc679ca52ca53f = function () {
    const ret = Game.cpu.limit;
    return ret;
  };
  imports.wbg.__wbg_tickLimit_3aa5c80997ab477b = function () {
    const ret = Game.cpu.tickLimit;
    return ret;
  };
  imports.wbg.__wbg_bucket_031582e8f7867e7e = function () {
    const ret = Game.cpu.bucket;
    return ret;
  };
  imports.wbg.__wbg_getHeapStatistics_973209eb5f6ad318 = function () {
    const ret = Game.cpu.getHeapStatistics();
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_getUsed_61909a660b0ae0ca = function () {
    const ret = Game.cpu.getUsed();
    return ret;
  };
  imports.wbg.__wbg_generatePixel_42c2227c5bc41867 = function () {
    const ret = Game.cpu.generatePixel();
    return ret;
  };
  imports.wbg.__wbg_totalheapsize_f9817ff5c79b5aee = function (arg0) {
    const ret = getObject(arg0).total_heap_size;
    return ret;
  };
  imports.wbg.__wbg_heapsizelimit_397f274cace6c6de = function (arg0) {
    const ret = getObject(arg0).heap_size_limit;
    return ret;
  };
  imports.wbg.__wbg_externallyallocatedsize_fa8f3f8448ce1b98 = function (arg0) {
    const ret = getObject(arg0).externally_allocated_size;
    return ret;
  };
  imports.wbg.__wbg_get_ac6072ecac1ab4ec = function (arg0, arg1, arg2) {
    const ret = getObject(arg0).get(arg1, arg2);
    return ret;
  };
  imports.wbg.__wbg_getRawBuffer_5d59a6f5eef0c2fe = function (arg0) {
    const ret = getObject(arg0).getRawBuffer();
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_getRawBuffer_0c5c1a63cfae6bbe = function (arg0, arg1) {
    const ret = getObject(arg0).getRawBuffer(getObject(arg1));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_packed_9c1a64e69389301e = function (arg0) {
    const ret = getObject(arg0).__packedPos;
    return ret;
  };
  imports.wbg.__wbg_progressinternal_4fbd8979f938328e = function (arg0) {
    const ret = getObject(arg0).progress;
    return ret;
  };
  imports.wbg.__wbg_progresstotalinternal_a266a73ed1b67a66 = function (arg0) {
    const ret = getObject(arg0).progressTotal;
    return ret;
  };
  imports.wbg.__wbg_structuretypeinternal_11e53524f1984502 = function (arg0) {
    var _spawn$extension$road2;
    const ret = getObject(arg0).structureType;
    return (_spawn$extension$road2 = {
      "spawn": 0,
      "extension": 1,
      "road": 2,
      "constructedWall": 3,
      "rampart": 4,
      "keeperLair": 5,
      "portal": 6,
      "controller": 7,
      "link": 8,
      "storage": 9,
      "tower": 10,
      "observer": 11,
      "powerBank": 12,
      "powerSpawn": 13,
      "extractor": 14,
      "lab": 15,
      "terminal": 16,
      "container": 17,
      "nuker": 18,
      "factory": 19,
      "invaderCore": 20
    }[ret]) !== null && _spawn$extension$road2 !== void 0 ? _spawn$extension$road2 : 21;
  };
  imports.wbg.__wbg_remove_bee446ac73707224 = function (arg0) {
    const ret = getObject(arg0).remove();
    return ret;
  };
  imports.wbg.__wbg_idinternal_c193ef7f926aa07d = function (arg0) {
    const ret = getObject(arg0).id;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_amount_a2d9a609920908fb = function (arg0) {
    const ret = getObject(arg0).amount;
    return ret;
  };
  imports.wbg.__wbg_resourcetype_f4a4f882be6579a1 = function (arg0) {
    var _energy$power$H$O$U$L;
    const ret = getObject(arg0).resourceType;
    return (_energy$power$H$O$U$L = {
      "energy": 0,
      "power": 1,
      "H": 2,
      "O": 3,
      "U": 4,
      "L": 5,
      "K": 6,
      "Z": 7,
      "X": 8,
      "G": 9,
      "silicon": 10,
      "metal": 11,
      "biomass": 12,
      "mist": 13,
      "OH": 14,
      "ZK": 15,
      "UL": 16,
      "UH": 17,
      "UO": 18,
      "KH": 19,
      "KO": 20,
      "LH": 21,
      "LO": 22,
      "ZH": 23,
      "ZO": 24,
      "GH": 25,
      "GO": 26,
      "UH2O": 27,
      "UHO2": 28,
      "KH2O": 29,
      "KHO2": 30,
      "LH2O": 31,
      "LHO2": 32,
      "ZH2O": 33,
      "ZHO2": 34,
      "GH2O": 35,
      "GHO2": 36,
      "XUH2O": 37,
      "XUHO2": 38,
      "XKH2O": 39,
      "XKHO2": 40,
      "XLH2O": 41,
      "XLHO2": 42,
      "XZH2O": 43,
      "XZHO2": 44,
      "XGH2O": 45,
      "XGHO2": 46,
      "ops": 47,
      "utrium_bar": 48,
      "lemergium_bar": 49,
      "zynthium_bar": 50,
      "keanium_bar": 51,
      "ghodium_melt": 52,
      "oxidant": 53,
      "reductant": 54,
      "purifier": 55,
      "battery": 56,
      "composite": 57,
      "crystal": 58,
      "liquid": 59,
      "wire": 60,
      "switch": 61,
      "transistor": 62,
      "microchip": 63,
      "circuit": 64,
      "device": 65,
      "cell": 66,
      "phlegm": 67,
      "tissue": 68,
      "muscle": 69,
      "organoid": 70,
      "organism": 71,
      "alloy": 72,
      "tube": 73,
      "fixtures": 74,
      "frame": 75,
      "hydraulics": 76,
      "machine": 77,
      "condensate": 78,
      "concentrate": 79,
      "extract": 80,
      "spirit": 81,
      "emanation": 82,
      "essence": 83,
      "score": 84,
      "symbol_aleph": 85,
      "symbol_beth": 86,
      "symbol_gimmel": 87,
      "symbol_daleth": 88,
      "symbol_he": 89,
      "symbol_waw": 90,
      "symbol_zayin": 91,
      "symbol_heth": 92,
      "symbol_teth": 93,
      "symbol_yodh": 94,
      "symbol_kaph": 95,
      "symbol_lamedh": 96,
      "symbol_mem": 97,
      "symbol_nun": 98,
      "symbol_samekh": 99,
      "symbol_ayin": 100,
      "symbol_pe": 101,
      "symbol_tsade": 102,
      "symbol_qoph": 103,
      "symbol_res": 104,
      "symbol_sim": 105,
      "symbol_taw": 106,
      "T": 107
    }[ret]) !== null && _energy$power$H$O$U$L !== void 0 ? _energy$power$H$O$U$L : 108;
  };
  imports.wbg.__wbg_myinternal_34a921e8e1318280 = function (arg0) {
    const ret = getObject(arg0).my;
    return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
  };
  imports.wbg.__wbg_owner_5d079ae6a0547c64 = function (arg0) {
    const ret = getObject(arg0).owner;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_username_56d73a57441ac3fd = function (arg0, arg1) {
    const ret = getObject(arg1).username;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
  };
  imports.wbg.__wbg_ispublic_7d2db192050a91a4 = function (arg0) {
    const ret = getObject(arg0).isPublic;
    return ret;
  };
  imports.wbg.__wbg_x_97c79bb11147e323 = function (arg0) {
    const ret = getObject(arg0).x;
    return ret;
  };
  imports.wbg.__wbg_y_d85d939c960a9e07 = function (arg0) {
    const ret = getObject(arg0).y;
    return ret;
  };
  imports.wbg.__wbg_level_3472441b34ae87b9 = function () {
    const ret = Game.gcl.level;
    return ret;
  };
  imports.wbg.__wbg_progress_258cc589ef6c35a3 = function () {
    const ret = Game.gcl.progress;
    return ret;
  };
  imports.wbg.__wbg_progressTotal_082f1ffdc667d357 = function () {
    const ret = Game.gcl.progressTotal;
    return ret;
  };
  imports.wbg.__wbg_color_1ddee9bd1571462d = function (arg0) {
    const ret = getObject(arg0).color;
    return ret;
  };
  imports.wbg.__wbg_name_9b28d124534ede16 = function (arg0, arg1) {
    const ret = getObject(arg1).name;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
  };
  imports.wbg.__wbg_remove_4068854ab9cd7a15 = function (arg0) {
    getObject(arg0).remove();
  };
  imports.wbg.__wbg_newinternal_51317dc1a7d167e3 = function (arg0, arg1, arg2) {
    const ret = new RoomPosition(arg0, arg1, getObject(arg2));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_roomnameinternal_dc6e34ca63f4a4a3 = function (arg0) {
    const ret = getObject(arg0).roomName;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_setpacked_e36387710a3ff5bb = function (arg0, arg1) {
    getObject(arg0).__packedPos = arg1 >>> 0;
  };
  imports.wbg.__wbg_spawnCreep_cbeb034c62d318b9 = function (arg0, arg1, arg2, arg3, arg4) {
    const ret = getObject(arg0).spawnCreep(getObject(arg1), getStringFromWasm0(arg2, arg3), getObject(arg4));
    return ret;
  };
  imports.wbg.__wbg_recycleCreep_3766c804bd113d5c = function (arg0, arg1) {
    const ret = getObject(arg0).recycleCreep(getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_renewCreep_ecb0107030b4efcd = function (arg0, arg1) {
    const ret = getObject(arg0).renewCreep(getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_store_ed2723bd1b52812e = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_foreignsegment_1421a75b3fd0af09 = function () {
    const ret = RawMemory.foreignSegment;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_get_b8228a9d3cb3714b = function () {
    const ret = RawMemory.get();
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_set_cf96eae38e793219 = function (arg0) {
    RawMemory.set(getObject(arg0));
  };
  imports.wbg.__wbg_setActiveSegments_1f28e6e690205c54 = function (arg0) {
    RawMemory.setActiveSegments(getObject(arg0));
  };
  imports.wbg.__wbg_setActiveForeignSegment_c66945c79cbd039d = function (arg0, arg1) {
    RawMemory.setActiveForeignSegment(getObject(arg0), arg1 === 0xFFFFFF ? undefined : arg1);
  };
  imports.wbg.__wbg_jspos_7b7f24b33948398e = function (arg0) {
    const ret = getObject(arg0).pos;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_room_f3c33d5b3aec8ea2 = function (arg0) {
    const ret = getObject(arg0).room;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_x_b84a687ed2f85905 = function (arg0) {
    const ret = getObject(arg0).x;
    return ret;
  };
  imports.wbg.__wbg_y_e217c3387daf9935 = function (arg0) {
    const ret = getObject(arg0).y;
    return ret;
  };
  imports.wbg.__wbg_findInRange_925435432be083a5 = function (arg0, arg1, arg2, arg3) {
    const ret = getObject(arg0).findInRange(arg1, arg2, getObject(arg3));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_level_e65831d1fde5ee00 = function (arg0) {
    const ret = getObject(arg0).level;
    return ret;
  };
  imports.wbg.__wbg_spawning_6aa587c669d527fa = function (arg0) {
    const ret = getObject(arg0).spawning;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_name_80e83e6034d57f6c = function (arg0) {
    const ret = getObject(arg0).name;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_needtime_392a634064d36116 = function (arg0) {
    const ret = getObject(arg0).needTime;
    return ret;
  };
  imports.wbg.__wbg_remainingtime_bcd9215379159208 = function (arg0) {
    const ret = getObject(arg0).remainingTime;
    return ret;
  };
  imports.wbg.__wbg_username_93ab65086f70334a = function (arg0) {
    const ret = getObject(arg0).username;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_id_70ab24a63f7634bf = function (arg0) {
    const ret = getObject(arg0).id;
    return ret;
  };
  imports.wbg.__wbg_data_84f1bfdeb2db7349 = function (arg0) {
    const ret = getObject(arg0).data;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_store_1fa4f17d413a4bab = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_idinternal_98aec1bae648eaab = function (arg0) {
    const ret = getObject(arg0).id;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_store_86cbf630d108d869 = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_credits_f701ab1ed29ca05c = function () {
    const ret = Game.market.credits;
    return ret;
  };
  imports.wbg.__wbg_idinternal_d8fbbd34a20a3afd = function (arg0) {
    const ret = getObject(arg0).id;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_store_4ba6224686727d8c = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_getCapacity_2f288324200f0cfb = function (arg0, arg1, arg2) {
    const ret = getObject(arg1).getCapacity(arg2 === 109 ? undefined : ["energy", "power", "H", "O", "U", "L", "K", "Z", "X", "G", "silicon", "metal", "biomass", "mist", "OH", "ZK", "UL", "UH", "UO", "KH", "KO", "LH", "LO", "ZH", "ZO", "GH", "GO", "UH2O", "UHO2", "KH2O", "KHO2", "LH2O", "LHO2", "ZH2O", "ZHO2", "GH2O", "GHO2", "XUH2O", "XUHO2", "XKH2O", "XKHO2", "XLH2O", "XLHO2", "XZH2O", "XZHO2", "XGH2O", "XGHO2", "ops", "utrium_bar", "lemergium_bar", "zynthium_bar", "keanium_bar", "ghodium_melt", "oxidant", "reductant", "purifier", "battery", "composite", "crystal", "liquid", "wire", "switch", "transistor", "microchip", "circuit", "device", "cell", "phlegm", "tissue", "muscle", "organoid", "organism", "alloy", "tube", "fixtures", "frame", "hydraulics", "machine", "condensate", "concentrate", "extract", "spirit", "emanation", "essence", "score", "symbol_aleph", "symbol_beth", "symbol_gimmel", "symbol_daleth", "symbol_he", "symbol_waw", "symbol_zayin", "symbol_heth", "symbol_teth", "symbol_yodh", "symbol_kaph", "symbol_lamedh", "symbol_mem", "symbol_nun", "symbol_samekh", "symbol_ayin", "symbol_pe", "symbol_tsade", "symbol_qoph", "symbol_res", "symbol_sim", "symbol_taw", "T"][arg2]);
    getDataViewMemory0().setInt32(arg0 + 4 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
  };
  imports.wbg.__wbg_getFreeCapacity_bfb7550a9f100d5b = function (arg0, arg1, arg2) {
    const ret = getObject(arg1).getFreeCapacity(arg2 === 109 ? undefined : ["energy", "power", "H", "O", "U", "L", "K", "Z", "X", "G", "silicon", "metal", "biomass", "mist", "OH", "ZK", "UL", "UH", "UO", "KH", "KO", "LH", "LO", "ZH", "ZO", "GH", "GO", "UH2O", "UHO2", "KH2O", "KHO2", "LH2O", "LHO2", "ZH2O", "ZHO2", "GH2O", "GHO2", "XUH2O", "XUHO2", "XKH2O", "XKHO2", "XLH2O", "XLHO2", "XZH2O", "XZHO2", "XGH2O", "XGHO2", "ops", "utrium_bar", "lemergium_bar", "zynthium_bar", "keanium_bar", "ghodium_melt", "oxidant", "reductant", "purifier", "battery", "composite", "crystal", "liquid", "wire", "switch", "transistor", "microchip", "circuit", "device", "cell", "phlegm", "tissue", "muscle", "organoid", "organism", "alloy", "tube", "fixtures", "frame", "hydraulics", "machine", "condensate", "concentrate", "extract", "spirit", "emanation", "essence", "score", "symbol_aleph", "symbol_beth", "symbol_gimmel", "symbol_daleth", "symbol_he", "symbol_waw", "symbol_zayin", "symbol_heth", "symbol_teth", "symbol_yodh", "symbol_kaph", "symbol_lamedh", "symbol_mem", "symbol_nun", "symbol_samekh", "symbol_ayin", "symbol_pe", "symbol_tsade", "symbol_qoph", "symbol_res", "symbol_sim", "symbol_taw", "T"][arg2]);
    getDataViewMemory0().setInt32(arg0 + 4 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
  };
  imports.wbg.__wbg_getUsedCapacity_107f04bf6fe0f458 = function (arg0, arg1, arg2) {
    const ret = getObject(arg1).getUsedCapacity(arg2 === 109 ? undefined : ["energy", "power", "H", "O", "U", "L", "K", "Z", "X", "G", "silicon", "metal", "biomass", "mist", "OH", "ZK", "UL", "UH", "UO", "KH", "KO", "LH", "LO", "ZH", "ZO", "GH", "GO", "UH2O", "UHO2", "KH2O", "KHO2", "LH2O", "LHO2", "ZH2O", "ZHO2", "GH2O", "GHO2", "XUH2O", "XUHO2", "XKH2O", "XKHO2", "XLH2O", "XLHO2", "XZH2O", "XZHO2", "XGH2O", "XGHO2", "ops", "utrium_bar", "lemergium_bar", "zynthium_bar", "keanium_bar", "ghodium_melt", "oxidant", "reductant", "purifier", "battery", "composite", "crystal", "liquid", "wire", "switch", "transistor", "microchip", "circuit", "device", "cell", "phlegm", "tissue", "muscle", "organoid", "organism", "alloy", "tube", "fixtures", "frame", "hydraulics", "machine", "condensate", "concentrate", "extract", "spirit", "emanation", "essence", "score", "symbol_aleph", "symbol_beth", "symbol_gimmel", "symbol_daleth", "symbol_he", "symbol_waw", "symbol_zayin", "symbol_heth", "symbol_teth", "symbol_yodh", "symbol_kaph", "symbol_lamedh", "symbol_mem", "symbol_nun", "symbol_samekh", "symbol_ayin", "symbol_pe", "symbol_tsade", "symbol_qoph", "symbol_res", "symbol_sim", "symbol_taw", "T"][arg2]);
    getDataViewMemory0().setInt32(arg0 + 4 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
  };
  imports.wbg.__wbg_store_bd5e786d2cb206ee = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_path_38598c948557a564 = function (arg0) {
    const ret = getObject(arg0).path;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_attack_48ea80d103d6467e = function (arg0, arg1) {
    const ret = getObject(arg0).attack(getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_heal_7a144966127db1a9 = function (arg0, arg1) {
    const ret = getObject(arg0).heal(getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_repair_cc013b485c89290c = function (arg0, arg1) {
    const ret = getObject(arg0).repair(getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_search_968f0947e6f1f91f = function (arg0, arg1, arg2) {
    const ret = PathFinder.search(getObject(arg0), getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_roomcallback_70ede79933ec5c36 = function (arg0, arg1) {
    getObject(arg0).roomCallback = getObject(arg1);
  };
  imports.wbg.__wbg_cost_74703e549469c2ab = function (arg0) {
    const ret = getObject(arg0).cost;
    return ret;
  };
  imports.wbg.__wbg_incomplete_1d46dac35e06371c = function (arg0) {
    const ret = getObject(arg0).incomplete;
    return ret;
  };
  imports.wbg.__wbg_describeExits_27551810a951aa78 = function (arg0) {
    const ret = Game.map.describeExits(getObject(arg0));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_getRoomTerrain_de7e854c5bcae92b = function () {
    return handleError(function (arg0) {
      const ret = Game.map.getRoomTerrain(getObject(arg0));
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbg_getWorldSize_3cc40709f78f176d = function () {
    const ret = Game.map.getWorldSize();
    return ret;
  };
  imports.wbg.__wbg_status_0a7d5ac848f1b551 = function (arg0) {
    var _normal$closed$novice;
    const ret = getObject(arg0).status;
    return (_normal$closed$novice = {
      "normal": 0,
      "closed": 1,
      "novice": 2,
      "respawn": 3
    }[ret]) !== null && _normal$closed$novice !== void 0 ? _normal$closed$novice : 4;
  };
  imports.wbg.__wbg_timestamp_ad818bfd5e641be2 = function (arg0, arg1) {
    const ret = getObject(arg1).timestamp;
    getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
  };
  imports.wbg.__wbg_getRoomStatus_473df283e26a4a41 = function () {
    return handleError(function (arg0) {
      const ret = Game.map.getRoomStatus(getObject(arg0));
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbg_constructionsites_7bd4eac2e2d7e83b = function () {
    const ret = Game.constructionSites;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_creeps_db1f682ce5b6254b = function () {
    const ret = Game.creeps;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_flags_ee35096514ee4ffb = function () {
    const ret = Game.flags;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_resources_91944b4c15212ce3 = function () {
    const ret = Game.resources;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_rooms_8265844828084f14 = function () {
    const ret = Game.rooms;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_time_2223e935e4676691 = function () {
    const ret = Game.time;
    return ret;
  };
  imports.wbg.__wbg_getObjectById_9568b4fd872be44f = function (arg0) {
    const ret = Game.getObjectById(getObject(arg0));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_notify_3e0a91c8d44a1716 = function (arg0, arg1, arg2) {
    Game.notify(getObject(arg0), arg1 === 0 ? undefined : arg2 >>> 0);
  };
  imports.wbg.__wbg_idinternal_8e99a353022d67fc = function (arg0) {
    const ret = getObject(arg0).id;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_hitsinternal_0b32c5b85faacb5e = function (arg0, arg1) {
    const ret = getObject(arg1).hits;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
  };
  imports.wbg.__wbg_hitsmaxinternal_c377bcdb1da79fab = function (arg0, arg1) {
    const ret = getObject(arg1).hitsMax;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
  };
  imports.wbg.__wbg_transferEnergy_8e77728a26f524e4 = function (arg0, arg1, arg2, arg3) {
    const ret = getObject(arg0).transferEnergy(getObject(arg1), arg2 === 0 ? undefined : arg3 >>> 0);
    return ret;
  };
  imports.wbg.__wbg_store_c2df528a5aea99e5 = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_findRoute_b3393dc7b5ee2a67 = function (arg0, arg1, arg2) {
    const ret = Game.map.findRoute(getObject(arg0), getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_routecallback_27bccae1ce5a98d7 = function (arg0, arg1) {
    getObject(arg0).routeCallback = getObject(arg1);
  };
  imports.wbg.__wbg_mineraltype_d4913c9727184c38 = function (arg0) {
    var _energy$power$H$O$U$L2;
    const ret = getObject(arg0).mineralType;
    return (_energy$power$H$O$U$L2 = {
      "energy": 0,
      "power": 1,
      "H": 2,
      "O": 3,
      "U": 4,
      "L": 5,
      "K": 6,
      "Z": 7,
      "X": 8,
      "G": 9,
      "silicon": 10,
      "metal": 11,
      "biomass": 12,
      "mist": 13,
      "OH": 14,
      "ZK": 15,
      "UL": 16,
      "UH": 17,
      "UO": 18,
      "KH": 19,
      "KO": 20,
      "LH": 21,
      "LO": 22,
      "ZH": 23,
      "ZO": 24,
      "GH": 25,
      "GO": 26,
      "UH2O": 27,
      "UHO2": 28,
      "KH2O": 29,
      "KHO2": 30,
      "LH2O": 31,
      "LHO2": 32,
      "ZH2O": 33,
      "ZHO2": 34,
      "GH2O": 35,
      "GHO2": 36,
      "XUH2O": 37,
      "XUHO2": 38,
      "XKH2O": 39,
      "XKHO2": 40,
      "XLH2O": 41,
      "XLHO2": 42,
      "XZH2O": 43,
      "XZHO2": 44,
      "XGH2O": 45,
      "XGHO2": 46,
      "ops": 47,
      "utrium_bar": 48,
      "lemergium_bar": 49,
      "zynthium_bar": 50,
      "keanium_bar": 51,
      "ghodium_melt": 52,
      "oxidant": 53,
      "reductant": 54,
      "purifier": 55,
      "battery": 56,
      "composite": 57,
      "crystal": 58,
      "liquid": 59,
      "wire": 60,
      "switch": 61,
      "transistor": 62,
      "microchip": 63,
      "circuit": 64,
      "device": 65,
      "cell": 66,
      "phlegm": 67,
      "tissue": 68,
      "muscle": 69,
      "organoid": 70,
      "organism": 71,
      "alloy": 72,
      "tube": 73,
      "fixtures": 74,
      "frame": 75,
      "hydraulics": 76,
      "machine": 77,
      "condensate": 78,
      "concentrate": 79,
      "extract": 80,
      "spirit": 81,
      "emanation": 82,
      "essence": 83,
      "score": 84,
      "symbol_aleph": 85,
      "symbol_beth": 86,
      "symbol_gimmel": 87,
      "symbol_daleth": 88,
      "symbol_he": 89,
      "symbol_waw": 90,
      "symbol_zayin": 91,
      "symbol_heth": 92,
      "symbol_teth": 93,
      "symbol_yodh": 94,
      "symbol_kaph": 95,
      "symbol_lamedh": 96,
      "symbol_mem": 97,
      "symbol_nun": 98,
      "symbol_samekh": 99,
      "symbol_ayin": 100,
      "symbol_pe": 101,
      "symbol_tsade": 102,
      "symbol_qoph": 103,
      "symbol_res": 104,
      "symbol_sim": 105,
      "symbol_taw": 106,
      "T": 107
    }[ret]) !== null && _energy$power$H$O$U$L2 !== void 0 ? _energy$power$H$O$U$L2 : 108;
  };
  imports.wbg.__wbg_energy_131a8df5fd1e234d = function (arg0) {
    const ret = getObject(arg0).energy;
    return ret;
  };
  imports.wbg.__wbg_energycapacity_1d50a7bc556883e8 = function (arg0) {
    const ret = getObject(arg0).energyCapacity;
    return ret;
  };
  imports.wbg.__wbg_idinternal_74d0460df67d2753 = function (arg0) {
    const ret = getObject(arg0).id;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_destroy_2f9afa3e178f80d5 = function (arg0) {
    const ret = getObject(arg0).destroy();
    return ret;
  };
  imports.wbg.__wbg_isActive_47a60dffc5008a04 = function (arg0) {
    const ret = getObject(arg0).isActive();
    return ret;
  };
  imports.wbg.__wbg_bodyinternal_6e8475ce1e3da8c2 = function (arg0) {
    const ret = getObject(arg0).body;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_fatigueinternal_d2e89d47eef10cbe = function (arg0) {
    const ret = getObject(arg0).fatigue;
    return ret;
  };
  imports.wbg.__wbg_hitsinternal_3e876fb7e716a18f = function (arg0) {
    const ret = getObject(arg0).hits;
    return ret;
  };
  imports.wbg.__wbg_hitsmaxinternal_cfe28749941ea3be = function (arg0) {
    const ret = getObject(arg0).hitsMax;
    return ret;
  };
  imports.wbg.__wbg_myinternal_abbdc21a790e32fd = function (arg0) {
    const ret = getObject(arg0).my;
    return ret;
  };
  imports.wbg.__wbg_ownerinternal_9efe7d167fa26ddc = function (arg0) {
    const ret = getObject(arg0).owner;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_spawninginternal_f2987c04823791ce = function (arg0) {
    const ret = getObject(arg0).spawning;
    return ret;
  };
  imports.wbg.__wbg_storeinternal_5b78aa25fb786184 = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_tickstoliveinternal_54508b587da8d578 = function (arg0, arg1) {
    const ret = getObject(arg1).ticksToLive;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
  };
  imports.wbg.__wbg_attackController_049149b2f40feb87 = function (arg0, arg1) {
    const ret = Creep.prototype.attackController.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_build_950980471e9fbf19 = function (arg0, arg1) {
    const ret = Creep.prototype.build.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_claimController_adbd5eabc200ad3e = function (arg0, arg1) {
    const ret = Creep.prototype.claimController.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_drop_fb182e5e35d82f4f = function (arg0, arg1, arg2, arg3) {
    const ret = Creep.prototype.drop.call(getObject(arg0), ["energy", "power", "H", "O", "U", "L", "K", "Z", "X", "G", "silicon", "metal", "biomass", "mist", "OH", "ZK", "UL", "UH", "UO", "KH", "KO", "LH", "LO", "ZH", "ZO", "GH", "GO", "UH2O", "UHO2", "KH2O", "KHO2", "LH2O", "LHO2", "ZH2O", "ZHO2", "GH2O", "GHO2", "XUH2O", "XUHO2", "XKH2O", "XKHO2", "XLH2O", "XLHO2", "XZH2O", "XZHO2", "XGH2O", "XGHO2", "ops", "utrium_bar", "lemergium_bar", "zynthium_bar", "keanium_bar", "ghodium_melt", "oxidant", "reductant", "purifier", "battery", "composite", "crystal", "liquid", "wire", "switch", "transistor", "microchip", "circuit", "device", "cell", "phlegm", "tissue", "muscle", "organoid", "organism", "alloy", "tube", "fixtures", "frame", "hydraulics", "machine", "condensate", "concentrate", "extract", "spirit", "emanation", "essence", "score", "symbol_aleph", "symbol_beth", "symbol_gimmel", "symbol_daleth", "symbol_he", "symbol_waw", "symbol_zayin", "symbol_heth", "symbol_teth", "symbol_yodh", "symbol_kaph", "symbol_lamedh", "symbol_mem", "symbol_nun", "symbol_samekh", "symbol_ayin", "symbol_pe", "symbol_tsade", "symbol_qoph", "symbol_res", "symbol_sim", "symbol_taw", "T"][arg1], arg2 === 0 ? undefined : arg3 >>> 0);
    return ret;
  };
  imports.wbg.__wbg_move_7968aaa466189526 = function (arg0, arg1) {
    const ret = Creep.prototype.move.call(getObject(arg0), arg1);
    return ret;
  };
  imports.wbg.__wbg_notifyWhenAttacked_88a22d6257b73f8e = function (arg0, arg1) {
    const ret = Creep.prototype.notifyWhenAttacked.call(getObject(arg0), arg1 !== 0);
    return ret;
  };
  imports.wbg.__wbg_pickup_566b8a2c02365bcf = function (arg0, arg1) {
    const ret = Creep.prototype.pickup.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_rangedMassAttack_b72684c4f510c891 = function (arg0) {
    const ret = Creep.prototype.rangedMassAttack.call(getObject(arg0));
    return ret;
  };
  imports.wbg.__wbg_reserveController_9c9442df4abe1743 = function (arg0, arg1) {
    const ret = Creep.prototype.reserveController.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_say_a65bcd286fec2467 = function (arg0, arg1, arg2, arg3) {
    const ret = Creep.prototype.say.call(getObject(arg0), getStringFromWasm0(arg1, arg2), arg3 !== 0);
    return ret;
  };
  imports.wbg.__wbg_signController_ba3ae647e8016490 = function (arg0, arg1, arg2, arg3) {
    const ret = Creep.prototype.signController.call(getObject(arg0), getObject(arg1), getStringFromWasm0(arg2, arg3));
    return ret;
  };
  imports.wbg.__wbg_suicide_d8e98d900c8b5949 = function (arg0) {
    const ret = Creep.prototype.suicide.call(getObject(arg0));
    return ret;
  };
  imports.wbg.__wbg_upgradeController_336f2bc28a476f3e = function (arg0, arg1) {
    const ret = Creep.prototype.upgradeController.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_idinternal_c90f9e93c7ca9ba9 = function (arg0) {
    const ret = getObject(arg0).id;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_nameinternal_da0cb02560582620 = function (arg0, arg1) {
    const ret = getObject(arg1).name;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
  };
  imports.wbg.__wbg_store_aa36a2f659939261 = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_instanceof_Creep_0a25e8609ef30b20 = function (arg0) {
    let result;
    try {
      result = getObject(arg0) instanceof Creep;
    } catch (_) {
      result = false;
    }
    const ret = result;
    return ret;
  };
  imports.wbg.__wbg_attack_a181d10a1ea95ba3 = function (arg0, arg1) {
    const ret = Creep.prototype.attack.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_harvest_2770e15e12c70882 = function (arg0, arg1) {
    const ret = Creep.prototype.harvest.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_heal_7f248f985aa01c8f = function (arg0, arg1) {
    const ret = Creep.prototype.heal.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_rangedAttack_49845b9b85674625 = function (arg0, arg1) {
    const ret = Creep.prototype.rangedAttack.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_rangedHeal_ab926f9bac7ae1ea = function (arg0, arg1) {
    const ret = Creep.prototype.rangedHeal.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_repair_89e43a32b1083eaa = function (arg0, arg1) {
    const ret = Creep.prototype.repair.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_transfer_97ddf20e00891d3b = function (arg0, arg1, arg2, arg3, arg4) {
    const ret = Creep.prototype.transfer.call(getObject(arg0), getObject(arg1), ["energy", "power", "H", "O", "U", "L", "K", "Z", "X", "G", "silicon", "metal", "biomass", "mist", "OH", "ZK", "UL", "UH", "UO", "KH", "KO", "LH", "LO", "ZH", "ZO", "GH", "GO", "UH2O", "UHO2", "KH2O", "KHO2", "LH2O", "LHO2", "ZH2O", "ZHO2", "GH2O", "GHO2", "XUH2O", "XUHO2", "XKH2O", "XKHO2", "XLH2O", "XLHO2", "XZH2O", "XZHO2", "XGH2O", "XGHO2", "ops", "utrium_bar", "lemergium_bar", "zynthium_bar", "keanium_bar", "ghodium_melt", "oxidant", "reductant", "purifier", "battery", "composite", "crystal", "liquid", "wire", "switch", "transistor", "microchip", "circuit", "device", "cell", "phlegm", "tissue", "muscle", "organoid", "organism", "alloy", "tube", "fixtures", "frame", "hydraulics", "machine", "condensate", "concentrate", "extract", "spirit", "emanation", "essence", "score", "symbol_aleph", "symbol_beth", "symbol_gimmel", "symbol_daleth", "symbol_he", "symbol_waw", "symbol_zayin", "symbol_heth", "symbol_teth", "symbol_yodh", "symbol_kaph", "symbol_lamedh", "symbol_mem", "symbol_nun", "symbol_samekh", "symbol_ayin", "symbol_pe", "symbol_tsade", "symbol_qoph", "symbol_res", "symbol_sim", "symbol_taw", "T"][arg2], arg3 === 0 ? undefined : arg4 >>> 0);
    return ret;
  };
  imports.wbg.__wbg_withdraw_a56071292dfcdda7 = function (arg0, arg1, arg2, arg3, arg4) {
    const ret = Creep.prototype.withdraw.call(getObject(arg0), getObject(arg1), ["energy", "power", "H", "O", "U", "L", "K", "Z", "X", "G", "silicon", "metal", "biomass", "mist", "OH", "ZK", "UL", "UH", "UO", "KH", "KO", "LH", "LO", "ZH", "ZO", "GH", "GO", "UH2O", "UHO2", "KH2O", "KHO2", "LH2O", "LHO2", "ZH2O", "ZHO2", "GH2O", "GHO2", "XUH2O", "XUHO2", "XKH2O", "XKHO2", "XLH2O", "XLHO2", "XZH2O", "XZHO2", "XGH2O", "XGHO2", "ops", "utrium_bar", "lemergium_bar", "zynthium_bar", "keanium_bar", "ghodium_melt", "oxidant", "reductant", "purifier", "battery", "composite", "crystal", "liquid", "wire", "switch", "transistor", "microchip", "circuit", "device", "cell", "phlegm", "tissue", "muscle", "organoid", "organism", "alloy", "tube", "fixtures", "frame", "hydraulics", "machine", "condensate", "concentrate", "extract", "spirit", "emanation", "essence", "score", "symbol_aleph", "symbol_beth", "symbol_gimmel", "symbol_daleth", "symbol_he", "symbol_waw", "symbol_zayin", "symbol_heth", "symbol_teth", "symbol_yodh", "symbol_kaph", "symbol_lamedh", "symbol_mem", "symbol_nun", "symbol_samekh", "symbol_ayin", "symbol_pe", "symbol_tsade", "symbol_qoph", "symbol_res", "symbol_sim", "symbol_taw", "T"][arg2], arg3 === 0 ? undefined : arg4 >>> 0);
    return ret;
  };
  imports.wbg.__wbg_part_29daa248af65a34e = function (arg0) {
    var _move$work$carry$atta;
    const ret = getObject(arg0).type;
    return (_move$work$carry$atta = {
      "move": 0,
      "work": 1,
      "carry": 2,
      "attack": 3,
      "ranged_attack": 4,
      "tough": 5,
      "heal": 6,
      "claim": 7
    }[ret]) !== null && _move$work$carry$atta !== void 0 ? _move$work$carry$atta : 8;
  };
  imports.wbg.__wbg_hits_13d1c3db4bd0e66c = function (arg0) {
    const ret = getObject(arg0).hits;
    return ret;
  };
  imports.wbg.__wbg_creep_8e8725d030a8411b = function (arg0) {
    const ret = getObject(arg0).creep;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_energy_fc7421445ec7a729 = function (arg0) {
    const ret = getObject(arg0).energy;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_resource_be4f6a63d0439801 = function (arg0) {
    const ret = getObject(arg0).resource;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_source_ed8c01e434833079 = function (arg0) {
    const ret = getObject(arg0).source;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_mineral_a79f66df2b57802b = function (arg0) {
    const ret = getObject(arg0).mineral;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_structure_899125548b507fa8 = function (arg0) {
    const ret = getObject(arg0).structure;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_flag_b9aea8b5163e4015 = function (arg0) {
    const ret = getObject(arg0).flag;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_constructionsite_1e61a8d5046cca2e = function (arg0) {
    const ret = getObject(arg0).constructionSite;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_nuke_9d5fa15ea10f97a6 = function (arg0) {
    const ret = getObject(arg0).nuke;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_terrain_cb5e44c70a3b5fa1 = function (arg0, arg1) {
    const ret = getObject(arg1).terrain;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
  };
  imports.wbg.__wbg_tombstone_e3ce0d7c1cf8e358 = function (arg0) {
    const ret = getObject(arg0).tombstone;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_powercreep_a92cbde661ed9e63 = function (arg0) {
    const ret = getObject(arg0).powerCreep;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_deposit_e80f5b9742425013 = function (arg0) {
    const ret = getObject(arg0).deposit;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_ruin_b4966ccc6deae444 = function (arg0) {
    const ret = getObject(arg0).ruin;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_name_16f4ff7a3db26a91 = function () {
    const ret = Game.shard.name;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_nameinternal_81ea8bea19676a63 = function (arg0) {
    const ret = getObject(arg0).name;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_createConstructionSite_cf7b1e34160e7a05 = function (arg0, arg1, arg2, arg3, arg4) {
    const ret = Room.prototype.createConstructionSite.call(getObject(arg0), arg1, arg2, ["spawn", "extension", "road", "constructedWall", "rampart", "keeperLair", "portal", "controller", "link", "storage", "tower", "observer", "powerBank", "powerSpawn", "extractor", "lab", "terminal", "container", "nuker", "factory", "invaderCore"][arg3], getObject(arg4));
    return ret;
  };
  imports.wbg.__wbg_getEventLog_e0cc6b6746ea21ee = function (arg0, arg1) {
    const ret = Room.prototype.getEventLog.call(getObject(arg0), arg1 !== 0);
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_store_a8617cab184360eb = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_controller_44590b47fc7d9ea7 = function (arg0) {
    const ret = getObject(arg0).controller;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_energyavailable_5135d2343c3a7eaf = function (arg0) {
    const ret = getObject(arg0).energyAvailable;
    return ret;
  };
  imports.wbg.__wbg_energycapacityavailable_687ab550b8e142b5 = function (arg0) {
    const ret = getObject(arg0).energyCapacityAvailable;
    return ret;
  };
  imports.wbg.__wbg_find_40f868934bfe7228 = function (arg0, arg1, arg2) {
    const ret = getObject(arg0).find(arg1, getObject(arg2));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_getTerrain_4036878c5c475617 = function (arg0) {
    const ret = Room.prototype.getTerrain.call(getObject(arg0));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_lookForAt_a4c37763a6475977 = function (arg0, arg1, arg2, arg3) {
    const ret = Room.prototype.lookForAt.call(getObject(arg0), ["creep", "energy", "resource", "source", "mineral", "structure", "flag", "constructionSite", "nuke", "terrain", "tombstone", "powerCreep", "deposit", "ruin", "scoreContainer", "scoreCollector", "symbolContainer", "symbolDecoder", "reactor"][arg1], arg2, arg3);
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_lookForAtArea_2cfed6c24d2e64a6 = function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    const ret = Room.prototype.lookForAtArea.call(getObject(arg0), ["creep", "energy", "resource", "source", "mineral", "structure", "flag", "constructionSite", "nuke", "terrain", "tombstone", "powerCreep", "deposit", "ruin", "scoreContainer", "scoreCollector", "symbolContainer", "symbolDecoder", "reactor"][arg1], arg2, arg3, arg4, arg5, arg6 !== 0);
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbindgen_jsval_loose_eq = function (arg0, arg1) {
    const ret = getObject(arg0) == getObject(arg1);
    return ret;
  };
  imports.wbg.__wbindgen_boolean_get = function (arg0) {
    const v = getObject(arg0);
    const ret = typeof v === 'boolean' ? v ? 1 : 0 : 2;
    return ret;
  };
  imports.wbg.__wbindgen_as_number = function (arg0) {
    const ret = +getObject(arg0);
    return ret;
  };
  imports.wbg.__wbg_getwithrefkey_edc2c8960f0f1191 = function (arg0, arg1) {
    const ret = getObject(arg0)[getObject(arg1)];
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_set_f975102236d3c502 = function (arg0, arg1, arg2) {
    getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
  };
  imports.wbg.__wbg_get_3baa728f9d58d3f6 = function (arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_length_ae22078168b726f5 = function (arg0) {
    const ret = getObject(arg0).length;
    return ret;
  };
  imports.wbg.__wbg_new_a220cf903aa02ca2 = function () {
    const ret = new Array();
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_new_525245e2b9901204 = function () {
    const ret = new Object();
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_set_673dda6c73d19609 = function (arg0, arg1, arg2) {
    getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
  };
  imports.wbg.__wbg_push_37c89022f34c01ca = function (arg0, arg1) {
    const ret = getObject(arg0).push(getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_instanceof_ArrayBuffer_61dfc3198373c902 = function (arg0) {
    let result;
    try {
      result = getObject(arg0) instanceof ArrayBuffer;
    } catch (_) {
      result = false;
    }
    const ret = result;
    return ret;
  };
  imports.wbg.__wbg_isSafeInteger_7f1ed56200d90674 = function (arg0) {
    const ret = Number.isSafeInteger(getObject(arg0));
    return ret;
  };
  imports.wbg.__wbg_create_74febf7e6d272aa6 = function (arg0) {
    const ret = Object.create(getObject(arg0));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_keys_7840ae453e408eab = function (arg0) {
    const ret = Object.keys(getObject(arg0));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_values_3b5ac1979695a16f = function (arg0) {
    const ret = Object.values(getObject(arg0));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_buffer_b7b08af79b0b0974 = function (arg0) {
    const ret = getObject(arg0).buffer;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_newwithbyteoffsetandlength_8a2cb9ca96b27ec9 = function (arg0, arg1, arg2) {
    const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_new_ea1883e1e5e86686 = function (arg0) {
    const ret = new Uint8Array(getObject(arg0));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_set_d1e79e2388520f18 = function (arg0, arg1, arg2) {
    getObject(arg0).set(getObject(arg1), arg2 >>> 0);
  };
  imports.wbg.__wbg_length_8339fcf5d8ecd12e = function (arg0) {
    const ret = getObject(arg0).length;
    return ret;
  };
  imports.wbg.__wbg_instanceof_Uint8Array_247a91427532499e = function (arg0) {
    let result;
    try {
      result = getObject(arg0) instanceof Uint8Array;
    } catch (_) {
      result = false;
    }
    const ret = result;
    return ret;
  };
  imports.wbg.__wbindgen_debug_string = function (arg0, arg1) {
    const ret = debugString(getObject(arg1));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
  };
  imports.wbg.__wbindgen_throw = function (arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
  };
  imports.wbg.__wbindgen_memory = function () {
    const ret = wasm.memory;
    return addHeapObject(ret);
  };
  imports.wbg.__wbindgen_closure_wrapper782 = function (arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 97, __wbg_adapter_38);
    return addHeapObject(ret);
  };
  imports.wbg.__wbindgen_closure_wrapper784 = function (arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 97, __wbg_adapter_41);
    return addHeapObject(ret);
  };
  return imports;
}
function __wbg_finalize_init(instance, module) {
  wasm = instance.exports;
  cachedDataViewMemory0 = null;
  cachedUint8ArrayMemory0 = null;
  return wasm;
}
function initSync(module) {
  if (wasm !== undefined) return wasm;
  if (typeof module !== 'undefined' && Object.getPrototypeOf(module) === Object.prototype) ({
    module
  } = module);else console.warn('using deprecated parameters for `initSync()`; pass a single object instead');
  const imports = __wbg_get_imports();
  if (!(module instanceof WebAssembly.Module)) {
    module = new WebAssembly.Module(module);
  }
  const instance = new WebAssembly.Instance(module, imports);
  return __wbg_finalize_init(instance);
}

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
  const processedArgs = _.map(arguments, arg => {
    if (arg instanceof Error) {
      // On this version of Node, the `stack` property of errors contains
      // the message as well.
      try {
        return arg.stack;
      } catch (e) {
        console.log("[JS] Error while processing error:", e);
        return arg;
      }
    } else {
      return arg;
    }
  }).join(" ");
  console.log("[JS] ERROR:", processedArgs);
  Game.notify(processedArgs);
}
global.help = function () {
  return `
  Available commands:
  - help(): display this message
  - toggle_intents_profiling(): toggle intents profiling on and off
  - toggle_creepsay(): toggle creepsay on and off
  - clear_scouting_data(): clear the scouting data
  - hauler_rescan(): rescan the hauler network for each room
  - pause_exec(): pause execution of the bot
  - wipe_memory(): wipe all memory
  `;
};
global.toggle_creepsay = function () {
  if (wasm_instance) {
    toggle_creepsay();
    return `[JS] Toggled creepsay.`;
  } else {
    return `[JS] Module not loaded.`;
  }
};
global.toggle_intents_profiling = function () {
  if (wasm_instance) {
    toggle_intent_subtraction();
    return `[JS] Toggled intent subtraction.`;
  } else {
    return `[JS] Module not loaded.`;
  }
};
global.clear_scouting_data = function () {
  if (wasm_instance) {
    wipe_scouting_data();
    return `[JS] Cleared scouting data.`;
  } else {
    return `[JS] Module not loaded.`;
  }
};
global.hauler_rescan = function () {
  if (wasm_instance) {
    hauler_rescan();
    return `[JS] Rescanned hauler network.`;
  } else {
    return `[JS] Module not loaded.`;
  }
};
global.pause_exec = function () {
  pause_exec = !pause_exec;
  return `[JS] Setting execution pause to: ${pause_exec}`;
};
global.wipe_memory = function () {
  Memory = {};
  RawMemory.set(JSON.stringify(Memory));
  if (wasm_instance) {
    wipe_memory();
  }
  return "[JS] Memory wiped";
};

// Set to true to have JS call Game.cpu.halt() on the next tick it processes.
// This is used so that console output from the end of the erroring tick
// will still be emitted, since calling halt destroys the environment instantly.
// The environment will be re-created with a fresh heap next tick automatically.
// We lose a tick of processing here, but it should be exceptional that code
// throws at all.
let halt_next_tick = false;
let pause_exec = false;

// cache for each step of the wasm module's initialization
let wasm_bytes, wasm_module, wasm_instance;
module.exports.loop = function () {
  // need to freshly override the fake console object each tick
  console.error = console_error;
  if (pause_exec) {
    console.log("[JS] Skipping execution on tick: " + Game.time);
    return;
  }
  try {
    if (halt_next_tick) {
      // We encountered an error, skip execution in this tick and get
      // a new environment next tick.
      console.log("[JS] Resetting IVM...");
      Game.cpu.halt();
      return;
    }

    // temporarily need to polyfill this too because there's a bug causing the warn
    // in initSync to fire in bindgen 0.2.93
    console.warn = console.log;

    // Decouple `Memory` from `RawMemory`, but give it `TempMemory` to persist to so that
    // `moveTo` can cache. This avoids issues where the game tries to insert data into `Memory`
    // that is not expected.
    delete global.Memory;
    global.TempMemory = global.TempMemory || Object.create(null);
    global.Memory = global.TempMemory;
    if (wasm_instance) {
      game_loop();
    } else {
      console.log("[JS] Module not loaded... loading");

      // Only load the wasm module if there is enough bucket to complete it this tick.
      let bucket = Game.cpu.bucket;
      if (bucket < BUCKET_TO_COMPILE) {
        console.log(`[JS] ${bucket}/${BUCKET_TO_COMPILE} bucket to compile wasm`);
        return;
      }
      let cpu_before = Game.cpu.getUsed();
      console.log("[JS] Compiling...");
      // run each step of the load process, saving each result so that this can happen over multiple ticks
      if (!wasm_bytes) wasm_bytes = require(MODULE_NAME);
      if (!wasm_module) wasm_module = new WebAssembly.Module(wasm_bytes);
      if (!wasm_instance) wasm_instance = initSync(wasm_module);

      // remove the bytes from the heap and require cache, we don't need 'em anymore
      wasm_bytes = null;
      delete require.cache[MODULE_NAME];
      let cpu_after = Game.cpu.getUsed();
      console.log(`[JS] ${cpu_after - cpu_before}cpu used to initialize crustacean`);

      // I mean, hey, if we have double our execution time, fuck it.
      // Why not run it?
      if (Game.cpu.bucket > 1000) {
        // This used to be called on the JS side, but its
        // been moved to WASM to ensure it executes when the rust code executes.
        console.log(`[JS] We have ${Game.cpu.bucket} CPU in the bucket, so we are running it.`);
        //wasm_module.init();
        game_loop();
        console.log(`[JS] Successfully executed bot in the same tick that we loaded it. Huzzah!`);
      }
      console.log("[JS] Module loaded");
    }
  } catch (e) {
    if (e instanceof WebAssembly.CompileError || e instanceof WebAssembly.LinkError) {
      console.log(`[JS] exception during wasm compilation: ${e}`);
    } else if (e instanceof WebAssembly.RuntimeError) {
      console.log(`[JS] wasm aborted`);
    } else {
      console.log(`[JS] unexpected exception: ${e}`);
    }
    console.log(`[JS] Unknown error...`);
    console.log(e.stack);
    console.log(`[JS] destroying environment...`);

    // reset everything
    halt_next_tick = true;
  }
};
