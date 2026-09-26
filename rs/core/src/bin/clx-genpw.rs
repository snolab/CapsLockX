//! Generate a password and type it — a clx plugin, in one program.
//!
//! ```text
//! clx plugin clx-genpw          # default profile
//! clx plugin clx-genpw qpw      # a specific one
//! ```
//!
//! The whole contract is "print effects to stdout", so this prints exactly one
//! `k "…"` line and exits. clx performs it; clx never learns that the string was
//! a password, and nothing is stored anywhere.
//!
//! It replaces the AHK hotstrings in `Modules/QuickInput.ahk` (`#DPW#` and
//! friends), keeping their alphabets and shapes so muscle memory carries over,
//! and fixing three things about them:
//!
//!   * **The RNG.** AHK v1's `Random` is a clock-seeded Mersenne Twister, so its
//!     output is predictable to anyone who knows roughly when it ran. This uses
//!     the OS CSPRNG.
//!   * **Modulo bias.** Rejection sampling, so every character of an alphabet is
//!     equally likely rather than the first few being slightly favoured.
//!   * **`#SPW#`'s alphabet**, which contains a literal space and a duplicated
//!     comma (`Modules/QuickInput.ahk:138`). A space in a password is a support
//!     call waiting to happen.
//!
//! Deliberately not ported: the `#DPW#` *trigger*. A hotstring types its trigger
//! into the app before erasing it, so `#DPW#` plus five backspaces leak into
//! incremental search and "user is typing" indicators. A key binding has no such
//! problem, and clx has no hotstring engine to reproduce it with anyway.

use std::io::Write as _;

/// Alphabets, kept character-for-character from the AHK originals so the output
/// looks like what the user is used to. `0 O I l` are absent from the
/// human-typeable sets on purpose: they are the ambiguous ones.
const UPPER_NOAMBIG: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZ";
const LOWER_NOAMBIG: &str = "123456789abcdefghijkmnopqrstuvwxyz";
const MIXED_NOAMBIG: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const HEX_UPPER: &str = "0123456789ABCDEF";
const HEX_LOWER: &str = "0123456789abcdef";

/// `#SPW#`'s symbol set, minus the space and the duplicate comma. Quote and
/// backslash stay in: they are legitimate password characters, and the effect
/// language can carry them once escaped — see `escape`.
const SYMBOLS: &str = "!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";

/// Consonant/vowel pairs, so the result is pronounceable by a Japanese speaker.
const ROMAJI_CONSONANTS: &str = "xktnmwhrypbdsfg";
const ROMAJI_VOWELS: &str = "aeiou";

const USAGE: &str = "usage: clx-genpw [profile]

profiles (default: dpw)
  dpw    Zg1y9xy_hcswt71                        two sections, unambiguous
  pw     yyCTCNYodECTLr2h                       16 mixed, unambiguous (alias: wpw)
  npw    7500331260229289                       16 digits
  hex    7618DB5EAC135893                       16 uppercase hex
  hexl   31a0c5dcc46f0ff6                       16 lowercase hex
  qpw    4428-UW4R-58YS-ALLR                    four groups, uppercase
  uuid   345103d0-9de1-d5c6-425f-867dfbf555ea   random, not RFC-versioned
  spw    KO?C[D_>!c$sQ-|7                       16 with symbols
  jpw    kotemuha-nyrasei                       pronounceable romaji
";

/// One uniformly random character of `alphabet`.
///
/// Rejection sampling rather than `byte % len`: the modulo would make the first
/// `256 % len` characters marginally likelier, which is a small bias but a free
/// one to avoid.
fn pick(alphabet: &[char]) -> char {
    let len = alphabet.len();
    assert!(len > 0 && len <= 256, "alphabet must be 1..=256 characters");
    let limit = (256 / len) * len; // largest multiple of len that fits in a byte
    loop {
        let mut byte = [0u8; 1];
        getrandom::getrandom(&mut byte).expect("the OS CSPRNG is unavailable");
        let value = byte[0] as usize;
        if value < limit {
            return alphabet[value % len];
        }
    }
}

fn draw(alphabet: &str, n: usize) -> String {
    let chars: Vec<char> = alphabet.chars().collect();
    (0..n).map(|_| pick(&chars)).collect()
}

fn romaji(syllables: usize) -> String {
    (0..syllables)
        .map(|_| format!("{}{}", draw(ROMAJI_CONSONANTS, 1), draw(ROMAJI_VOWELS, 1)))
        .collect()
}

fn generate(profile: &str) -> Option<String> {
    Some(match profile {
        // 1 leading uppercase so it satisfies "must contain a capital" rules.
        "dpw" => format!(
            "{}{}_{}",
            draw(UPPER_NOAMBIG, 1),
            draw(LOWER_NOAMBIG, 6),
            draw(LOWER_NOAMBIG, 7)
        ),
        // #PW# and #WPW# were character-for-character identical in the AHK.
        "pw" | "wpw" => draw(MIXED_NOAMBIG, 16),
        "npw" => draw(DIGITS, 16),
        "hex" => draw(HEX_UPPER, 16),
        "hexl" => draw(HEX_LOWER, 16),
        "qpw" => (0..4)
            .map(|_| draw(UPPER_NOAMBIG, 4))
            .collect::<Vec<_>>()
            .join("-"),
        // Shaped like a UUID; not a real one. The AHK called it 偽UUID — fake —
        // and that honesty is worth keeping: no version or variant bits are set.
        "uuid" => format!(
            "{}-{}-{}-{}-{}",
            draw(HEX_LOWER, 8),
            draw(HEX_LOWER, 4),
            draw(HEX_LOWER, 4),
            draw(HEX_LOWER, 4),
            draw(HEX_LOWER, 12)
        ),
        "spw" => draw(SYMBOLS, 16),
        "jpw" => format!("{}-{}", romaji(4), romaji(4)),
        _ => return None,
    })
}

/// Escape a payload for the effect language's `k "…"`.
///
/// `effects::parse` accepts `\\` and `\"` inside the quotes, and a password from
/// the symbol alphabet can contain both. Without this, `spw` would produce an
/// unparseable line — and an unparseable line used to be echoed to stderr, which
/// is precisely how a generated password would have ended up in a log.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(c),
        }
    }
    out
}

