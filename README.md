# Physis

[![Crates.io](https://img.shields.io/crates/v/physis)](https://crates.io/crates/physis) [![Docs Badge](https://img.shields.io/badge/docs-latest-blue)](https://docs.xiv.zone/docs/physis)

Physis is a library for reading and writing FFXIV data. It doesn't only know how to read many formats, but it can write some of them too.

## Supported Game Versions

All game versions are supported, including benchmark versions of the game. Only the Windows client data is tested and other platforms probably won't work.

## Supported Platforms

Physis compiles and runs on all major platforms including Windows, macOS, Linux and WebAssembly.

## Usage

If you want to use Physis in your Rust project, you can simply add it as a dependency in `Cargo.toml`:

```toml
[dependencies]
physis = "0.4"
```

Documentation is available online at [docs.xiv.zone](https://docs.xiv.zone/docs/physis). It's automatically updated as new
commits are pushed to the main branch.

C# projects can use [PhysisSharp](https://github.com/redstrate/PhysisSharp) which exposes Physis in C#.

C/C++ projects (or anything that can interface with C libraries) can use [libphysis](https://github.com/redstrate/libphysis).

## Building

You need to set up [Rust](https://www.rust-lang.org/learn/get-started) and then run `cargo build`. Although Physis is a library, we have a few examples you can run.

## Contributing & Support

Feel free to submit patches to help fix bugs or add functionality. Filing issues is appreciated, but I do this in my free time so please don't expect professional support.

See [CONTRIBUTING](CONTRIBUTING.md) for more information about the project.

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
