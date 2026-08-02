/* Auto-generated from verify/js — do not edit by hand.
 * Run: node extensions/browser/scripts/bundle-from-verify-js.js
 * Copyright (c) 2026 MonkeyKing.dev
 */
(function (global) {
  "use strict";
  // Minimal Buffer polyfill for hex (browser)
  if (typeof Buffer === "undefined") {
    global.Buffer = {
      from: function (u8) {
        return {
          toString: function (enc) {
            if (enc !== "hex") throw new Error("Buffer polyfill only supports hex");
            var hex = "";
            for (var i = 0; i < u8.length; i++) hex += u8[i].toString(16).padStart(2, "0");
            return hex;
          }
        };
      }
    };
  }
  var modules = Object.create(null);
  function require(id) {
    var m = modules[id];
    if (!m) throw new Error("hashseal-verify browser bundle: missing module " + id);
    return m.exports;
  }
  function define(id, factory) {
    var module = { exports: {} };
    modules[id] = module;
    factory(require, module, module.exports);
  }
  define("vendor/noble/crypto.js", function (require, module, exports) {
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.crypto = void 0;
exports.crypto = typeof globalThis === 'object' && 'crypto' in globalThis ? globalThis.crypto : undefined;
//# sourceMappingURL=crypto.js.map
  });
  define("vendor/noble/_assert.js", function (require, module, exports) {
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isBytes = isBytes;
exports.number = number;
exports.bool = bool;
exports.bytes = bytes;
exports.hash = hash;
exports.exists = exists;
exports.output = output;
function number(n) {
    if (!Number.isSafeInteger(n) || n < 0)
        throw new Error(`positive integer expected, not ${n}`);
}
function bool(b) {
    if (typeof b !== 'boolean')
        throw new Error(`boolean expected, not ${b}`);
}
// copied from utils
function isBytes(a) {
    return (a instanceof Uint8Array ||
        (a != null && typeof a === 'object' && a.constructor.name === 'Uint8Array'));
}
function bytes(b, ...lengths) {
    if (!isBytes(b))
        throw new Error('Uint8Array expected');
    if (lengths.length > 0 && !lengths.includes(b.length))
        throw new Error(`Uint8Array expected of length ${lengths}, not of length=${b.length}`);
}
function hash(h) {
    if (typeof h !== 'function' || typeof h.create !== 'function')
        throw new Error('Hash should be wrapped by utils.wrapConstructor');
    number(h.outputLen);
    number(h.blockLen);
}
function exists(instance, checkFinished = true) {
    if (instance.destroyed)
        throw new Error('Hash instance has been destroyed');
    if (checkFinished && instance.finished)
        throw new Error('Hash#digest() has already been called');
}
function output(out, instance) {
    bytes(out);
    const min = instance.outputLen;
    if (out.length < min) {
        throw new Error(`digestInto() expects output buffer of length at least ${min}`);
    }
}
const assert = { number, bool, bytes, hash, exists, output };
exports.default = assert;
//# sourceMappingURL=_assert.js.map
  });
  define("vendor/noble/_u64.js", function (require, module, exports) {
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.add5L = exports.add5H = exports.add4H = exports.add4L = exports.add3H = exports.add3L = exports.rotlBL = exports.rotlBH = exports.rotlSL = exports.rotlSH = exports.rotr32L = exports.rotr32H = exports.rotrBL = exports.rotrBH = exports.rotrSL = exports.rotrSH = exports.shrSL = exports.shrSH = exports.toBig = void 0;
exports.fromBig = fromBig;
exports.split = split;
exports.add = add;
const U32_MASK64 = /* @__PURE__ */ BigInt(2 ** 32 - 1);
const _32n = /* @__PURE__ */ BigInt(32);
// We are not using BigUint64Array, because they are extremely slow as per 2022
function fromBig(n, le = false) {
    if (le)
        return { h: Number(n & U32_MASK64), l: Number((n >> _32n) & U32_MASK64) };
    return { h: Number((n >> _32n) & U32_MASK64) | 0, l: Number(n & U32_MASK64) | 0 };
}
function split(lst, le = false) {
    let Ah = new Uint32Array(lst.length);
    let Al = new Uint32Array(lst.length);
    for (let i = 0; i < lst.length; i++) {
        const { h, l } = fromBig(lst[i], le);
        [Ah[i], Al[i]] = [h, l];
    }
    return [Ah, Al];
}
const toBig = (h, l) => (BigInt(h >>> 0) << _32n) | BigInt(l >>> 0);
exports.toBig = toBig;
// for Shift in [0, 32)
const shrSH = (h, _l, s) => h >>> s;
exports.shrSH = shrSH;
const shrSL = (h, l, s) => (h << (32 - s)) | (l >>> s);
exports.shrSL = shrSL;
// Right rotate for Shift in [1, 32)
const rotrSH = (h, l, s) => (h >>> s) | (l << (32 - s));
exports.rotrSH = rotrSH;
const rotrSL = (h, l, s) => (h << (32 - s)) | (l >>> s);
exports.rotrSL = rotrSL;
// Right rotate for Shift in (32, 64), NOTE: 32 is special case.
const rotrBH = (h, l, s) => (h << (64 - s)) | (l >>> (s - 32));
exports.rotrBH = rotrBH;
const rotrBL = (h, l, s) => (h >>> (s - 32)) | (l << (64 - s));
exports.rotrBL = rotrBL;
// Right rotate for shift===32 (just swaps l&h)
const rotr32H = (_h, l) => l;
exports.rotr32H = rotr32H;
const rotr32L = (h, _l) => h;
exports.rotr32L = rotr32L;
// Left rotate for Shift in [1, 32)
const rotlSH = (h, l, s) => (h << s) | (l >>> (32 - s));
exports.rotlSH = rotlSH;
const rotlSL = (h, l, s) => (l << s) | (h >>> (32 - s));
exports.rotlSL = rotlSL;
// Left rotate for Shift in (32, 64), NOTE: 32 is special case.
const rotlBH = (h, l, s) => (l << (s - 32)) | (h >>> (64 - s));
exports.rotlBH = rotlBH;
const rotlBL = (h, l, s) => (h << (s - 32)) | (l >>> (64 - s));
exports.rotlBL = rotlBL;
// JS uses 32-bit signed integers for bitwise operations which means we cannot
// simple take carry out of low bit sum by shift, we need to use division.
function add(Ah, Al, Bh, Bl) {
    const l = (Al >>> 0) + (Bl >>> 0);
    return { h: (Ah + Bh + ((l / 2 ** 32) | 0)) | 0, l: l | 0 };
}
// Addition with more than 2 elements
const add3L = (Al, Bl, Cl) => (Al >>> 0) + (Bl >>> 0) + (Cl >>> 0);
exports.add3L = add3L;
const add3H = (low, Ah, Bh, Ch) => (Ah + Bh + Ch + ((low / 2 ** 32) | 0)) | 0;
exports.add3H = add3H;
const add4L = (Al, Bl, Cl, Dl) => (Al >>> 0) + (Bl >>> 0) + (Cl >>> 0) + (Dl >>> 0);
exports.add4L = add4L;
const add4H = (low, Ah, Bh, Ch, Dh) => (Ah + Bh + Ch + Dh + ((low / 2 ** 32) | 0)) | 0;
exports.add4H = add4H;
const add5L = (Al, Bl, Cl, Dl, El) => (Al >>> 0) + (Bl >>> 0) + (Cl >>> 0) + (Dl >>> 0) + (El >>> 0);
exports.add5L = add5L;
const add5H = (low, Ah, Bh, Ch, Dh, Eh) => (Ah + Bh + Ch + Dh + Eh + ((low / 2 ** 32) | 0)) | 0;
exports.add5H = add5H;
// prettier-ignore
const u64 = {
    fromBig, split, toBig,
    shrSH, shrSL,
    rotrSH, rotrSL, rotrBH, rotrBL,
    rotr32H, rotr32L,
    rotlSH, rotlSL, rotlBH, rotlBL,
    add, add3L, add3H, add4L, add4H, add5H, add5L,
};
exports.default = u64;
//# sourceMappingURL=_u64.js.map
  });
  define("vendor/noble/utils.js", function (require, module, exports) {
"use strict";
/*! noble-hashes - MIT License (c) 2022 Paul Miller (paulmillr.com) */
Object.defineProperty(exports, "__esModule", { value: true });
exports.Hash = exports.nextTick = exports.byteSwapIfBE = exports.byteSwap = exports.isLE = exports.rotl = exports.rotr = exports.createView = exports.u32 = exports.u8 = void 0;
exports.isBytes = isBytes;
exports.byteSwap32 = byteSwap32;
exports.bytesToHex = bytesToHex;
exports.hexToBytes = hexToBytes;
exports.asyncLoop = asyncLoop;
exports.utf8ToBytes = utf8ToBytes;
exports.toBytes = toBytes;
exports.concatBytes = concatBytes;
exports.checkOpts = checkOpts;
exports.wrapConstructor = wrapConstructor;
exports.wrapConstructorWithOpts = wrapConstructorWithOpts;
exports.wrapXOFConstructorWithOpts = wrapXOFConstructorWithOpts;
exports.randomBytes = randomBytes;
// We use WebCrypto aka globalThis.crypto, which exists in browsers and node.js 16+.
// node.js versions earlier than v19 don't declare it in global scope.
// For node.js, package.json#exports field mapping rewrites import
// from `crypto` to `cryptoNode`, which imports native module.
// Makes the utils un-importable in browsers without a bundler.
// Once node.js 18 is deprecated (2025-04-30), we can just drop the import.
const crypto_1 = require("vendor/noble/crypto.js");
const _assert_js_1 = require("vendor/noble/_assert.js");
// export { isBytes } from './_assert.js';
// We can't reuse isBytes from _assert, because somehow this causes huge perf issues
function isBytes(a) {
    return (a instanceof Uint8Array ||
        (a != null && typeof a === 'object' && a.constructor.name === 'Uint8Array'));
}
// Cast array to different type
const u8 = (arr) => new Uint8Array(arr.buffer, arr.byteOffset, arr.byteLength);
exports.u8 = u8;
const u32 = (arr) => new Uint32Array(arr.buffer, arr.byteOffset, Math.floor(arr.byteLength / 4));
exports.u32 = u32;
// Cast array to view
const createView = (arr) => new DataView(arr.buffer, arr.byteOffset, arr.byteLength);
exports.createView = createView;
// The rotate right (circular right shift) operation for uint32
const rotr = (word, shift) => (word << (32 - shift)) | (word >>> shift);
exports.rotr = rotr;
// The rotate left (circular left shift) operation for uint32
const rotl = (word, shift) => (word << shift) | ((word >>> (32 - shift)) >>> 0);
exports.rotl = rotl;
exports.isLE = new Uint8Array(new Uint32Array([0x11223344]).buffer)[0] === 0x44;
// The byte swap operation for uint32
const byteSwap = (word) => ((word << 24) & 0xff000000) |
    ((word << 8) & 0xff0000) |
    ((word >>> 8) & 0xff00) |
    ((word >>> 24) & 0xff);
exports.byteSwap = byteSwap;
// Conditionally byte swap if on a big-endian platform
exports.byteSwapIfBE = exports.isLE ? (n) => n : (n) => (0, exports.byteSwap)(n);
// In place byte swap for Uint32Array
function byteSwap32(arr) {
    for (let i = 0; i < arr.length; i++) {
        arr[i] = (0, exports.byteSwap)(arr[i]);
    }
}
// Array where index 0xf0 (240) is mapped to string 'f0'
const hexes = /* @__PURE__ */ Array.from({ length: 256 }, (_, i) => i.toString(16).padStart(2, '0'));
/**
 * @example bytesToHex(Uint8Array.from([0xca, 0xfe, 0x01, 0x23])) // 'cafe0123'
 */
function bytesToHex(bytes) {
    (0, _assert_js_1.bytes)(bytes);
    // pre-caching improves the speed 6x
    let hex = '';
    for (let i = 0; i < bytes.length; i++) {
        hex += hexes[bytes[i]];
    }
    return hex;
}
// We use optimized technique to convert hex string to byte array
const asciis = { _0: 48, _9: 57, _A: 65, _F: 70, _a: 97, _f: 102 };
function asciiToBase16(char) {
    if (char >= asciis._0 && char <= asciis._9)
        return char - asciis._0;
    if (char >= asciis._A && char <= asciis._F)
        return char - (asciis._A - 10);
    if (char >= asciis._a && char <= asciis._f)
        return char - (asciis._a - 10);
    return;
}
/**
 * @example hexToBytes('cafe0123') // Uint8Array.from([0xca, 0xfe, 0x01, 0x23])
 */
function hexToBytes(hex) {
    if (typeof hex !== 'string')
        throw new Error('hex string expected, got ' + typeof hex);
    const hl = hex.length;
    const al = hl / 2;
    if (hl % 2)
        throw new Error('padded hex string expected, got unpadded hex of length ' + hl);
    const array = new Uint8Array(al);
    for (let ai = 0, hi = 0; ai < al; ai++, hi += 2) {
        const n1 = asciiToBase16(hex.charCodeAt(hi));
        const n2 = asciiToBase16(hex.charCodeAt(hi + 1));
        if (n1 === undefined || n2 === undefined) {
            const char = hex[hi] + hex[hi + 1];
            throw new Error('hex string expected, got non-hex character "' + char + '" at index ' + hi);
        }
        array[ai] = n1 * 16 + n2;
    }
    return array;
}
// There is no setImmediate in browser and setTimeout is slow.
// call of async fn will return Promise, which will be fullfiled only on
// next scheduler queue processing step and this is exactly what we need.
const nextTick = async () => { };
exports.nextTick = nextTick;
// Returns control to thread each 'tick' ms to avoid blocking
async function asyncLoop(iters, tick, cb) {
    let ts = Date.now();
    for (let i = 0; i < iters; i++) {
        cb(i);
        // Date.now() is not monotonic, so in case if clock goes backwards we return return control too
        const diff = Date.now() - ts;
        if (diff >= 0 && diff < tick)
            continue;
        await (0, exports.nextTick)();
        ts += diff;
    }
}
/**
 * @example utf8ToBytes('abc') // new Uint8Array([97, 98, 99])
 */
function utf8ToBytes(str) {
    if (typeof str !== 'string')
        throw new Error(`utf8ToBytes expected string, got ${typeof str}`);
    return new Uint8Array(new TextEncoder().encode(str)); // https://bugzil.la/1681809
}
/**
 * Normalizes (non-hex) string or Uint8Array to Uint8Array.
 * Warning: when Uint8Array is passed, it would NOT get copied.
 * Keep in mind for future mutable operations.
 */
function toBytes(data) {
    if (typeof data === 'string')
        data = utf8ToBytes(data);
    (0, _assert_js_1.bytes)(data);
    return data;
}
/**
 * Copies several Uint8Arrays into one.
 */
function concatBytes(...arrays) {
    let sum = 0;
    for (let i = 0; i < arrays.length; i++) {
        const a = arrays[i];
        (0, _assert_js_1.bytes)(a);
        sum += a.length;
    }
    const res = new Uint8Array(sum);
    for (let i = 0, pad = 0; i < arrays.length; i++) {
        const a = arrays[i];
        res.set(a, pad);
        pad += a.length;
    }
    return res;
}
// For runtime check if class implements interface
class Hash {
    // Safe version that clones internal state
    clone() {
        return this._cloneInto();
    }
}
exports.Hash = Hash;
const toStr = {}.toString;
function checkOpts(defaults, opts) {
    if (opts !== undefined && toStr.call(opts) !== '[object Object]')
        throw new Error('Options should be object or undefined');
    const merged = Object.assign(defaults, opts);
    return merged;
}
function wrapConstructor(hashCons) {
    const hashC = (msg) => hashCons().update(toBytes(msg)).digest();
    const tmp = hashCons();
    hashC.outputLen = tmp.outputLen;
    hashC.blockLen = tmp.blockLen;
    hashC.create = () => hashCons();
    return hashC;
}
function wrapConstructorWithOpts(hashCons) {
    const hashC = (msg, opts) => hashCons(opts).update(toBytes(msg)).digest();
    const tmp = hashCons({});
    hashC.outputLen = tmp.outputLen;
    hashC.blockLen = tmp.blockLen;
    hashC.create = (opts) => hashCons(opts);
    return hashC;
}
function wrapXOFConstructorWithOpts(hashCons) {
    const hashC = (msg, opts) => hashCons(opts).update(toBytes(msg)).digest();
    const tmp = hashCons({});
    hashC.outputLen = tmp.outputLen;
    hashC.blockLen = tmp.blockLen;
    hashC.create = (opts) => hashCons(opts);
    return hashC;
}
/**
 * Secure PRNG. Uses `crypto.getRandomValues`, which defers to OS.
 */
function randomBytes(bytesLength = 32) {
    if (crypto_1.crypto && typeof crypto_1.crypto.getRandomValues === 'function') {
        return crypto_1.crypto.getRandomValues(new Uint8Array(bytesLength));
    }
    // Legacy Node.js compatibility
    if (crypto_1.crypto && typeof crypto_1.crypto.randomBytes === 'function') {
        return crypto_1.crypto.randomBytes(bytesLength);
    }
    throw new Error('crypto.getRandomValues must be defined');
}
//# sourceMappingURL=utils.js.map
  });
  define("vendor/noble/_blake.js", function (require, module, exports) {
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.BLAKE = exports.SIGMA = void 0;
const _assert_js_1 = require("vendor/noble/_assert.js");
const utils_js_1 = require("vendor/noble/utils.js");
// Blake is based on ChaCha permutation.
// For BLAKE2b, the two extra permutations for rounds 10 and 11 are SIGMA[10..11] = SIGMA[0..1].
// prettier-ignore
exports.SIGMA = new Uint8Array([
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3,
    11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4,
    7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8,
    9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13,
    2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9,
    12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11,
    13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10,
    6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5,
    10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0,
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3,
]);
class BLAKE extends utils_js_1.Hash {
    constructor(blockLen, outputLen, opts = {}, keyLen, saltLen, persLen) {
        super();
        this.blockLen = blockLen;
        this.outputLen = outputLen;
        this.length = 0;
        this.pos = 0;
        this.finished = false;
        this.destroyed = false;
        (0, _assert_js_1.number)(blockLen);
        (0, _assert_js_1.number)(outputLen);
        (0, _assert_js_1.number)(keyLen);
        if (outputLen < 0 || outputLen > keyLen)
            throw new Error('outputLen bigger than keyLen');
        if (opts.key !== undefined && (opts.key.length < 1 || opts.key.length > keyLen))
            throw new Error(`key must be up 1..${keyLen} byte long or undefined`);
        if (opts.salt !== undefined && opts.salt.length !== saltLen)
            throw new Error(`salt must be ${saltLen} byte long or undefined`);
        if (opts.personalization !== undefined && opts.personalization.length !== persLen)
            throw new Error(`personalization must be ${persLen} byte long or undefined`);
        this.buffer32 = (0, utils_js_1.u32)((this.buffer = new Uint8Array(blockLen)));
    }
    update(data) {
        (0, _assert_js_1.exists)(this);
        // Main difference with other hashes: there is flag for last block,
        // so we cannot process current block before we know that there
        // is the next one. This significantly complicates logic and reduces ability
        // to do zero-copy processing
        const { blockLen, buffer, buffer32 } = this;
        data = (0, utils_js_1.toBytes)(data);
        const len = data.length;
        const offset = data.byteOffset;
        const buf = data.buffer;
        for (let pos = 0; pos < len;) {
            // If buffer is full and we still have input (don't process last block, same as blake2s)
            if (this.pos === blockLen) {
                if (!utils_js_1.isLE)
                    (0, utils_js_1.byteSwap32)(buffer32);
                this.compress(buffer32, 0, false);
                if (!utils_js_1.isLE)
                    (0, utils_js_1.byteSwap32)(buffer32);
                this.pos = 0;
            }
            const take = Math.min(blockLen - this.pos, len - pos);
            const dataOffset = offset + pos;
            // full block && aligned to 4 bytes && not last in input
            if (take === blockLen && !(dataOffset % 4) && pos + take < len) {
                const data32 = new Uint32Array(buf, dataOffset, Math.floor((len - pos) / 4));
                if (!utils_js_1.isLE)
                    (0, utils_js_1.byteSwap32)(data32);
                for (let pos32 = 0; pos + blockLen < len; pos32 += buffer32.length, pos += blockLen) {
                    this.length += blockLen;
                    this.compress(data32, pos32, false);
                }
                if (!utils_js_1.isLE)
                    (0, utils_js_1.byteSwap32)(data32);
                continue;
            }
            buffer.set(data.subarray(pos, pos + take), this.pos);
            this.pos += take;
            this.length += take;
            pos += take;
        }
        return this;
    }
    digestInto(out) {
        (0, _assert_js_1.exists)(this);
        (0, _assert_js_1.output)(out, this);
        const { pos, buffer32 } = this;
        this.finished = true;
        // Padding
        this.buffer.subarray(pos).fill(0);
        if (!utils_js_1.isLE)
            (0, utils_js_1.byteSwap32)(buffer32);
        this.compress(buffer32, 0, true);
        if (!utils_js_1.isLE)
            (0, utils_js_1.byteSwap32)(buffer32);
        const out32 = (0, utils_js_1.u32)(out);
        this.get().forEach((v, i) => (out32[i] = (0, utils_js_1.byteSwapIfBE)(v)));
    }
    digest() {
        const { buffer, outputLen } = this;
        this.digestInto(buffer);
        const res = buffer.slice(0, outputLen);
        this.destroy();
        return res;
    }
    _cloneInto(to) {
        const { buffer, length, finished, destroyed, outputLen, pos } = this;
        to || (to = new this.constructor({ dkLen: outputLen }));
        to.set(...this.get());
        to.length = length;
        to.finished = finished;
        to.destroyed = destroyed;
        to.outputLen = outputLen;
        to.buffer.set(buffer);
        to.pos = pos;
        return to;
    }
}
exports.BLAKE = BLAKE;
//# sourceMappingURL=_blake.js.map
  });
  define("vendor/noble/blake2s.js", function (require, module, exports) {
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.blake2s = exports.BLAKE2s = exports.B2S_IV = void 0;
exports.compress = compress;
const _blake_js_1 = require("vendor/noble/_blake.js");
const _u64_js_1 = require("vendor/noble/_u64.js");
const utils_js_1 = require("vendor/noble/utils.js");
// Initial state: same as SHA256
// first 32 bits of the fractional parts of the square roots of the first 8 primes 2..19
// prettier-ignore
exports.B2S_IV = new Uint32Array([
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
]);
// Mixing function G splitted in two halfs
function G1s(a, b, c, d, x) {
    a = (a + b + x) | 0;
    d = (0, utils_js_1.rotr)(d ^ a, 16);
    c = (c + d) | 0;
    b = (0, utils_js_1.rotr)(b ^ c, 12);
    return { a, b, c, d };
}
function G2s(a, b, c, d, x) {
    a = (a + b + x) | 0;
    d = (0, utils_js_1.rotr)(d ^ a, 8);
    c = (c + d) | 0;
    b = (0, utils_js_1.rotr)(b ^ c, 7);
    return { a, b, c, d };
}
// prettier-ignore
function compress(s, offset, msg, rounds, v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15) {
    let j = 0;
    for (let i = 0; i < rounds; i++) {
        ({ a: v0, b: v4, c: v8, d: v12 } = G1s(v0, v4, v8, v12, msg[offset + s[j++]]));
        ({ a: v0, b: v4, c: v8, d: v12 } = G2s(v0, v4, v8, v12, msg[offset + s[j++]]));
        ({ a: v1, b: v5, c: v9, d: v13 } = G1s(v1, v5, v9, v13, msg[offset + s[j++]]));
        ({ a: v1, b: v5, c: v9, d: v13 } = G2s(v1, v5, v9, v13, msg[offset + s[j++]]));
        ({ a: v2, b: v6, c: v10, d: v14 } = G1s(v2, v6, v10, v14, msg[offset + s[j++]]));
        ({ a: v2, b: v6, c: v10, d: v14 } = G2s(v2, v6, v10, v14, msg[offset + s[j++]]));
        ({ a: v3, b: v7, c: v11, d: v15 } = G1s(v3, v7, v11, v15, msg[offset + s[j++]]));
        ({ a: v3, b: v7, c: v11, d: v15 } = G2s(v3, v7, v11, v15, msg[offset + s[j++]]));
        ({ a: v0, b: v5, c: v10, d: v15 } = G1s(v0, v5, v10, v15, msg[offset + s[j++]]));
        ({ a: v0, b: v5, c: v10, d: v15 } = G2s(v0, v5, v10, v15, msg[offset + s[j++]]));
        ({ a: v1, b: v6, c: v11, d: v12 } = G1s(v1, v6, v11, v12, msg[offset + s[j++]]));
        ({ a: v1, b: v6, c: v11, d: v12 } = G2s(v1, v6, v11, v12, msg[offset + s[j++]]));
        ({ a: v2, b: v7, c: v8, d: v13 } = G1s(v2, v7, v8, v13, msg[offset + s[j++]]));
        ({ a: v2, b: v7, c: v8, d: v13 } = G2s(v2, v7, v8, v13, msg[offset + s[j++]]));
        ({ a: v3, b: v4, c: v9, d: v14 } = G1s(v3, v4, v9, v14, msg[offset + s[j++]]));
        ({ a: v3, b: v4, c: v9, d: v14 } = G2s(v3, v4, v9, v14, msg[offset + s[j++]]));
    }
    return { v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15 };
}
class BLAKE2s extends _blake_js_1.BLAKE {
    constructor(opts = {}) {
        super(64, opts.dkLen === undefined ? 32 : opts.dkLen, opts, 32, 8, 8);
        // Internal state, same as SHA-256
        this.v0 = exports.B2S_IV[0] | 0;
        this.v1 = exports.B2S_IV[1] | 0;
        this.v2 = exports.B2S_IV[2] | 0;
        this.v3 = exports.B2S_IV[3] | 0;
        this.v4 = exports.B2S_IV[4] | 0;
        this.v5 = exports.B2S_IV[5] | 0;
        this.v6 = exports.B2S_IV[6] | 0;
        this.v7 = exports.B2S_IV[7] | 0;
        const keyLength = opts.key ? opts.key.length : 0;
        this.v0 ^= this.outputLen | (keyLength << 8) | (0x01 << 16) | (0x01 << 24);
        if (opts.salt) {
            const salt = (0, utils_js_1.u32)((0, utils_js_1.toBytes)(opts.salt));
            this.v4 ^= (0, utils_js_1.byteSwapIfBE)(salt[0]);
            this.v5 ^= (0, utils_js_1.byteSwapIfBE)(salt[1]);
        }
        if (opts.personalization) {
            const pers = (0, utils_js_1.u32)((0, utils_js_1.toBytes)(opts.personalization));
            this.v6 ^= (0, utils_js_1.byteSwapIfBE)(pers[0]);
            this.v7 ^= (0, utils_js_1.byteSwapIfBE)(pers[1]);
        }
        if (opts.key) {
            // Pad to blockLen and update
            const tmp = new Uint8Array(this.blockLen);
            tmp.set((0, utils_js_1.toBytes)(opts.key));
            this.update(tmp);
        }
    }
    get() {
        const { v0, v1, v2, v3, v4, v5, v6, v7 } = this;
        return [v0, v1, v2, v3, v4, v5, v6, v7];
    }
    // prettier-ignore
    set(v0, v1, v2, v3, v4, v5, v6, v7) {
        this.v0 = v0 | 0;
        this.v1 = v1 | 0;
        this.v2 = v2 | 0;
        this.v3 = v3 | 0;
        this.v4 = v4 | 0;
        this.v5 = v5 | 0;
        this.v6 = v6 | 0;
        this.v7 = v7 | 0;
    }
    compress(msg, offset, isLast) {
        const { h, l } = (0, _u64_js_1.fromBig)(BigInt(this.length));
        // prettier-ignore
        const { v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15 } = compress(_blake_js_1.SIGMA, offset, msg, 10, this.v0, this.v1, this.v2, this.v3, this.v4, this.v5, this.v6, this.v7, exports.B2S_IV[0], exports.B2S_IV[1], exports.B2S_IV[2], exports.B2S_IV[3], l ^ exports.B2S_IV[4], h ^ exports.B2S_IV[5], isLast ? ~exports.B2S_IV[6] : exports.B2S_IV[6], exports.B2S_IV[7]);
        this.v0 ^= v0 ^ v8;
        this.v1 ^= v1 ^ v9;
        this.v2 ^= v2 ^ v10;
        this.v3 ^= v3 ^ v11;
        this.v4 ^= v4 ^ v12;
        this.v5 ^= v5 ^ v13;
        this.v6 ^= v6 ^ v14;
        this.v7 ^= v7 ^ v15;
    }
    destroy() {
        this.destroyed = true;
        this.buffer32.fill(0);
        this.set(0, 0, 0, 0, 0, 0, 0, 0);
    }
}
exports.BLAKE2s = BLAKE2s;
/**
 * BLAKE2s - optimized for 32-bit platforms. JS doesn't have uint64, so it's faster than BLAKE2b.
 * @param msg - message that would be hashed
 * @param opts - dkLen, key, salt, personalization
 */
exports.blake2s = (0, utils_js_1.wrapConstructorWithOpts)((opts) => new BLAKE2s(opts));
//# sourceMappingURL=blake2s.js.map
  });
  define("vendor/noble/blake3.js", function (require, module, exports) {
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.blake3 = exports.BLAKE3 = void 0;
const _assert_js_1 = require("vendor/noble/_assert.js");
const _u64_js_1 = require("vendor/noble/_u64.js");
const _blake_js_1 = require("vendor/noble/_blake.js");
const blake2s_js_1 = require("vendor/noble/blake2s.js");
const utils_js_1 = require("vendor/noble/utils.js");
const SIGMA = /* @__PURE__ */ (() => {
    const Id = Array.from({ length: 16 }, (_, i) => i);
    const permute = (arr) => [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8].map((i) => arr[i]);
    const res = [];
    for (let i = 0, v = Id; i < 7; i++, v = permute(v))
        res.push(...v);
    return Uint8Array.from(res);
})();
// Why is this so slow? It should be 6x faster than blake2b.
// - There is only 30% reduction in number of rounds from blake2s
// - This function uses tree mode to achive parallelisation via SIMD and threading,
//   however in JS we don't have threads and SIMD, so we get only overhead from tree structure
// - It is possible to speed it up via Web Workers, hovewer it will make code singnificantly more
//   complicated, which we are trying to avoid, since this library is intended to be used
//   for cryptographic purposes. Also, parallelization happens only on chunk level (1024 bytes),
//   which won't really benefit small inputs.
class BLAKE3 extends _blake_js_1.BLAKE {
    constructor(opts = {}, flags = 0) {
        super(64, opts.dkLen === undefined ? 32 : opts.dkLen, {}, Number.MAX_SAFE_INTEGER, 0, 0);
        this.flags = 0 | 0;
        this.chunkPos = 0; // Position of current block in chunk
        this.chunksDone = 0; // How many chunks we already have
        this.stack = [];
        // Output
        this.posOut = 0;
        this.bufferOut32 = new Uint32Array(16);
        this.chunkOut = 0; // index of output chunk
        this.enableXOF = true;
        this.outputLen = opts.dkLen === undefined ? 32 : opts.dkLen;
        (0, _assert_js_1.number)(this.outputLen);
        if (opts.key !== undefined && opts.context !== undefined)
            throw new Error('Blake3: only key or context can be specified at same time');
        else if (opts.key !== undefined) {
            const key = (0, utils_js_1.toBytes)(opts.key).slice();
            if (key.length !== 32)
                throw new Error('Blake3: key should be 32 byte');
            this.IV = (0, utils_js_1.u32)(key);
            if (!utils_js_1.isLE)
                (0, utils_js_1.byteSwap32)(this.IV);
            this.flags = flags | 16 /* B3_Flags.KEYED_HASH */;
        }
        else if (opts.context !== undefined) {
            const context_key = new BLAKE3({ dkLen: 32 }, 32 /* B3_Flags.DERIVE_KEY_CONTEXT */)
                .update(opts.context)
                .digest();
            this.IV = (0, utils_js_1.u32)(context_key);
            if (!utils_js_1.isLE)
                (0, utils_js_1.byteSwap32)(this.IV);
            this.flags = flags | 64 /* B3_Flags.DERIVE_KEY_MATERIAL */;
        }
        else {
            this.IV = blake2s_js_1.B2S_IV.slice();
            this.flags = flags;
        }
        this.state = this.IV.slice();
        this.bufferOut = (0, utils_js_1.u8)(this.bufferOut32);
    }
    // Unused
    get() {
        return [];
    }
    set() { }
    b2Compress(counter, flags, buf, bufPos = 0) {
        const { state: s, pos } = this;
        const { h, l } = (0, _u64_js_1.fromBig)(BigInt(counter), true);
        // prettier-ignore
        const { v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15 } = (0, blake2s_js_1.compress)(SIGMA, bufPos, buf, 7, s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7], blake2s_js_1.B2S_IV[0], blake2s_js_1.B2S_IV[1], blake2s_js_1.B2S_IV[2], blake2s_js_1.B2S_IV[3], h, l, pos, flags);
        s[0] = v0 ^ v8;
        s[1] = v1 ^ v9;
        s[2] = v2 ^ v10;
        s[3] = v3 ^ v11;
        s[4] = v4 ^ v12;
        s[5] = v5 ^ v13;
        s[6] = v6 ^ v14;
        s[7] = v7 ^ v15;
    }
    compress(buf, bufPos = 0, isLast = false) {
        // Compress last block
        let flags = this.flags;
        if (!this.chunkPos)
            flags |= 1 /* B3_Flags.CHUNK_START */;
        if (this.chunkPos === 15 || isLast)
            flags |= 2 /* B3_Flags.CHUNK_END */;
        if (!isLast)
            this.pos = this.blockLen;
        this.b2Compress(this.chunksDone, flags, buf, bufPos);
        this.chunkPos += 1;
        // If current block is last in chunk (16 blocks), then compress chunks
        if (this.chunkPos === 16 || isLast) {
            let chunk = this.state;
            this.state = this.IV.slice();
            // If not the last one, compress only when there are trailing zeros in chunk counter
            // chunks used as binary tree where current stack is path. Zero means current leaf is finished and can be compressed.
            // 1 (001) - leaf not finished (just push current chunk to stack)
            // 2 (010) - leaf finished at depth=1 (merge with last elm on stack and push back)
            // 3 (011) - last leaf not finished
            // 4 (100) - leafs finished at depth=1 and depth=2
            for (let last, chunks = this.chunksDone + 1; isLast || !(chunks & 1); chunks >>= 1) {
                if (!(last = this.stack.pop()))
                    break;
                this.buffer32.set(last, 0);
                this.buffer32.set(chunk, 8);
                this.pos = this.blockLen;
                this.b2Compress(0, this.flags | 4 /* B3_Flags.PARENT */, this.buffer32, 0);
                chunk = this.state;
                this.state = this.IV.slice();
            }
            this.chunksDone++;
            this.chunkPos = 0;
            this.stack.push(chunk);
        }
        this.pos = 0;
    }
    _cloneInto(to) {
        to = super._cloneInto(to);
        const { IV, flags, state, chunkPos, posOut, chunkOut, stack, chunksDone } = this;
        to.state.set(state.slice());
        to.stack = stack.map((i) => Uint32Array.from(i));
        to.IV.set(IV);
        to.flags = flags;
        to.chunkPos = chunkPos;
        to.chunksDone = chunksDone;
        to.posOut = posOut;
        to.chunkOut = chunkOut;
        to.enableXOF = this.enableXOF;
        to.bufferOut32.set(this.bufferOut32);
        return to;
    }
    destroy() {
        this.destroyed = true;
        this.state.fill(0);
        this.buffer32.fill(0);
        this.IV.fill(0);
        this.bufferOut32.fill(0);
        for (let i of this.stack)
            i.fill(0);
    }
    // Same as b2Compress, but doesn't modify state and returns 16 u32 array (instead of 8)
    b2CompressOut() {
        const { state: s, pos, flags, buffer32, bufferOut32: out32 } = this;
        const { h, l } = (0, _u64_js_1.fromBig)(BigInt(this.chunkOut++));
        if (!utils_js_1.isLE)
            (0, utils_js_1.byteSwap32)(buffer32);
        // prettier-ignore
        const { v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15 } = (0, blake2s_js_1.compress)(SIGMA, 0, buffer32, 7, s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7], blake2s_js_1.B2S_IV[0], blake2s_js_1.B2S_IV[1], blake2s_js_1.B2S_IV[2], blake2s_js_1.B2S_IV[3], l, h, pos, flags);
        out32[0] = v0 ^ v8;
        out32[1] = v1 ^ v9;
        out32[2] = v2 ^ v10;
        out32[3] = v3 ^ v11;
        out32[4] = v4 ^ v12;
        out32[5] = v5 ^ v13;
        out32[6] = v6 ^ v14;
        out32[7] = v7 ^ v15;
        out32[8] = s[0] ^ v8;
        out32[9] = s[1] ^ v9;
        out32[10] = s[2] ^ v10;
        out32[11] = s[3] ^ v11;
        out32[12] = s[4] ^ v12;
        out32[13] = s[5] ^ v13;
        out32[14] = s[6] ^ v14;
        out32[15] = s[7] ^ v15;
        if (!utils_js_1.isLE) {
            (0, utils_js_1.byteSwap32)(buffer32);
            (0, utils_js_1.byteSwap32)(out32);
        }
        this.posOut = 0;
    }
    finish() {
        if (this.finished)
            return;
        this.finished = true;
        // Padding
        this.buffer.fill(0, this.pos);
        // Process last chunk
        let flags = this.flags | 8 /* B3_Flags.ROOT */;
        if (this.stack.length) {
            flags |= 4 /* B3_Flags.PARENT */;
            if (!utils_js_1.isLE)
                (0, utils_js_1.byteSwap32)(this.buffer32);
            this.compress(this.buffer32, 0, true);
            if (!utils_js_1.isLE)
                (0, utils_js_1.byteSwap32)(this.buffer32);
            this.chunksDone = 0;
            this.pos = this.blockLen;
        }
        else {
            flags |= (!this.chunkPos ? 1 /* B3_Flags.CHUNK_START */ : 0) | 2 /* B3_Flags.CHUNK_END */;
        }
        this.flags = flags;
        this.b2CompressOut();
    }
    writeInto(out) {
        (0, _assert_js_1.exists)(this, false);
        (0, _assert_js_1.bytes)(out);
        this.finish();
        const { blockLen, bufferOut } = this;
        for (let pos = 0, len = out.length; pos < len;) {
            if (this.posOut >= blockLen)
                this.b2CompressOut();
            const take = Math.min(blockLen - this.posOut, len - pos);
            out.set(bufferOut.subarray(this.posOut, this.posOut + take), pos);
            this.posOut += take;
            pos += take;
        }
        return out;
    }
    xofInto(out) {
        if (!this.enableXOF)
            throw new Error('XOF is not possible after digest call');
        return this.writeInto(out);
    }
    xof(bytes) {
        (0, _assert_js_1.number)(bytes);
        return this.xofInto(new Uint8Array(bytes));
    }
    digestInto(out) {
        (0, _assert_js_1.output)(out, this);
        if (this.finished)
            throw new Error('digest() was already called');
        this.enableXOF = false;
        this.writeInto(out);
        this.destroy();
        return out;
    }
    digest() {
        return this.digestInto(new Uint8Array(this.outputLen));
    }
}
exports.BLAKE3 = BLAKE3;
/**
 * BLAKE3 hash function.
 * @param msg - message that would be hashed
 * @param opts - dkLen, key, context
 */
exports.blake3 = (0, utils_js_1.wrapXOFConstructorWithOpts)((opts) => new BLAKE3(opts));
//# sourceMappingURL=blake3.js.map
  });
  define("check.js", function (require, module, exports) {
/**
 * HashSeal instruct document check — FULL canonical mode (digest only).
 * Mirrors hashseal-core instruct algorithm. Zero npm dependencies.
 *
 * Copyright (c) 2026 MonkeyKing.dev
 */

"use strict";

const { blake3 } = require("vendor/noble/blake3.js");

const SEAL_FIELD = "hashseal";
const SIG_FIELD = "hashseal_sig";
const KEY_ID_FIELD = "hashseal_key_id";

const RESERVED = new Set([SEAL_FIELD, SIG_FIELD, KEY_ID_FIELD]);

/**
 * @typedef {Object} CheckResult
 * @property {boolean} ok
 * @property {"valid"|"mismatch"|"missing_seal"|"invalid_format"} status
 * @property {string|null} algorithm
 * @property {string|null} expected  qualified digest e.g. blake3:hex
 * @property {string|null} actual
 * @property {string|null} message
 */

/**
 * Check a sealed instruct markdown document (text in memory).
 * @param {string} text
 * @param {{ field?: string }} [opts]
 * @returns {CheckResult}
 */
function checkDocumentText(text, opts) {
  const field = (opts && opts.field) || SEAL_FIELD;
  const doc = parseDocument(text);
  if (!doc.hadFrontMatter) {
    const actual = computeDigest(doc);
    return {
      ok: false,
      status: "missing_seal",
      algorithm: "blake3",
      expected: null,
      actual: actual.qualified,
      message: "missing hashseal field",
    };
  }
  const sealRaw = extractReservedField(doc.fmLines, field);
  if (sealRaw == null) {
    const actual = computeDigest(doc);
    return {
      ok: false,
      status: "missing_seal",
      algorithm: "blake3",
      expected: null,
      actual: actual.qualified,
      message: "missing hashseal field",
    };
  }
  const expected = parseDigest(sealRaw);
  if (!expected) {
    return {
      ok: false,
      status: "invalid_format",
      algorithm: null,
      expected: null,
      actual: null,
      message: `invalid digest format: ${sealRaw}`,
    };
  }
  // Use algorithm from seal for hashing (blake3 only implemented here)
  if (expected.algorithm !== "blake3") {
    return {
      ok: false,
      status: "invalid_format",
      algorithm: expected.algorithm,
      expected: expected.qualified,
      actual: null,
      message: `unsupported algorithm: ${expected.algorithm}`,
    };
  }
  const actual = computeDigest(doc);
  if (actual.hex !== expected.hex || actual.algorithm !== expected.algorithm) {
    return {
      ok: false,
      status: "mismatch",
      algorithm: expected.algorithm,
      expected: expected.qualified,
      actual: actual.qualified,
      message: null,
    };
  }
  return {
    ok: true,
    status: "valid",
    algorithm: actual.algorithm,
    expected: expected.qualified,
    actual: actual.qualified,
    message: null,
  };
}

/**
 * Blake3 hex digest of UTF-8 bytes (no algorithm prefix).
 * @param {string|Uint8Array} data
 * @returns {string} lowercase hex
 */
function blake3Hex(data) {
  const bytes =
    typeof data === "string" ? new TextEncoder().encode(data) : data;
  return Buffer.from(blake3(bytes)).toString("hex");
}

/**
 * @param {string|Uint8Array} data
 * @returns {{ algorithm: string, hex: string, qualified: string }}
 */
function blake3Digest(data) {
  const hex = blake3Hex(data);
  return { algorithm: "blake3", hex, qualified: `blake3:${hex}` };
}

// --- parse / canonical (mirrors hashseal-core instruct.rs) ---

function stripBom(s) {
  return s.charCodeAt(0) === 0xfeff ? s.slice(1) : s;
}

function normalizeLf(s) {
  return s.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

function parseDocument(text) {
  text = stripBom(text);
  if (text.startsWith("---\n") || text.startsWith("---\r\n")) {
    const afterOpen = text.startsWith("---\r\n") ? text.slice(5) : text.slice(4);
    let search = afterOpen;
    let offset = 0;
    while (true) {
      const idx = search.indexOf("\n---");
      if (idx < 0) break;
      const after = search.slice(idx + 1);
      const rest = after.slice(3);
      const closed =
        rest.length === 0 ||
        rest.startsWith("\n") ||
        rest.startsWith("\r\n") ||
        rest.startsWith("\r");
      if (closed) {
        const fmBlock = afterOpen.slice(0, offset + idx);
        let bodyStart = idx + 1 + 3;
        let body = afterOpen.slice(bodyStart);
        if (body.startsWith("\r\n")) body = body.slice(2);
        else if (body.startsWith("\n")) body = body.slice(1);
        else if (body.startsWith("\r")) body = body.slice(1);
        const fmLines = normalizeLf(fmBlock).split("\n");
        // split("\n") on trailing empty keeps last empty; LF block without trailing \n is fine
        return {
          fmLines,
          hadFrontMatter: true,
          bodyRaw: body,
        };
      }
      offset += idx + 1;
      search = search.slice(idx + 1);
    }
  }
  return { fmLines: [], hadFrontMatter: false, bodyRaw: text };
}

function isReservedKey(key) {
  return RESERVED.has(key);
}

function forEachFmEntry(lines, f) {
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    const trimmed = line.trim();
    if (trimmed === "" || trimmed.startsWith("#")) {
      i += 1;
      continue;
    }
    if (line.startsWith(" ") || line.startsWith("\t")) {
      i += 1;
      continue;
    }
    const colon = trimmed.indexOf(":");
    if (colon >= 0) {
      const key = trimmed.slice(0, colon).trim();
      const rest = trimmed.slice(colon + 1).trim();
      if (isReservedKey(key)) {
        i += 1;
        while (i < lines.length) {
          const L = lines[i];
          if (L.startsWith(" ") || L.startsWith("\t")) {
            i += 1;
            continue;
          }
          if (L.trim() === "") {
            if (
              i + 1 < lines.length &&
              (lines[i + 1].startsWith(" ") || lines[i + 1].startsWith("\t"))
            ) {
              i += 1;
              continue;
            }
            break;
          }
          break;
        }
        continue;
      }
      if (rest === "|" || rest === ">" || rest === "|-" || rest === ">-") {
        let val = "";
        i += 1;
        while (
          i < lines.length &&
          (lines[i].startsWith(" ") || lines[i].startsWith("\t"))
        ) {
          if (val !== "") val += "\n";
          val += lines[i].trimStart();
          i += 1;
        }
        f(key, val);
        continue;
      }
      const val = rest.replace(/^"|"$/g, "");
      f(key, val);
    }
    i += 1;
  }
}

function fmMap(lines) {
  /** @type {Map<string, string>} */
  const map = new Map();
  forEachFmEntry(lines, (k, v) => {
    map.set(k, v);
  });
  return map;
}

function extractReservedField(lines, field) {
  let i = 0;
  while (i < lines.length) {
    const trimmed = lines[i].trim();
    const colon = trimmed.indexOf(":");
    if (colon >= 0) {
      const k = trimmed.slice(0, colon).trim();
      if (k === field) {
        const rest = trimmed.slice(colon + 1).trim();
        if (rest === "|" || rest === ">" || rest === "|-" || rest === ">-") {
          let val = "";
          i += 1;
          while (i < lines.length) {
            const L = lines[i];
            const empty = L.trim() === "";
            const indented = L.startsWith(" ") || L.startsWith("\t");
            if (
              indented ||
              (empty &&
                i + 1 < lines.length &&
                (lines[i + 1].startsWith(" ") || lines[i + 1].startsWith("\t")))
            ) {
              if (empty) {
                val += "\n";
                i += 1;
                continue;
              }
              if (val !== "") val += "\n";
              val += L.trimStart();
              i += 1;
              continue;
            }
            break;
          }
          return val;
        }
        return rest.replace(/^"|"$/g, "");
      }
    }
    i += 1;
  }
  return null;
}

function canonicalFmString(map) {
  const keys = Array.from(map.keys()).sort();
  let s = "";
  for (const k of keys) {
    const v = map.get(k);
    s += k;
    s += ": ";
    if (v === "" || v.includes(":") || v.includes("#") || v.includes(" ")) {
      s += '"';
      s += v.replace(/"/g, '\\"');
      s += '"';
    } else {
      s += v;
    }
    s += "\n";
  }
  return s;
}

function hashPayload(doc) {
  const bodyLf = normalizeLf(doc.bodyRaw);
  const map = fmMap(doc.fmLines);
  if (map.size === 0) {
    return new TextEncoder().encode(bodyLf);
  }
  let payload = canonicalFmString(map);
  payload += "\n";
  payload += bodyLf;
  return new TextEncoder().encode(payload);
}

function computeDigest(doc) {
  return blake3Digest(hashPayload(doc));
}

function parseDigest(raw) {
  const s = String(raw).trim().replace(/^"|"$/g, "");
  const idx = s.indexOf(":");
  if (idx < 0) return null;
  const algorithm = s.slice(0, idx).toLowerCase();
  const hex = s.slice(idx + 1).trim().toLowerCase();
  if (!hex || !/^[0-9a-f]+$/.test(hex)) return null;
  return { algorithm, hex, qualified: `${algorithm}:${hex}` };
}

module.exports = {
  checkDocumentText,
  blake3Hex,
  blake3Digest,
  SEAL_FIELD,
  // exported for unit tests / debugging
  _internal: {
    parseDocument,
    hashPayload,
    computeDigest,
    normalizeLf,
  },
};

  });
  define("tree.js", function (require, module, exports) {
/**
 * In-memory tree verify — mirrors hashseal-core tree hash + verify policy.
 * Zero npm dependencies. Used for multi-lang tree-v1 vectors without a filesystem walk.
 *
 * Copyright (c) 2026 MonkeyKing.dev
 */

"use strict";

const { blake3Digest } = require("check.js");

const DEFAULT_TEXT_EXTENSIONS = new Set([
  "md",
  "txt",
  "toml",
  "yml",
  "yaml",
  "json",
  "rs",
  "java",
  "go",
  "py",
  "js",
  "ts",
  "tsx",
  "jsx",
  "css",
  "html",
  "xml",
  "sh",
  "ps1",
  "c",
  "h",
  "cpp",
  "cs",
  "rb",
  "svg",
]);

function normalizeLf(s) {
  return s.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

function extOf(path) {
  const i = path.lastIndexOf(".");
  if (i < 0) return "";
  return path.slice(i + 1).toLowerCase();
}

/**
 * Hash one path+content with core tree policy.
 * @param {string} path posix relative path
 * @param {string} content file body (UTF-8 string; binary as latin1 if needed)
 * @param {{ lineEndingsLfText?: boolean, textExtensions?: Set<string>|string[] }} [opts]
 * @returns {{ digest: string, size: number, qualified: string, hex: string }}
 */
function hashTreeFileContent(path, content, opts) {
  const lfText = !opts || opts.lineEndingsLfText !== false;
  const textExts =
    opts && opts.textExtensions
      ? opts.textExtensions instanceof Set
        ? opts.textExtensions
        : new Set(opts.textExtensions)
      : DEFAULT_TEXT_EXTENSIONS;
  const size =
    typeof Buffer !== "undefined"
      ? Buffer.byteLength(content, "utf8")
      : new TextEncoder().encode(content).length;
  let data = content;
  if (lfText && textExts.has(extOf(path))) {
    data = normalizeLf(content.replace(/^\uFEFF/, ""));
  }
  const d = blake3Digest(data);
  return {
    digest: d.qualified,
    qualified: d.qualified,
    hex: d.hex,
    size,
  };
}

/**
 * Verify in-memory files against ledger entries (same findings as hashseal-core verify_tree).
 * @param {Record<string, string>} files path → content
 * @param {Array<{ path: string, digest: string, size?: number }>} ledgerEntries
 * @param {{ lineEndingsLfText?: boolean, textExtensions?: Set<string>|string[] }} [opts]
 * @returns {{ ok: boolean, checked: number, findings: Array<{path:string,status:string,expected:string|null,actual:string|null}> }}
 */
function verifyTreeInMemory(files, ledgerEntries, opts) {
  const current = new Map();
  const paths = Object.keys(files || {}).sort();
  for (const p of paths) {
    const h = hashTreeFileContent(p, files[p], opts);
    current.set(p, h.qualified);
  }

  const findings = [];
  const expectedPaths = new Set();
  const entries = ledgerEntries || [];

  for (const e of entries) {
    expectedPaths.add(e.path);
    const actual = current.get(e.path);
    if (actual === undefined) {
      findings.push({
        path: e.path,
        status: "removed",
        expected: e.digest,
        actual: null,
      });
    } else if (actual !== e.digest) {
      findings.push({
        path: e.path,
        status: "mismatch",
        expected: e.digest,
        actual,
      });
    }
  }

  for (const [path, digest] of current) {
    if (!expectedPaths.has(path)) {
      findings.push({
        path,
        status: "added",
        expected: null,
        actual: digest,
      });
    }
  }

  findings.sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
  return {
    ok: findings.length === 0,
    checked: entries.length,
    findings,
  };
}

module.exports = {
  hashTreeFileContent,
  verifyTreeInMemory,
  DEFAULT_TEXT_EXTENSIONS,
  normalizeLf,
};

  });
  var check = require("check.js");
  var tree = require("tree.js");
  var api = {
    checkDocumentText: check.checkDocumentText,
    blake3Hex: check.blake3Hex,
    blake3Digest: check.blake3Digest,
    SEAL_FIELD: check.SEAL_FIELD,
    hashTreeFileContent: tree.hashTreeFileContent,
    verifyTreeInMemory: tree.verifyTreeInMemory
  };
  if (typeof module !== "undefined" && module.exports) module.exports = api;
  global.HashsealVerify = api;
})(typeof globalThis !== "undefined" ? globalThis : typeof window !== "undefined" ? window : this);
