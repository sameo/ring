// Copyright 2015-2016 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

//! Testing framework.
//!
//! Unlike the rest of *ring*, this testing framework uses panics pretty
//! liberally. It was originally designed for internal use--it drives most of
//! *ring*'s internal tests, and so it is optimized for getting *ring*'s tests
//! written quickly at the expense of some usability. The documentation is
//! lacking. The best way to learn it is to look at some examples. The digest
//! tests are the most complicated because they use named sections. Other tests
//! avoid named sections and so are easier to understand.
//!
//! # Examples
//!
//! ## Writing Tests
//!
//! Input files look like this:
//!
//! ```text
//! # This is a comment.
//!
//! HMAC = SHA1
//! Input = "My test data"
//! Key = ""
//! Output = 61afdecb95429ef494d61fdee15990cabf0826fc
//!
//! HMAC = SHA256
//! Input = "Sample message for keylen<blocklen"
//! Key = 000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F
//! Output = A28CF43130EE696A98F14A37678B56BCFCBDD9E5CF69717FECF5480F0EBDF790
//! ```
//!
//! Test cases are separated with blank lines. Note how the bytes of the `Key`
//! attribute are specified as a quoted string in the first test case and as
//! hex in the second test case; you can use whichever form is more convenient
//! and you can mix and match within the same file. The empty sequence of bytes
//! can only be represented with the quoted string form (`""`).
//!
//! Here's how you would consume the test data:
//!
//! ```ignore
//! use ring::test;
//!
//! test::from_file("src/hmac_tests.txt", |section, test_case| {
//!     assert_eq!(section, ""); // This test doesn't use named sections.
//!
//!     let digest_alg = test_case.consume_digest_alg("HMAC");
//!     let input = test_case.consume_bytes("Input");
//!     let key = test_case.consume_bytes("Key");
//!     let output = test_case.consume_bytes("Output");
//!
//!     // Do the actual testing here
//! });
//! ```
//!
//! Note that `consume_digest_alg` automatically maps the string "SHA1" to a
//! reference to `digest::SHA1`, "SHA256" to `digest::SHA256`, etc.
//!
//! ## Output When a Test Fails
//!
//! When a test case fails, the framework automatically prints out the test
//! case. If the test case failed with a panic, then the backtrace of the panic
//! will be printed too. For example, let's say the failing test case looks
//! like this:
//!
//! ```text
//! Curve = P-256
//! a = 2b11cb945c8cf152ffa4c9c2b1c965b019b35d0b7626919ef0ae6cb9d232f8af
//! b = 18905f76a53755c679fb732b7762251075ba95fc5fedb60179e730d418a9143c
//! r = 18905f76a53755c679fb732b7762251075ba95fc5fedb60179e730d418a9143c
//! ```
//! If the test fails, this will be printed (if `$RUST_BACKTRACE` is `1`):
//!
//! ```text
//! src/example_tests.txt: Test panicked.
//! Curve = P-256
//! a = 2b11cb945c8cf152ffa4c9c2b1c965b019b35d0b7626919ef0ae6cb9d232f8af
//! b = 18905f76a53755c679fb732b7762251075ba95fc5fedb60179e730d418a9143c
//! r = 18905f76a53755c679fb732b7762251075ba95fc5fedb60179e730d418a9143c
//! thread 'example_test' panicked at 'Test failed.', src\test.rs:206
//! stack backtrace:
//!    0:     0x7ff654a05c7c - std::rt::lang_start::h61f4934e780b4dfc
//!    1:     0x7ff654a04f32 - std::rt::lang_start::h61f4934e780b4dfc
//!    2:     0x7ff6549f505d - std::panicking::rust_panic_with_hook::hfe203e3083c2b544
//!    3:     0x7ff654a0825b - rust_begin_unwind
//!    4:     0x7ff6549f63af - std::panicking::begin_panic_fmt::h484cd47786497f03
//!    5:     0x7ff654a07e9b - rust_begin_unwind
//!    6:     0x7ff654a0ae95 - core::panicking::panic_fmt::h257ceb0aa351d801
//!    7:     0x7ff654a0b190 - core::panicking::panic::h4bb1497076d04ab9
//!    8:     0x7ff65496dc41 - from_file<closure>
//!                         at C:\Users\Example\example\<core macros>:4
//!    9:     0x7ff65496d49c - example_test
//!                         at C:\Users\Example\example\src\example.rs:652
//!   10:     0x7ff6549d192a - test::stats::Summary::new::ha139494ed2e4e01f
//!   11:     0x7ff6549d51a2 - test::stats::Summary::new::ha139494ed2e4e01f
//!   12:     0x7ff654a0a911 - _rust_maybe_catch_panic
//!   13:     0x7ff6549d56dd - test::stats::Summary::new::ha139494ed2e4e01f
//!   14:     0x7ff654a03783 - std::sys::thread::Thread::new::h2b08da6cd2517f79
//!   15:     0x7ff968518101 - BaseThreadInitThunk
//! ```
//!
//! Notice that the output shows the name of the data file
//! (`src/example_tests.txt`), the test inputs that led to the failure, and the
//! stack trace to the line in the test code that panicked: entry 9 in the
//! stack trace pointing to line 652 of the file `example.rs`.

