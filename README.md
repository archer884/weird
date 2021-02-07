# weird

> Salted Base32 encoding for 64-bit values.

[Crockford Base32 Encoding](https://www.crockford.com/wrmg/base32.html) is most commonly used to make numeric identifiers slightly more user-resistant. Similar to [Hashids](http://hashids.org/), the purpose here is to make the identifiers shorter and less confusing. Unlike Hashids, Crockford Base32 does nothing to conceal the real value of the number (beyond the actual encoding, anyway) and the fact that they are sequential is still pretty obvious when you see consecutive identifiers side by side. **This is where Weird differs from Crockford.**

Rather than directly encoding values as Base32, weird employs a shuffled alphabet and applies an arbitrary salt to the encoded values. This is to give any observer the appearance of non-sequential data and to make it more difficult to derive the original identifiers based on the encoded identifiers.

This library does not support encoding and decoding of arbitrary data; there is [another library for that](https://crates.io/crates/base32). Additionally, the spec supports the idea of check digits, but this library currently does not.

**The primary purpose of this library is to provide high performance, user-resistant encoding of numeric identifiers.** To that end, both encoding and decoding are, in fact, pretty darn fast. How fast? According to my testing, `crockford` (on which this library is based) decodes **fifty times faster** and encodes **twenty-seven times faster** than `harsh`. This library is roughly half as fast, given that it is applying an additional operation.

## Usage

Unlike the original `crockford` library, an instance of `weird::Weird` must be configured before use. The user may create an instance based on a salt or based on a salt and alphabet. (See documentation for more about the alphabet.)

The alphabet itself is generated based on an implementation of `rand::Rng` that is unlikely to change (read: I'm much too lazy to change it) and which, therefore, should provide stable functionality for this constructor. If you prefer not to trust me on that (understandable!), implement your own rng and pass that to the constructor of `Alphabet`.

### Encoding

```rust
// A salt can be anything that translates into a slice of bytes.
let weird = Weird::from_salt("Salt goes here");
let encoded = weird.encode(13); // I have no idea what this would look like.
                                // It's random, remember?
```

#### Plan B (faster encoding)

The `Weird::encode_into()` method permits encoding directly to `std::fmt::Write`. This allows reuse of a buffer or... you know, whatever you wanna do. The standard encoding mechanism is, in fact, implemented in terms of this; it simply unwraps the result since, of course, encoding into a string can't really fail.

### Decoding

Decoding can fail (since the decoder can accept arbitrary strings), so it returns a result instead.

```rust
let weird = Weird::from_salt("Salt goes here.");
let decoded = weird.decode("Hello, world!"); // I bet this isn't valid.

let id = match decoded {
    Ok(id) => id,
    Err(_) => {
        println!("I knew that wasn't valid!");
        return;
    }
};
```