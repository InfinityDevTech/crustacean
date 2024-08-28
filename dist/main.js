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
function isLikeNone(x) {
  return x === undefined || x === null;
}
let cachedFloat64Memory0 = null;
function getFloat64Memory0() {
  if (cachedFloat64Memory0 === null || cachedFloat64Memory0.byteLength === 0) {
    cachedFloat64Memory0 = new Float64Array(wasm.memory.buffer);
  }
  return cachedFloat64Memory0;
}
let cachedInt32Memory0 = null;
function getInt32Memory0() {
  if (cachedInt32Memory0 === null || cachedInt32Memory0.byteLength === 0) {
    cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
  }
  return cachedInt32Memory0;
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
let cachedUint8Memory0 = null;
function getUint8Memory0() {
  if (cachedUint8Memory0 === null || cachedUint8Memory0.byteLength === 0) {
    cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
  }
  return cachedUint8Memory0;
}
function getStringFromWasm0(ptr, len) {
  ptr = ptr >>> 0;
  return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}
let WASM_VECTOR_LEN = 0;
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
    getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
    WASM_VECTOR_LEN = buf.length;
    return ptr;
  }
  let len = arg.length;
  let ptr = malloc(len, 1) >>> 0;
  const mem = getUint8Memory0();
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
    const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
    const ret = encodeString(arg, view);
    offset += ret.written;
    ptr = realloc(ptr, len, offset, 1) >>> 0;
  }
  WASM_VECTOR_LEN = offset;
  return ptr;
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
  const ret = wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h5c6f53d5aab22de2(arg0, arg1, addHeapObject(arg2));
  return takeObject(ret);
}
function __wbg_adapter_41(arg0, arg1, arg2, arg3) {
  const ret = wasm._dyn_core__ops__function__FnMut__A_B___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h0b62e9590d8c7240(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
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
} : new FinalizationRegistry(ptr => wasm.__wbg_searchgoal_free(ptr >>> 0));
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
    wasm.__wbg_searchgoal_free(ptr);
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
  imports.wbg.__wbindgen_cb_drop = function (arg0) {
    const obj = takeObject(arg0).original;
    if (obj.cnt-- == 1) {
      obj.a = 0;
      return true;
    }
    const ret = false;
    return ret;
  };
  imports.wbg.__wbindgen_is_object = function (arg0) {
    const val = getObject(arg0);
    const ret = typeof val === 'object' && val !== null;
    return ret;
  };
  imports.wbg.__wbindgen_number_get = function (arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof obj === 'number' ? obj : undefined;
    getFloat64Memory0()[arg0 / 8 + 1] = isLikeNone(ret) ? 0 : ret;
    getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
  };
  imports.wbg.__wbindgen_error_new = function (arg0, arg1) {
    const ret = new Error(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
  };
  imports.wbg.__wbindgen_string_get = function (arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof obj === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
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
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
  };
  imports.wbg.__wbindgen_is_null = function (arg0) {
    const ret = getObject(arg0) === null;
    return ret;
  };
  imports.wbg.__wbindgen_is_undefined = function (arg0) {
    const ret = getObject(arg0) === undefined;
    return ret;
  };
  imports.wbg.__wbindgen_in = function (arg0, arg1) {
    const ret = getObject(arg0) in getObject(arg1);
    return ret;
  };
  imports.wbg.__wbg_log_5bb5f88f245d7762 = function (arg0) {
    console.log(getObject(arg0));
  };
  imports.wbg.__wbg_structuretype_bfe4606c302aee7a = function (arg0) {
    const ret = getObject(arg0).structureType;
    return addHeapObject(ret);
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
  imports.wbg.__wbg_getRawBuffer_4d47f548254d91a2 = function (arg0, arg1) {
    const ret = getObject(arg0).getRawBuffer(getObject(arg1));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_path_1ab03fe9a33dbb68 = function (arg0) {
    const ret = getObject(arg0).path;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_search_2f7aa7773fc05e8d = function (arg0, arg1, arg2) {
    const ret = PathFinder.search(getObject(arg0), getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_roomcallback_6e4061099bd831fe = function (arg0, arg1) {
    getObject(arg0).roomCallback = getObject(arg1);
  };
  imports.wbg.__wbg_cost_a5a1c3015e27f7b0 = function (arg0) {
    const ret = getObject(arg0).cost;
    return ret;
  };
  imports.wbg.__wbg_incomplete_3a20ecae23c2516a = function (arg0) {
    const ret = getObject(arg0).incomplete;
    return ret;
  };
  imports.wbg.__wbg_setpacked_6e344813289201ab = function (arg0, arg1) {
    getObject(arg0).__packedPos = arg1 >>> 0;
  };
  imports.wbg.__wbg_name_cbcf6cfa99528834 = function () {
    const ret = Game.shard.name;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_addVisual_0c90cac2cd246597 = function (arg0, arg1) {
    console.addVisual(getObject(arg0), getObject(arg1));
  };
  imports.wbg.__wbg_getCapacity_e13816dbd7dc7a0a = function (arg0, arg1, arg2) {
    const ret = getObject(arg1).getCapacity(takeObject(arg2));
    getInt32Memory0()[arg0 / 4 + 1] = isLikeNone(ret) ? 0 : ret;
    getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
  };
  imports.wbg.__wbg_getFreeCapacity_1d9caf416b1fa82b = function (arg0, arg1, arg2) {
    const ret = getObject(arg1).getFreeCapacity(takeObject(arg2));
    getInt32Memory0()[arg0 / 4 + 1] = isLikeNone(ret) ? 0 : ret;
    getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
  };
  imports.wbg.__wbg_getUsedCapacity_408c0c172e8e37af = function (arg0, arg1, arg2) {
    const ret = getObject(arg1).getUsedCapacity(takeObject(arg2));
    getInt32Memory0()[arg0 / 4 + 1] = isLikeNone(ret) ? 0 : ret;
    getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
  };
  imports.wbg.__wbg_hitsinternal_8e06c8128599ee74 = function (arg0, arg1) {
    const ret = getObject(arg1).hits;
    getInt32Memory0()[arg0 / 4 + 1] = isLikeNone(ret) ? 0 : ret;
    getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
  };
  imports.wbg.__wbg_hitsmaxinternal_47962c1a93a258cc = function (arg0, arg1) {
    const ret = getObject(arg1).hitsMax;
    getInt32Memory0()[arg0 / 4 + 1] = isLikeNone(ret) ? 0 : ret;
    getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
  };
  imports.wbg.__wbg_get_703902b2e6d4d69f = function (arg0, arg1, arg2) {
    const ret = getObject(arg0).get(arg1, arg2);
    return ret;
  };
  imports.wbg.__wbg_getRawBuffer_8e83d03ddb3e99a7 = function (arg0) {
    const ret = getObject(arg0).getRawBuffer();
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_constructionsites_f95d1b6f317b02ba = function () {
    const ret = Game.constructionSites;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_creeps_141fbfbb5741c053 = function () {
    const ret = Game.creeps;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_flags_10401f6e35cf81db = function () {
    const ret = Game.flags;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_resources_03e8119be3e56935 = function () {
    const ret = Game.resources;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_rooms_7e266606d09702bc = function () {
    const ret = Game.rooms;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_time_0d192985f6f956ea = function () {
    const ret = Game.time;
    return ret;
  };
  imports.wbg.__wbg_getObjectById_f430ecc8387f6a76 = function (arg0) {
    const ret = Game.getObjectById(getObject(arg0));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_notify_ad8756caab03bdc4 = function (arg0, arg1, arg2) {
    Game.notify(getObject(arg0), arg1 === 0 ? undefined : arg2 >>> 0);
  };
  imports.wbg.__wbg_bodyinternal_76502b0b503ecf0a = function (arg0) {
    const ret = getObject(arg0).body;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_fatigueinternal_e3f04dfea3ccb197 = function (arg0) {
    const ret = getObject(arg0).fatigue;
    return ret;
  };
  imports.wbg.__wbg_hitsinternal_b6315db17cc606df = function (arg0) {
    const ret = getObject(arg0).hits;
    return ret;
  };
  imports.wbg.__wbg_hitsmaxinternal_d1f78266a05116ee = function (arg0) {
    const ret = getObject(arg0).hitsMax;
    return ret;
  };
  imports.wbg.__wbg_myinternal_7e1129cd5ecdfaa8 = function (arg0) {
    const ret = getObject(arg0).my;
    return ret;
  };
  imports.wbg.__wbg_ownerinternal_ffd438044211dd63 = function (arg0) {
    const ret = getObject(arg0).owner;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_spawninginternal_3775b3cf7a8cad31 = function (arg0) {
    const ret = getObject(arg0).spawning;
    return ret;
  };
  imports.wbg.__wbg_storeinternal_43d810b402158bd8 = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_tickstoliveinternal_50cbdaf1ad8c2b12 = function (arg0, arg1) {
    const ret = getObject(arg1).ticksToLive;
    getInt32Memory0()[arg0 / 4 + 1] = isLikeNone(ret) ? 0 : ret;
    getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
  };
  imports.wbg.__wbg_attackController_22d871b3946b5007 = function (arg0, arg1) {
    const ret = Creep.prototype.attackController.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_build_2542e2c236480a23 = function (arg0, arg1) {
    const ret = Creep.prototype.build.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_claimController_1657e79eaac25279 = function (arg0, arg1) {
    const ret = Creep.prototype.claimController.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_drop_95f5766b4b3c8b9b = function (arg0, arg1, arg2, arg3) {
    const ret = Creep.prototype.drop.call(getObject(arg0), takeObject(arg1), arg2 === 0 ? undefined : arg3 >>> 0);
    return ret;
  };
  imports.wbg.__wbg_move_0271d4c281dae7c0 = function (arg0, arg1) {
    const ret = Creep.prototype.move.call(getObject(arg0), arg1);
    return ret;
  };
  imports.wbg.__wbg_notifyWhenAttacked_52fa1287e42a735d = function (arg0, arg1) {
    const ret = Creep.prototype.notifyWhenAttacked.call(getObject(arg0), arg1 !== 0);
    return ret;
  };
  imports.wbg.__wbg_pickup_b1912ef16d472ad9 = function (arg0, arg1) {
    const ret = Creep.prototype.pickup.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_rangedMassAttack_600276bc1f2d2650 = function (arg0) {
    const ret = Creep.prototype.rangedMassAttack.call(getObject(arg0));
    return ret;
  };
  imports.wbg.__wbg_reserveController_7b792c683fd64766 = function (arg0, arg1) {
    const ret = Creep.prototype.reserveController.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_say_fbe6f5a0ff7800b7 = function (arg0, arg1, arg2, arg3) {
    const ret = Creep.prototype.say.call(getObject(arg0), getStringFromWasm0(arg1, arg2), arg3 !== 0);
    return ret;
  };
  imports.wbg.__wbg_signController_9156608ff574ac4e = function (arg0, arg1, arg2, arg3) {
    const ret = Creep.prototype.signController.call(getObject(arg0), getObject(arg1), getStringFromWasm0(arg2, arg3));
    return ret;
  };
  imports.wbg.__wbg_suicide_e0dafaff14435e77 = function (arg0) {
    const ret = Creep.prototype.suicide.call(getObject(arg0));
    return ret;
  };
  imports.wbg.__wbg_upgradeController_14394513eaf5e5db = function (arg0, arg1) {
    const ret = Creep.prototype.upgradeController.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_idinternal_9ca95b29d19dd2b8 = function (arg0) {
    const ret = getObject(arg0).id;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_nameinternal_95ffabd2bda4989e = function (arg0, arg1) {
    const ret = getObject(arg1).name;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
  };
  imports.wbg.__wbg_instanceof_Creep_4bb0472fa612a650 = function (arg0) {
    let result;
    try {
      result = getObject(arg0) instanceof Creep;
    } catch (_) {
      result = false;
    }
    const ret = result;
    return ret;
  };
  imports.wbg.__wbg_attack_7249084335f27260 = function (arg0, arg1) {
    const ret = Creep.prototype.attack.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_harvest_d088c136720d20bc = function (arg0, arg1) {
    const ret = Creep.prototype.harvest.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_heal_d62d0089c5422c1c = function (arg0, arg1) {
    const ret = Creep.prototype.heal.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_rangedAttack_87150513a247f3d3 = function (arg0, arg1) {
    const ret = Creep.prototype.rangedAttack.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_rangedHeal_6ca91c9c7984dfbd = function (arg0, arg1) {
    const ret = Creep.prototype.rangedHeal.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_repair_74e407c21b98fda8 = function (arg0, arg1) {
    const ret = Creep.prototype.repair.call(getObject(arg0), getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_transfer_efd13aa4f61b0f5e = function (arg0, arg1, arg2, arg3, arg4) {
    const ret = Creep.prototype.transfer.call(getObject(arg0), getObject(arg1), takeObject(arg2), arg3 === 0 ? undefined : arg4 >>> 0);
    return ret;
  };
  imports.wbg.__wbg_withdraw_3d88d7e33d9dd812 = function (arg0, arg1, arg2, arg3, arg4) {
    const ret = Creep.prototype.withdraw.call(getObject(arg0), getObject(arg1), takeObject(arg2), arg3 === 0 ? undefined : arg4 >>> 0);
    return ret;
  };
  imports.wbg.__wbg_part_aaff4ccd366f0983 = function (arg0) {
    const ret = getObject(arg0).type;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_hits_06d2bb77fc90eb46 = function (arg0) {
    const ret = getObject(arg0).hits;
    return ret;
  };
  imports.wbg.__wbg_idinternal_86ae208102215a0d = function (arg0) {
    const ret = getObject(arg0).id;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_destroy_cc5eccaa0920c90d = function (arg0) {
    const ret = getObject(arg0).destroy();
    return ret;
  };
  imports.wbg.__wbg_isActive_e4ab73c68937f415 = function (arg0) {
    const ret = getObject(arg0).isActive();
    return ret;
  };
  imports.wbg.__wbg_credits_41e61ed59e01fe2a = function () {
    const ret = Game.market.credits;
    return ret;
  };
  imports.wbg.__wbg_newinternal_c2fe9543479fcee6 = function (arg0, arg1, arg2) {
    const ret = new RoomPosition(arg0, arg1, getObject(arg2));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_roomnameinternal_8f6b38d9e344d141 = function (arg0) {
    const ret = getObject(arg0).roomName;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_packed_3da075786d3a3a14 = function (arg0) {
    const ret = getObject(arg0).__packedPos;
    return ret;
  };
  imports.wbg.__wbg_store_68d7fadc5dc1d2cd = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_store_cd5040ab303fffad = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_jspos_bcd38b3c41c48784 = function (arg0) {
    const ret = getObject(arg0).pos;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_room_291927c68d7d7b53 = function (arg0) {
    const ret = getObject(arg0).room;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_x_9a506e931c98e539 = function (arg0) {
    const ret = getObject(arg0).x;
    return ret;
  };
  imports.wbg.__wbg_y_5b5427eab5bcedc3 = function (arg0) {
    const ret = getObject(arg0).y;
    return ret;
  };
  imports.wbg.__wbg_findInRange_b7696b6893f373e6 = function (arg0, arg1, arg2, arg3) {
    const ret = getObject(arg0).findInRange(arg1, arg2, getObject(arg3));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_attack_4b64af2c497e32c8 = function (arg0, arg1) {
    const ret = getObject(arg0).attack(getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_heal_4ac0385487f89dd9 = function (arg0, arg1) {
    const ret = getObject(arg0).heal(getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_repair_24b3f2160ff1326c = function (arg0, arg1) {
    const ret = getObject(arg0).repair(getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_setbits_30335eca74a71e7c = function (arg0, arg1) {
    getObject(arg0)._bits = getObject(arg1);
  };
  imports.wbg.__wbg_unclaim_809fbc084bfb7d2d = function (arg0) {
    const ret = getObject(arg0).unclaim();
    return ret;
  };
  imports.wbg.__wbg_store_2b72f5d3ba7b3ce0 = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_level_533eecb826c91067 = function (arg0) {
    const ret = getObject(arg0).level;
    return ret;
  };
  imports.wbg.__wbg_progress_bfe88b84a4cbff77 = function (arg0, arg1) {
    const ret = getObject(arg1).progress;
    getInt32Memory0()[arg0 / 4 + 1] = isLikeNone(ret) ? 0 : ret;
    getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
  };
  imports.wbg.__wbg_progresstotal_3302ae70b633b347 = function (arg0, arg1) {
    const ret = getObject(arg1).progressTotal;
    getInt32Memory0()[arg0 / 4 + 1] = isLikeNone(ret) ? 0 : ret;
    getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
  };
  imports.wbg.__wbg_reservation_214d9f03b2b086a5 = function (arg0) {
    const ret = getObject(arg0).reservation;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_safemode_5cfb44b70a81284e = function (arg0, arg1) {
    const ret = getObject(arg1).safeMode;
    getInt32Memory0()[arg0 / 4 + 1] = isLikeNone(ret) ? 0 : ret;
    getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
  };
  imports.wbg.__wbg_sign_8d89aa2c3d1973ff = function (arg0) {
    const ret = getObject(arg0).sign;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_tickstodowngrade_269d46cc8703a78f = function (arg0, arg1) {
    const ret = getObject(arg1).ticksToDowngrade;
    getInt32Memory0()[arg0 / 4 + 1] = isLikeNone(ret) ? 0 : ret;
    getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
  };
  imports.wbg.__wbg_username_1f5ae6c74daf07c3 = function (arg0, arg1) {
    const ret = getObject(arg1).username;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
  };
  imports.wbg.__wbg_tickstoend_dbf8487a923401c1 = function (arg0) {
    const ret = getObject(arg0).ticksToEnd;
    return ret;
  };
  imports.wbg.__wbg_username_0a0fd29e6b3916f2 = function (arg0, arg1) {
    const ret = getObject(arg1).username;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
  };
  imports.wbg.__wbg_text_4e80dd42584ae8c4 = function (arg0, arg1) {
    const ret = getObject(arg1).text;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
  };
  imports.wbg.__wbg_static_accessor_ROOM_POSITION_PROTOTYPE_47316197ec1743ed = function () {
    const ret = RoomPosition.prototype;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_static_accessor_COST_MATRIX_PROTOTYPE_dbcfff99ab8e50a3 = function () {
    const ret = PathFinder.CostMatrix.prototype;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_level_2830e786fac9de30 = function () {
    const ret = Game.gcl.level;
    return ret;
  };
  imports.wbg.__wbg_progress_7a76f9ec7e3747bd = function () {
    const ret = Game.gcl.progress;
    return ret;
  };
  imports.wbg.__wbg_progressTotal_8429c133f94e8830 = function () {
    const ret = Game.gcl.progressTotal;
    return ret;
  };
  imports.wbg.__wbg_idinternal_1c699ee7072145fe = function (arg0) {
    const ret = getObject(arg0).id;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_mineraltype_2c0b05ad9c77ccc9 = function (arg0) {
    const ret = getObject(arg0).mineralType;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_energy_870a7efef3e88a80 = function (arg0) {
    const ret = getObject(arg0).energy;
    return ret;
  };
  imports.wbg.__wbg_energycapacity_b3f71ef01927f92c = function (arg0) {
    const ret = getObject(arg0).energyCapacity;
    return ret;
  };
  imports.wbg.__wbg_x_bfb36b020e3f57bb = function (arg0) {
    const ret = getObject(arg0).x;
    return ret;
  };
  imports.wbg.__wbg_y_a77f823c8ed815ec = function (arg0) {
    const ret = getObject(arg0).y;
    return ret;
  };
  imports.wbg.__wbg_level_d9881742526cf903 = function () {
    const ret = Game.gpl.level;
    return ret;
  };
  imports.wbg.__wbg_myinternal_fb20e7046e165ca3 = function (arg0) {
    const ret = getObject(arg0).my;
    return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
  };
  imports.wbg.__wbg_store_03e20848c1939256 = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_set_a96338392819c6a7 = function (arg0, arg1, arg2, arg3) {
    getObject(arg0)[getStringFromWasm0(arg1, arg2)] = getObject(arg3);
  };
  imports.wbg.__wbg_getvalue_2c82c5286b18f952 = function (arg0, arg1) {
    const ret = getObject(arg0)[getObject(arg1)];
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_owner_839ff6b662a83496 = function (arg0) {
    const ret = getObject(arg0).owner;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_username_be178158b2f0f8d5 = function (arg0, arg1) {
    const ret = getObject(arg1).username;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
  };
  imports.wbg.__wbg_creep_470870c6addffc80 = function (arg0) {
    const ret = getObject(arg0).creep;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_energy_e67b044058f50970 = function (arg0) {
    const ret = getObject(arg0).energy;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_resource_6001b90c46641cff = function (arg0) {
    const ret = getObject(arg0).resource;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_source_4ebe5de8b813dfa5 = function (arg0) {
    const ret = getObject(arg0).source;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_mineral_8b31f3a1d1d928ae = function (arg0) {
    const ret = getObject(arg0).mineral;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_structure_1036c13279c78568 = function (arg0) {
    const ret = getObject(arg0).structure;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_flag_49b925d335ab7536 = function (arg0) {
    const ret = getObject(arg0).flag;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_constructionsite_9b95a2dc5d545e35 = function (arg0) {
    const ret = getObject(arg0).constructionSite;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_nuke_0e46c74ee55afc53 = function (arg0) {
    const ret = getObject(arg0).nuke;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_terrain_ac9befcd1f72f6ac = function (arg0, arg1) {
    const ret = getObject(arg1).terrain;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
  };
  imports.wbg.__wbg_tombstone_ad56afd7bcb320f3 = function (arg0) {
    const ret = getObject(arg0).tombstone;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_powercreep_1be30cad332d64bd = function (arg0) {
    const ret = getObject(arg0).powerCreep;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_deposit_cb58830fad65a6a3 = function (arg0) {
    const ret = getObject(arg0).deposit;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_ruin_84ad1fd9272f2db8 = function (arg0) {
    const ret = getObject(arg0).ruin;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_idinternal_7bcee6134cea273f = function (arg0) {
    const ret = getObject(arg0).id;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_store_9f49f6fad2cc9f5c = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_spawnCreep_b1466fcddce096b0 = function (arg0, arg1, arg2, arg3, arg4) {
    const ret = getObject(arg0).spawnCreep(getObject(arg1), getStringFromWasm0(arg2, arg3), getObject(arg4));
    return ret;
  };
  imports.wbg.__wbg_recycleCreep_57dd532709a7144d = function (arg0, arg1) {
    const ret = getObject(arg0).recycleCreep(getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_renewCreep_02a4c2abce8d2a18 = function (arg0, arg1) {
    const ret = getObject(arg0).renewCreep(getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_store_a807d3be01cf06f0 = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_idinternal_84aef462b9875ae9 = function (arg0) {
    const ret = getObject(arg0).id;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_store_15041cce0f8b0b9e = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_amount_28d12c16e39e6179 = function (arg0) {
    const ret = getObject(arg0).amount;
    return ret;
  };
  imports.wbg.__wbg_resourcetype_05dcf501c1f11eaa = function (arg0) {
    const ret = getObject(arg0).resourceType;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_level_8766613800b80ae9 = function (arg0) {
    const ret = getObject(arg0).level;
    return ret;
  };
  imports.wbg.__wbg_spawning_67082ae7975d76d0 = function (arg0) {
    const ret = getObject(arg0).spawning;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_name_ade592e738910c14 = function (arg0) {
    const ret = getObject(arg0).name;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_needtime_0f45e26d6c149097 = function (arg0) {
    const ret = getObject(arg0).needTime;
    return ret;
  };
  imports.wbg.__wbg_remainingtime_5cce3fb46989ec1a = function (arg0) {
    const ret = getObject(arg0).remainingTime;
    return ret;
  };
  imports.wbg.__wbg_describeExits_aa19d621aade21d0 = function (arg0) {
    const ret = Game.map.describeExits(getObject(arg0));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_getRoomTerrain_48115eaf4d124fdd = function () {
    return handleError(function (arg0) {
      const ret = Game.map.getRoomTerrain(getObject(arg0));
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbg_getWorldSize_ecf1ede4c791ae5e = function () {
    const ret = Game.map.getWorldSize();
    return ret;
  };
  imports.wbg.__wbg_status_3573f003d3153be9 = function (arg0) {
    const ret = getObject(arg0).status;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_timestamp_51dfbc2ba8dcb051 = function (arg0, arg1) {
    const ret = getObject(arg1).timestamp;
    getFloat64Memory0()[arg0 / 8 + 1] = isLikeNone(ret) ? 0 : ret;
    getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
  };
  imports.wbg.__wbg_getRoomStatus_fbf7b602150efb20 = function () {
    return handleError(function (arg0) {
      const ret = Game.map.getRoomStatus(getObject(arg0));
      return addHeapObject(ret);
    }, arguments);
  };
  imports.wbg.__wbg_nameinternal_e33640de24ffe358 = function (arg0) {
    const ret = getObject(arg0).name;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_createConstructionSite_9b4be48e1334879c = function (arg0, arg1, arg2, arg3, arg4) {
    const ret = Room.prototype.createConstructionSite.call(getObject(arg0), arg1, arg2, takeObject(arg3), getObject(arg4));
    return ret;
  };
  imports.wbg.__wbg_getEventLog_fdd9a96b9267c901 = function (arg0, arg1) {
    const ret = Room.prototype.getEventLog.call(getObject(arg0), arg1 !== 0);
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_findRoute_7457404c11add931 = function (arg0, arg1, arg2) {
    const ret = Game.map.findRoute(getObject(arg0), getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_routecallback_e662f5035cee6509 = function (arg0, arg1) {
    getObject(arg0).routeCallback = getObject(arg1);
  };
  imports.wbg.__wbg_color_e42332ad4f3374a5 = function (arg0) {
    const ret = getObject(arg0).color;
    return ret;
  };
  imports.wbg.__wbg_name_0b5212969d6d9458 = function (arg0, arg1) {
    const ret = getObject(arg1).name;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
  };
  imports.wbg.__wbg_remove_5bd02be53f70b936 = function (arg0) {
    getObject(arg0).remove();
  };
  imports.wbg.__wbg_controller_f8b715979fbc78ee = function (arg0) {
    const ret = getObject(arg0).controller;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_energyavailable_635decbf0ea7ddea = function (arg0) {
    const ret = getObject(arg0).energyAvailable;
    return ret;
  };
  imports.wbg.__wbg_energycapacityavailable_0b57c1ed5d11acad = function (arg0) {
    const ret = getObject(arg0).energyCapacityAvailable;
    return ret;
  };
  imports.wbg.__wbg_find_6c426115681aec8b = function (arg0, arg1, arg2) {
    const ret = getObject(arg0).find(arg1, getObject(arg2));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_getTerrain_fb15d174613e42e4 = function (arg0) {
    const ret = Room.prototype.getTerrain.call(getObject(arg0));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_lookForAt_9bcaab4d800184fd = function (arg0, arg1, arg2, arg3) {
    const ret = Room.prototype.lookForAt.call(getObject(arg0), takeObject(arg1), arg2, arg3);
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_lookForAtArea_74d63103549899ea = function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    const ret = Room.prototype.lookForAtArea.call(getObject(arg0), takeObject(arg1), arg2, arg3, arg4, arg5, arg6 !== 0);
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_limit_3137c74d9aa8a7f0 = function () {
    const ret = Game.cpu.limit;
    return ret;
  };
  imports.wbg.__wbg_tickLimit_211af469f5e6300e = function () {
    const ret = Game.cpu.tickLimit;
    return ret;
  };
  imports.wbg.__wbg_bucket_155118e5aeeed1f2 = function () {
    const ret = Game.cpu.bucket;
    return ret;
  };
  imports.wbg.__wbg_getHeapStatistics_da7643b372604847 = function () {
    const ret = Game.cpu.getHeapStatistics();
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_getUsed_5f6b24a07d6fb984 = function () {
    const ret = Game.cpu.getUsed();
    return ret;
  };
  imports.wbg.__wbg_generatePixel_c68cae0080902ff8 = function () {
    const ret = Game.cpu.generatePixel();
    return ret;
  };
  imports.wbg.__wbg_progressinternal_5f49ba9b39eb45d2 = function (arg0) {
    const ret = getObject(arg0).progress;
    return ret;
  };
  imports.wbg.__wbg_progresstotalinternal_e716e276fff15a5d = function (arg0) {
    const ret = getObject(arg0).progressTotal;
    return ret;
  };
  imports.wbg.__wbg_structuretypeinternal_dbeb619440cf14d0 = function (arg0) {
    const ret = getObject(arg0).structureType;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_remove_79db9c8399420416 = function (arg0) {
    const ret = getObject(arg0).remove();
    return ret;
  };
  imports.wbg.__wbg_idinternal_5aff2b34c07664b3 = function (arg0) {
    const ret = getObject(arg0).id;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_store_fa998ef0a68fbf89 = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_transferEnergy_be79cf4875fbb39c = function (arg0, arg1, arg2, arg3) {
    const ret = getObject(arg0).transferEnergy(getObject(arg1), arg2 === 0 ? undefined : arg3 >>> 0);
    return ret;
  };
  imports.wbg.__wbg_store_b5fa8f55f736cfd7 = function (arg0) {
    const ret = getObject(arg0).store;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_foreignsegment_141f9124081c6257 = function () {
    const ret = RawMemory.foreignSegment;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
  };
  imports.wbg.__wbg_get_9971a5576abd792f = function () {
    const ret = RawMemory.get();
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_set_fe2b91460de2b577 = function (arg0) {
    RawMemory.set(getObject(arg0));
  };
  imports.wbg.__wbg_setActiveSegments_8ce8643e8d118509 = function (arg0) {
    RawMemory.setActiveSegments(getObject(arg0));
  };
  imports.wbg.__wbg_setActiveForeignSegment_c61151ff27192e58 = function (arg0, arg1) {
    RawMemory.setActiveForeignSegment(getObject(arg0), arg1 === 0xFFFFFF ? undefined : arg1);
  };
  imports.wbg.__wbg_totalheapsize_f41d9dec03c2a57e = function (arg0) {
    const ret = getObject(arg0).total_heap_size;
    return ret;
  };
  imports.wbg.__wbg_heapsizelimit_94ee03fe477afdc3 = function (arg0) {
    const ret = getObject(arg0).heap_size_limit;
    return ret;
  };
  imports.wbg.__wbg_externallyallocatedsize_d78234ab7bbe6457 = function (arg0) {
    const ret = getObject(arg0).externally_allocated_size;
    return ret;
  };
  imports.wbg.__wbg_ispublic_9389a8c09939bc16 = function (arg0) {
    const ret = getObject(arg0).isPublic;
    return ret;
  };
  imports.wbg.__wbg_username_50a6289765336e00 = function (arg0) {
    const ret = getObject(arg0).username;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_id_c8a3b2726b5b93f1 = function (arg0) {
    const ret = getObject(arg0).id;
    return ret;
  };
  imports.wbg.__wbg_data_b7bc456f4c7ab590 = function (arg0) {
    const ret = getObject(arg0).data;
    return addHeapObject(ret);
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
  imports.wbg.__wbg_get_bd8e338fbd5f5cc8 = function (arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_length_cd7af8117672b8b8 = function (arg0) {
    const ret = getObject(arg0).length;
    return ret;
  };
  imports.wbg.__wbg_new_16b304a2cfa7ff4a = function () {
    const ret = new Array();
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_new_72fb9a18b5ae2624 = function () {
    const ret = new Object();
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_set_d4638f722068f043 = function (arg0, arg1, arg2) {
    getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
  };
  imports.wbg.__wbg_push_a5b05aedc7234f9f = function (arg0, arg1) {
    const ret = getObject(arg0).push(getObject(arg1));
    return ret;
  };
  imports.wbg.__wbg_instanceof_ArrayBuffer_836825be07d4c9d2 = function (arg0) {
    let result;
    try {
      result = getObject(arg0) instanceof ArrayBuffer;
    } catch (_) {
      result = false;
    }
    const ret = result;
    return ret;
  };
  imports.wbg.__wbg_isSafeInteger_f7b04ef02296c4d2 = function (arg0) {
    const ret = Number.isSafeInteger(getObject(arg0));
    return ret;
  };
  imports.wbg.__wbg_create_a4affbe2b1332881 = function (arg0) {
    const ret = Object.create(getObject(arg0));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_keys_91e412b4b222659f = function (arg0) {
    const ret = Object.keys(getObject(arg0));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_values_9c75e6e2bfbdb70d = function (arg0) {
    const ret = Object.values(getObject(arg0));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_buffer_12d079cc21e14bdb = function (arg0) {
    const ret = getObject(arg0).buffer;
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_newwithbyteoffsetandlength_aa4a17c33a06e5cb = function (arg0, arg1, arg2) {
    const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_new_63b92bc8671ed464 = function (arg0) {
    const ret = new Uint8Array(getObject(arg0));
    return addHeapObject(ret);
  };
  imports.wbg.__wbg_set_a47bac70306a19a7 = function (arg0, arg1, arg2) {
    getObject(arg0).set(getObject(arg1), arg2 >>> 0);
  };
  imports.wbg.__wbg_length_c20a40f15020d68a = function (arg0) {
    const ret = getObject(arg0).length;
    return ret;
  };
  imports.wbg.__wbg_instanceof_Uint8Array_2b3bbecd033d19f6 = function (arg0) {
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
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
  };
  imports.wbg.__wbindgen_throw = function (arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
  };
  imports.wbg.__wbindgen_memory = function () {
    const ret = wasm.memory;
    return addHeapObject(ret);
  };
  imports.wbg.__wbindgen_closure_wrapper644 = function (arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 76, __wbg_adapter_38);
    return addHeapObject(ret);
  };
  imports.wbg.__wbindgen_closure_wrapper646 = function (arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 76, __wbg_adapter_41);
    return addHeapObject(ret);
  };
  return imports;
}
function __wbg_finalize_init(instance, module) {
  wasm = instance.exports;
  cachedFloat64Memory0 = null;
  cachedInt32Memory0 = null;
  cachedUint8Memory0 = null;
  return wasm;
}
function initSync(module) {
  if (wasm !== undefined) return wasm;
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