#[cfg(feature = "use_heap")]
use bits;

use {digest, error};

use std;
use std::string::String;
use std::vec::Vec;
use std::io::BufRead;

/// A test case. A test case consists of a set of named attributes. Every
/// attribute in the test case must be consumed exactly once; this helps catch
/// typos and omissions.
#[derive(Debug)]
pub struct TestCase {
    attributes: Vec<(String, String, bool)>,
}

impl TestCase {
    /// Maps the strings "SHA1", "SHA256", "SHA384", and "SHA512" to digest
    /// algorithms, maps "SHA224" to `None`, and panics on other (erroneous)
    /// inputs. "SHA224" is mapped to None because *ring* intentionally does
    /// not support SHA224, but we need to consume test vectors from NIST that
    /// have SHA224 vectors in them.
    pub fn consume_digest_alg(&mut self, key: &str)
                              -> Option<&'static digest::Algorithm> {
        let name = self.consume_string(key);
        match name.as_ref() {
            "SHA1" => Some(&digest::SHA1),
            "SHA224" => None, // We actively skip SHA-224 support.
            "SHA256" => Some(&digest::SHA256),
            "SHA384" => Some(&digest::SHA384),
            "SHA512" => Some(&digest::SHA512),
            "SHA512_256" => Some(&digest::SHA512_256),
            _ => panic!("Unsupported digest algorithm: {}", name),
        }
    }

    /// Returns the value of an attribute that is encoded as a sequence of an
    /// even number of hex digits, or as a double-quoted UTF-8 string. The
    /// empty (zero-length) value is represented as "".
    pub fn consume_bytes(&mut self, key: &str) -> Vec<u8> {
        let mut s = self.consume_string(key);
        if s.starts_with('\"') {
            // The value is a quoted strong.
            // XXX: We don't deal with any inner quotes.
            if !s.ends_with('\"') {
                panic!("expected quoted string, found {}", s);
            }
            let _ = s.pop();
            let _ = s.remove(0);
            Vec::from(s.as_bytes())
        } else {
            // The value is hex encoded.
            match from_hex(&s) {
                Ok(s) => s,
                Err(ref err_str) => {
                    panic!("{} in {}", err_str, s);
                },
            }
        }
    }

    /// Returns the value of an attribute that is an integer, in decimal
    /// notation.
    pub fn consume_usize(&mut self, key: &str) -> usize {
        let s = self.consume_string(key);
        s.parse::<usize>().unwrap()
    }

    /// Returns the value of an attribute that is an integer, in decimal
    /// notation, as a bit length.
    #[cfg(feature = "use_heap")]
    pub fn consume_usize_bits(&mut self, key: &str) -> bits::BitLength {
        let s = self.consume_string(key);
        let bits = s.parse::<usize>().unwrap();
        bits::BitLength::from_usize_bits(bits)
    }

    /// Returns the raw value of an attribute, without any unquoting or
    /// other interpretation.
    pub fn consume_string(&mut self, key: &str) -> String {
        self.consume_optional_string(key)
            .unwrap_or_else(|| panic!("No attribute named \"{}\"", key))
    }

    /// Like `consume_string()` except it returns `None` if the test case
    /// doesn't have the attribute.
    pub fn consume_optional_string(&mut self, key: &str) -> Option<String> {
        for &mut (ref name, ref value, ref mut consumed) in
                &mut self.attributes {
            if key == name {
                if *consumed {
                    panic!("Attribute {} was already consumed", key);
                }
                *consumed = true;
                return Some(value.clone());
            }
        }
        None
    }
}

/// Returns the path for *ring* source code root.
///
/// On iOS, source are assumed to be copied in the application bundle, as
/// a "src" directory along the test runner.
#[cfg(target_os = "ios")]
pub fn ring_src_path() -> std::path::PathBuf {
    std::env::current_exe().unwrap().parent().unwrap().join("src")
}

/// Returns the path for *ring* source code root.
///
/// On most platforms, the tests are run by cargo, so it's just the current
/// working directory.
#[cfg(not(target_os = "ios"))]
pub fn ring_src_path() -> std::path::PathBuf {
    std::path::PathBuf::from(".")
}