fn main() {
    let arg = std::env::args().nth(1);
    let profile = arg.as_deref().unwrap_or("dpw");

    if matches!(profile, "-h" | "--help" | "help") {
        print!("{USAGE}");
        return;
    }

    match generate(profile) {
        // One line, to stdout, and nowhere else. Never logged, never stored.
        Some(password) => println!("k \"{}\"", escape(&password)),
        None => {
            eprint!("clx-genpw: unknown profile {profile:?}\n\n{USAGE}");
            let _ = std::io::stderr().flush();
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every profile keeps the shape its AHK ancestor had, because the point is
    /// that muscle memory and any password-policy assumptions carry over.
    #[test]
    fn profiles_keep_their_shapes() {
        let dpw = generate("dpw").unwrap();
        assert_eq!(dpw.len(), 15, "Xxxxxxx_yyyyyyy is 7 + 1 + 7: {dpw}");
        assert_eq!(dpw.chars().nth(7), Some('_'));
        assert!(dpw.chars().next().unwrap().is_ascii_uppercase());

        assert_eq!(generate("pw").unwrap().len(), 16);
        assert_eq!(generate("wpw").unwrap().len(), 16);
        assert_eq!(generate("npw").unwrap().len(), 16);
        assert_eq!(generate("hex").unwrap().len(), 16);
        assert_eq!(generate("spw").unwrap().chars().count(), 16);
        assert_eq!(generate("qpw").unwrap().len(), 19); // 4*4 + 3 dashes

        let uuid = generate("uuid").unwrap();
        assert_eq!(uuid.len(), 36);
        assert_eq!(uuid.match_indices('-').count(), 4);

        assert_eq!(generate("jpw").unwrap().len(), 17); // 8 + '-' + 8
    }

    #[test]
    fn alphabets_exclude_the_ambiguous_characters() {
        for _ in 0..40 {
            for profile in ["dpw", "pw", "qpw"] {
                let out = generate(profile).unwrap();
                for bad in ['0', 'O', 'I', 'l'] {
                    assert!(!out.contains(bad), "{profile} produced {bad:?}: {out}");
                }
            }
        }
    }

    /// The AHK symbol set contained a literal space and a duplicated comma.
    #[test]
    fn the_symbol_alphabet_has_no_space_and_no_duplicates() {
        assert!(
            !SYMBOLS.contains(' '),
            "a space in a password is a support call"
        );
        let mut seen: Vec<char> = SYMBOLS.chars().collect();
        let before = seen.len();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(
            before,
            seen.len(),
            "duplicate characters skew the distribution"
        );
    }

    /// A quote or backslash in a password must survive the effect language.
    #[test]
    fn escaping_survives_the_effect_parser() {
        assert_eq!(escape("a\"b"), "a\\\"b");
        assert_eq!(escape("a\\b"), "a\\\\b");

        let line = format!("k \"{}\"", escape("pa\"ss\\word"));
        match capslockx_core::effects::parse(&line) {
            capslockx_core::effects::Effect::Type(t) => assert_eq!(t, "pa\"ss\\word"),
            other => panic!("did not round-trip through the effect parser: {other:?}"),
        }
    }

    /// Every symbol password round-trips, not just a hand-picked one.
    #[test]
    fn generated_symbol_passwords_all_round_trip() {
        for _ in 0..200 {
            let pw = generate("spw").unwrap();
            let line = format!("k \"{}\"", escape(&pw));
            match capslockx_core::effects::parse(&line) {
                capslockx_core::effects::Effect::Type(t) => assert_eq!(t, pw),
                other => panic!("a generated password did not survive as an effect: {other:?}"),
            }
        }
    }

    /// Not a randomness test — a wiring test. If `pick` were stuck, or an
    /// alphabet were being truncated, a whole region of it would never appear.
    #[test]
    fn the_whole_alphabet_gets_used() {
        let drawn: std::collections::HashSet<char> = draw(HEX_LOWER, 4000).chars().collect();
        for c in HEX_LOWER.chars() {
            assert!(drawn.contains(&c), "{c:?} never came up in 4000 draws");
        }
    }

    #[test]
    fn an_unknown_profile_is_refused_rather_than_guessed() {
        assert!(generate("nope").is_none());
    }
}
