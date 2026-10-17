# Physis

[![Crates.io](https://img.shields.io/crates/v/physis)](https://crates.io/crates/physis) [![Docs Badge](https://img.shields.io/badge/docs-latest-blue)](https://docs.xiv.zone/docs/physis)

Physis is a [Rust](https://www.rust-lang.org/learn/get-started) library for reading and writing FFXIV data.

```rust,no_run
use physis::{
    Result,
    resource::{SqPackResource, Resource},
    model::MDL,
};

fn main() -> Result<()> {
    // Construct a resource to read from SqPack:
    let mut resource = SqPackResource::from_existing("game");

    // Read the raw data of this file, our resource takes care of decompressing it:
    let bytes = resource.read("bgcommon/nature/earth/bgparts/earth01.mdl")?;

    // Or read and parse it:
    let mdl = resource.parsed::<MDL>("bgcommon/nature/earth/bgparts/earth01.mdl")?;
    
    Ok(())
}
```

Physis can be used to [apply patches for launchers](https://github.com/redstrate/Astra) or [build modding tools](https://github.com/redstrate/Novus)! We also [power server emulators](https://codeberg.org/redstrate/Kawari) so they can process Excel and zone data.

## Supported Game Versions

We aim to support all game versions and expansions, including the benchmarks. We also try to support all platforms, including defunct ones like the Playstation 3.

Physis does not support FINAL FANTASY XIV 1.x, except for applying their ZiPatch files. Check out our sister project [Krasis](https://codeberg.org/redstrate/Krasis).

## Supported Target Platforms

Physis compiles and runs on all major platforms including Windows, macOS, Linux and WebAssembly.

## Usage

To use Physis in your project, add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
physis = "0.7"
```

Documentation is available online at [docs.xiv.zone](https://docs.xiv.zone/docs/physis). It's automatically updated as new
commits are pushed to the main branch.

### C, C++ and others

C and C++ projects (or any compatible language through FFI) can use [libphysis](https://github.com/redstrate/libphysis).

## Contributing & Support

Feel free to submit PRs for fixing bugs or adding functionality. Filing issues is appreciated, but I do this in my free time so please don't expect professional support.

See [CONTRIBUTING](CONTRIBUTING.md) for more information about contributing back to the project!

## Credits & Thank You

* [Ioncannon](http://ffxivexplorer.fragmenterworks.com/research.php) for initially documenting the file formats.
* [ironworks](https://github.com/ackwell/ironworks) for reference and inspiration.
* [xiv.dev](https://https://xiv.dev/) for reference.

This project integrates code from [texture2ddecoder](https://github.com/UniversalGameExtraction/texture2ddecoder/) and [FFXIVTools](https://github.com/dlunch/FFXIVTools).

## License

![GPLv3](https://www.gnu.org/graphics/gplv3-127x51.png)

This project is licensed under the GNU General Public License 3. Some parts of the code or assets may be licensed differently, refer to the REUSE metadata.