/// Reads test cases out of the file with the path given by
/// `test_data_relative_file_path`, calling `f` on each vector until `f` fails
/// or until all the test vectors have been read. `f` can indicate failure
/// either by returning `Err()` or by panicking.
pub fn from_file<F>(test_data_relative_file_path: &str, mut f: F)
                    where F: FnMut(&str, &mut TestCase)
                                   -> Result<(), error::Unspecified> {
    let path = ring_src_path().join(test_data_relative_file_path);
    let file = std::fs::File::open(path).unwrap();
    let mut lines = std::io::BufReader::new(&file).lines();

    let mut current_section = String::from("");
    let mut failed = false;

    while let Some(mut test_case) = parse_test_case(&mut current_section,
                                                    &mut lines) {
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                f(&current_section, &mut test_case)
            }));
        let result = match result {
            Ok(Ok(())) => {
                if !test_case.attributes.iter().any(
                        |&(_, _, ref consumed)| !consumed) {
                    Ok(())
                } else {
                    failed = true;
                    Err("Test didn't consume all attributes.")
                }
            },
            Ok(Err(_)) => Err("Test returned Err(error::Unspecified)."),
            Err(_) => Err("Test panicked."),
        };

        if let Err(msg) = result {
            failed = true;

            println!("{}: {}", test_data_relative_file_path, msg);
            for (ref name, ref value, ref consumed) in test_case.attributes {
                let consumed_str = if *consumed { "" } else { " (unconsumed)" };
                println!("{}{} = {}", name, consumed_str, value);
            }
        };
    }

    if failed {
        panic!("Test failed.")
    }
}

/// Decode an string of hex digits into a sequence of bytes. The input must
/// have an even number of digits.
pub fn from_hex(hex_str: &str) -> Result<Vec<u8>, String> {
    if hex_str.len() % 2 != 0 {
        return Err(
            String::from("Hex string does not have an even number of digits"));
    }

    fn from_hex_digit(d: u8) -> Result<u8, String> {
        if d >= b'0' && d <= b'9' {
            Ok(d - b'0')
        } else if d >= b'a' && d <= b'f' {
            Ok(d - b'a' + 10u8)
        } else if d >= b'A' && d <= b'F' {
            Ok(d - b'A' + 10u8)
        } else {
            Err(format!("Invalid hex digit '{}'", d as char))
        }
    }

    let mut result = Vec::with_capacity(hex_str.len() / 2);
    for digits in hex_str.as_bytes().chunks(2) {
        let hi = from_hex_digit(digits[0])?;
        let lo = from_hex_digit(digits[1])?;
        result.push((hi * 0x10) | lo);
    }
    Ok(result)
}

type FileLines<'a> = std::io::Lines<std::io::BufReader<&'a std::fs::File>>;

fn parse_test_case(current_section: &mut String, lines: &mut FileLines)
                   -> Option<TestCase> {
    let mut attributes = Vec::new();

    let mut is_first_line = true;
    loop {
        let line = match lines.next() {
            None => None,
            Some(result) => Some(result.unwrap()),
        };

        if cfg!(feature = "test_logging") {
            if let Some(ref text) = line {
                println!("Line: {}", text);
            }
        }

        match line {
            // If we get to EOF when we're not in the middle of a test case,
            // then we're done.
            None if is_first_line => {
                return None;
            },

            // End of the file on a non-empty test cases ends the test case.
            None => {
                return Some(TestCase { attributes });
            },

            // A blank line ends a test case if the test case isn't empty.
            Some(ref line) if line.is_empty() => {
                if !is_first_line {
                    return Some(TestCase { attributes });
                }
                // Ignore leading blank lines.
            },

            // Comments start with '#'; ignore them.
            Some(ref line) if line.starts_with('#') => {},

            Some(ref line) if line.starts_with('[') => {
                assert!(is_first_line);
                assert!(line.ends_with(']'));
                current_section.truncate(0);
                current_section.push_str(line);
                let _ = current_section.pop();
                let _ = current_section.remove(0);
            },

            Some(ref line) => {
                is_first_line = false;

                let parts: Vec<&str> = line.splitn(2, " = ").collect();
                if parts.len() != 2 {
                    panic!("Syntax error: Expected Key = Value.");
                };

                let key = parts[0].trim();
                let value = parts[1].trim();

                // Don't allow the value to be ommitted. An empty value can be
                // represented as an empty quoted string.
                assert_ne!(value.len(), 0);

                // Checking is_none() ensures we don't accept duplicate keys.
                attributes.push((String::from(key), String::from(value), false));
            },
        }
    }
}

/// Deterministic implementations of `ring::rand::SecureRandom`.
///
/// These implementations are particularly useful for testing implementations
/// of randomized algorithms & protocols using known-answer-tests where the
/// test vectors contain the random seed to use. They are also especially
/// useful for some types of fuzzing.
#[allow(missing_docs)]
pub mod rand {
    use core;
    use {error, polyfill, rand};

