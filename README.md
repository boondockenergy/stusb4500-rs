# STUSB4500 
This is a platform agnostic Rust driver for the for STUSB4500 I2C
USB-PD Sink controller using the [`embedded-hal`] traits.

This driver allows you to:
- Read and write sink PDO's (Power Data Object) from the controller.
- Negotiate a power contract with the source.
- Query current source capabilities.

**Still under development. Only basic PDO control has been tested (set_pdo & soft_reset) as working**

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[`embedded-hal`]: https://github.com/rust-embedded/embedded-hal