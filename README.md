# Physis

[![Crates.io](https://img.shields.io/crates/v/physis)](https://crates.io/crates/physis) [![Docs Badge](https://img.shields.io/badge/docs-latest-blue)](https://docs.xiv.zone/docs/physis)

Physis is a [Rust](https://www.rust-lang.org/learn/get-started) library for reading and writing FFXIV data.

```rust,no_run
use physis::{
    Error,
    resource::{SqPackResource, Resource},
    model::MDL,
};

fn main() -> Result<(), Error> {
    // Construct a resource to read from SqPack:
    let mut resource = SqPackResource::from_existing("game");

    // Read the raw data of this file, our resource takes care of decompressing it:
    let bytes = resource.read(".mdl").ok_or(Error::Unknown)?;

    // Or read and parse it:
    let mdl = resource.parsed::<MDL>("test.mdl")?;
    
    Ok(())
}
```

Physis can be used to [apply patches for launchers](https://github.com/redstrate/Astra), [build modding tools](https://github.com/redstrate/Novus) - and these are C++ projects! We can also [power server emulators](https://redstrate.com/Kawari) which deal with a lot of Excel and zone data. Our dependency tree is small, so [we are also easy to use on the web](https://github.com/redstrate/Auracite).

## Supported Game Versions

We aim to support all game versions (A Realm Reborn onward), including the benchmarks. We also try to support all platforms - including the Playstation 3.

## Supported Target Platforms

Physis compiles and runs on all major platforms including Windows, macOS, Linux and WebAssembly.

## Usage

To use Physis in your project, add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
physis = "0.5"
```

Documentation is available online at [docs.xiv.zone](https://docs.xiv.zone/docs/physis). It's automatically updated as new
commits are pushed to the main branch.

### C, C++ and others

C and C++ projects (or any compatible language through FFI) can use [libphysis](https://github.com/redstrate/libphysis).

## Contributing & Support

Feel free to submit PRs for fixing bugs or adding functionality. Filing issues is appreciated, but I do this in my free time so please don't expect professional support.

See [CONTRIBUTING](CONTRIBUTING.md) for more information about contributing back to the project!

## Credits & Thank You

* [ironworks](https://github.com/ackwell/ironworks) for inspiration and reference.
* [goatcorp](https://goatcorp.github.io) for XIVQuickLauncher, docs.xiv.dev, and more.
* [Ioncannon](http://ffxivexplorer.fragmenterworks.com/research.php) for initially documenting the file formats.
* [binrw team](https://binrw.rs) for the awesome Rust library that powers our parsing!
* [sha1-smol](https://github.com/mitsuhiko/sha1-smol) for a dependency-free SHA1 implementation.
* [FFXIVTools](https://github.com/dlunch/FFXIVTools) for it's Havok parsing implementation.
* [texture2ddecoder](https://github.com/UniversalGameExtraction/texture2ddecoder/) for it's BCn texture decoding functions.

And everyone else who writes open-source software for FFXIV!

## License

![GPLv3](https://www.gnu.org/graphics/gplv3-127x51.png)

This project is licensed under the GNU General Public License 3. Some parts of the code or assets may be licensed differently, refer to the REUSE metadata.
