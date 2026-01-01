# Physis

[![Crates.io](https://img.shields.io/crates/v/physis)](https://crates.io/crates/physis) [![Docs Badge](https://img.shields.io/badge/docs-latest-blue)](https://docs.xiv.zone/docs/physis)

Physis is a library for reading and writing FFXIV data. It knows how to read many of the game's formats, and can write some of them too.

## Supported Game Versions

All game versions are supported, including benchmark versions of the game. Support for other platforms like the PS3 is somewhat available.

## Supported Target Platforms

Physis compiles and runs on all major platforms including Windows, macOS, Linux and WebAssembly.

## Usage

Physis exposes it's API in a few different languages:

### Rust

If you want to use Physis in your Rust project, you can simply add it as a dependency in `Cargo.toml`:

```toml
[dependencies]
physis = "0.5"
```

Documentation is available online at [docs.xiv.zone](https://docs.xiv.zone/docs/physis). It's automatically updated as new
commits are pushed to the main branch.

If you need a high-level Excel API, see [Icarus](https://github.com/redstrate/Icarus) which is based off of Physis.

### C and C++

C and C++ projects (or any language that can interface with C) can use [libphysis](https://github.com/redstrate/libphysis).

## Building

You need to set up [Rust](https://www.rust-lang.org/learn/get-started) and then run `cargo build`. Although Physis is a library, we have a few examples you can run.

## Contributing & Support

Feel free to submit patches to help fix bugs or add functionality. Filing issues is appreciated, but I do this in my free time so please don't expect professional support.

See [CONTRIBUTING](CONTRIBUTING.md) for more information about contributing back to the project!

## Credits & Thank You

* [goatcorp](https://goatcorp.github.io) (XIVQuickLauncher, docs.xiv.dev, and even more)
* [Ioncannon](http://ffxivexplorer.fragmenterworks.com/research.php) (FFXIV Data Explorer) for the first documenting the file formats
* [binrw team](https://binrw.rs) for an awesome Rust library!
* [sha1-smol](https://github.com/mitsuhiko/sha1-smol) for a dependency-free SHA1 implementation
* [FFXIVTools](https://github.com/dlunch/FFXIVTools) for it's Havok parsing implementation
* [texture2ddecoder](https://github.com/UniversalGameExtraction/texture2ddecoder/) for it's BCn texture decoding functions.

And everyone else who writes FFXIV tools!

## License

![GPLv3](https://www.gnu.org/graphics/gplv3-127x51.png)

This project is licensed under the GNU General Public License 3. Some parts of the code or assets may be licensed differently, refer to the REUSE metadata.
