# weird

> Salted Base32 encoding for 64-bit values.

[Crockford Base32 Encoding](https://www.crockford.com/wrmg/base32.html) is most commonly used to make numeric identifiers slightly more user-resistant. Similar to [Hashids](http://hashids.org/), the purpose here is to make the identifiers shorter and less confusing. Unlike Hashids, Crockford Base32 does nothing to conceal the real value of the number (beyond the actual encoding, anyway) and the fact that they are sequential is still pretty obvious when you see consecutive identifiers side by side. **This is where Weird differs from Crockford.**

Rather than directly encoding values as Base32, weird first **applies an arbitrary salt,** which may be supplied statically or at runtime, to all encoded values.

This library does not support encoding and decoding of arbitrary data; there is [another library for that](https://crates.io/crates/base32). Additionally, the spec supports the idea of check digits, but this library currently does not.

**The primary purpose of this library is to provide high performance, user-resistant encoding of numeric identifiers.** To that end, both encoding and decoding are, in fact, pretty darn fast. How fast? According to my testing, `crockford` (on which this library is based) decodes **fifty times faster** and encodes **twenty-seven times faster** than `harsh`.