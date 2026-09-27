// Generate a password and type it — the AHK #DPW# family, as a script.
//
//   scripts/bindings.clx:
//     p         script C:/Users/snomi/CapsLockX/scripts/genpw.js dpw
//     space+[   script C:/Users/snomi/CapsLockX/scripts/genpw.js qpw
//
// This replaces clx-genpw.exe. Same output, no compiled binary: nothing to build
// per platform, nothing unsigned for an antivirus heuristic to find, and you can
// read it before trusting it. See lab/script-store.
//
// Two things the host provides, and they are the whole reason this can be a
// script at all:
//
//   clx.random(n)  bytes from the OS CSPRNG. NOT Math.random(), which is not
//                  cryptographically secure — AHK's clock-seeded Mersenne
//                  Twister was the specific defect worth fixing, and reaching
//                  for Math.random() here would reintroduce it.
//   { type: … }    a structured step. The host formats and escapes it, so a
//                  password containing a quote or a backslash cannot be mangled
//                  by this file getting escaping subtly wrong.

// Alphabets kept character-for-character from Modules/QuickInput.ahk, so output
// looks like what the muscle memory expects. 0 O I l are absent on purpose: they
// are the ambiguous ones.
const UPPER = "123456789ABCDEFGHJKLMNPQRSTUVWXYZ";
const LOWER = "123456789abcdefghijkmnopqrstuvwxyz";
const MIXED = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
const DIGITS = "0123456789";
const HEX_UP = "0123456789ABCDEF";
const HEX_LO = "0123456789abcdef";

// #SPW#'s set, minus the literal space and duplicated comma it shipped with
// (QuickInput.ahk:138). A space in a password is a support call waiting to happen.
const SYMBOLS =
  "!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";

const CONSONANTS = "xktnmwhrypbdsfg";
const VOWELS = "aeiou";

// Rejection sampling, not `byte % len`: the modulo would make the first
// `256 % len` characters of an alphabet marginally likelier. A small bias, and a
// free one to avoid.
function pick(set, n) {
  const limit = Math.floor(256 / set.length) * set.length;
  let out = "";
  while (out.length < n) {
    for (const byte of clx.random(n * 2)) {
      if (byte < limit) {
        out += set[byte % set.length];
        if (out.length === n) break;
      }
    }
  }
  return out;
}

function romaji(syllables) {
  let out = "";
  for (let i = 0; i < syllables; i++) out += pick(CONSONANTS, 1) + pick(VOWELS, 1);
  return out;
}

const profiles = {
  // One leading uppercase, so it satisfies "must contain a capital" rules.
  dpw: () => pick(UPPER, 1) + pick(LOWER, 6) + "_" + pick(LOWER, 7),
  // #PW# and #WPW# were character-for-character identical in the AHK.
  pw: () => pick(MIXED, 16),
  wpw: () => pick(MIXED, 16),
  npw: () => pick(DIGITS, 16),
  hex: () => pick(HEX_UP, 16),
  hexl: () => pick(HEX_LO, 16),
  qpw: () => [0, 1, 2, 3].map(() => pick(UPPER, 4)).join("-"),
  // Shaped like a UUID, not a real one — the AHK called it 偽UUID, fake, and that
  // honesty is worth keeping: no version or variant bits are set.
  uuid: () =>
    pick(HEX_LO, 8) +
    "-" +
    pick(HEX_LO, 4) +
    "-" +
    pick(HEX_LO, 4) +
    "-" +
    pick(HEX_LO, 4) +
    "-" +
    pick(HEX_LO, 12),
  spw: () => pick(SYMBOLS, 16),
  jpw: () => romaji(4) + "-" + romaji(4),
};

const name = clx.args[0] || "dpw";
const make = profiles[name];

// A wrong profile name is reported, not guessed at. Throwing reaches the user as
// a logged reason and types nothing — far better than quietly typing whatever a
// fallback would have produced into a password field.
if (!make) {
  throw new Error(
    "unknown profile " +
      JSON.stringify(name) +
      " — expected one of " +
      Object.keys(profiles).join(" "),
  );
}

[{ type: make() }];