    /// An implementation of `SecureRandom` that always fills the output slice
    /// with the given byte.
    #[derive(Debug)]
    pub struct FixedByteRandom {
        pub byte: u8,
    }

    impl rand::SecureRandom for FixedByteRandom {
        fn fill(&self, dest: &mut [u8]) -> Result<(), error::Unspecified> {
            polyfill::slice::fill(dest, self.byte);
            Ok(())
        }
    }

    /// An implementation of `SecureRandom` that always fills the output slice
    /// with the slice in `bytes`. The length of the slice given to `slice`
    /// must match exactly.
    #[derive(Debug)]
    pub struct FixedSliceRandom<'a> {
        pub bytes: &'a [u8],
    }

    impl<'a> rand::SecureRandom for FixedSliceRandom<'a> {
        fn fill(&self, dest: &mut [u8]) -> Result<(), error::Unspecified> {
            dest.copy_from_slice(self.bytes);
            Ok(())
        }
    }

    /// An implementation of `SecureRandom` where each slice in `bytes` is a
    /// test vector for one call to `fill()`. *Not thread-safe.*
    ///
    /// The first slice in `bytes` is the output for the first call to
    /// `fill()`, the second slice is the output for the second call to
    /// `fill()`, etc. The output slice passed to `fill()` must have exactly
    /// the length of the corresponding entry in `bytes`. `current` must be
    /// initialized to zero. `fill()` must be called exactly once for each
    /// entry in `bytes`.
    #[derive(Debug)]
    pub struct FixedSliceSequenceRandom<'a> {
        /// The value.
        pub bytes: &'a [&'a [u8]],
        pub current: core::cell::UnsafeCell<usize>,
    }

    impl<'a> rand::SecureRandom for FixedSliceSequenceRandom<'a> {
        fn fill(&self, dest: &mut [u8]) -> Result<(), error::Unspecified> {
            let current = unsafe { *self.current.get() };
            let bytes = self.bytes[current];
            dest.copy_from_slice(bytes);
            // Remember that we returned this slice and prepare to return
            // the next one, if any.
            unsafe { *self.current.get() += 1 };
            Ok(())
        }
    }

    impl<'a> Drop for FixedSliceSequenceRandom<'a> {
        fn drop(&mut self) {
            // Ensure that `fill()` was called exactly the right number of
            // times.
            assert_eq!(unsafe { *self.current.get() }, self.bytes.len());
        }
    }
}


#[cfg(test)]
mod tests {
    use {error, test};

    #[test]
    fn one_ok() {
        test::from_file("src/test_1_tests.txt", |_, test_case| {
            let _ = test_case.consume_string("Key");
            Ok(())
        });
    }

    #[test]
    #[should_panic(expected = "Test failed.")]
    fn one_err() {
        test::from_file("src/test_1_tests.txt", |_, test_case| {
            let _ = test_case.consume_string("Key");
            Err(error::Unspecified)
        });
    }

    #[test]
    #[should_panic(expected = "Test failed.")]
    fn one_panics() {
        test::from_file("src/test_1_tests.txt", |_, test_case| {
            let _ = test_case.consume_string("Key");
            panic!("");
        });
    }

    #[test]
    #[should_panic(expected = "Test failed.")]
    fn first_err() { err_one(0) }

    #[test]
    #[should_panic(expected = "Test failed.")]
    fn middle_err() { err_one(1) }

    #[test]
    #[should_panic(expected = "Test failed.")]
    fn last_err() { err_one(2) }

    fn err_one(test_to_fail: usize) {
        let mut n = 0;
        test::from_file("src/test_3_tests.txt", |_, test_case| {
            let _ = test_case.consume_string("Key");
            let result = if n != test_to_fail {
                Ok(())
            } else {
                Err(error::Unspecified)
            };
            n += 1;
            result
        });
    }

    #[test]
    #[should_panic(expected = "Test failed.")]
    fn first_panic() { panic_one(0) }

    #[test]
    #[should_panic(expected = "Test failed.")]
    fn middle_panic() { panic_one(1) }

    #[test]
    #[should_panic(expected = "Test failed.")]
    fn last_panic() { panic_one(2) }

    fn panic_one(test_to_fail: usize) {
        let mut n = 0;
        test::from_file("src/test_3_tests.txt", |_, test_case| {
            let _ = test_case.consume_string("Key");
            if n == test_to_fail {
                panic!("Oh Noes!");
            };
            n += 1;
            Ok(())
        });
    }

    #[test]
    #[should_panic(expected = "Syntax error: Expected Key = Value.")]
    fn syntax_error() {
        test::from_file("src/test_1_syntax_error_tests.txt", |_, _| Ok(()));
    }

    #[test]
    #[should_panic]
    fn file_not_found() {
        test::from_file("src/test_file_not_found_tests.txt", |_, _| Ok(()));
    }
}
