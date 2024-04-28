# Physis

[![Crates.io](https://img.shields.io/crates/v/physis)](https://crates.io/crates/physis)

Physis is a library for reading and writing FFXIV data.

Even though this library was written with and for [Rust](https://www.rust-lang.org/) in mind, Physis has bindings for other languages:
* [PhysisSharp](https://github.com/redstrate/PhysisSharp) can be used in any C# application, and is built on top of libphysis.
* [libphysis](https://github.com/redstrate/libphysis) can be used for anything that can interface with the C FFI. [Novus](https://github.com/redstrate/Novus) and [Astra](https://github.com/redstrate/Astra) is built on top of libphysis, for example.

## Goals

Physis should:

* ... Make it easy for developers to tinker around with game data.
* ... Document file formats along the way, to make writing future and applications libraries easier.
* ... Make parsing data safe, and create automated tests when possible.
* ... Aim to be have minimal dependencies, and easy to use regardless of platform.

## Features

Here is a list of supported formats and their status:

| File Format | Read | Write | Note |
| --- | --- | --- | --- |
| [Configuration files](https://docs.xiv.zone/format/cfg/) | ✅ | ✅ | |
| [Saved character data](https://docs.xiv.zone/format/chardat/) | ✅ | ❌ | Only some versions are currently supported. |
| [Chara make params](https://docs.xiv.zone/format/cmp/) | ✅ | ❌ | |
| Dictionaries | ✅ | ❌ | |
| [Excel data](https://docs.xiv.zone/format/exd/) | ✅ | ❌ | |
| [File infos](https://docs.xiv.zone/format/fiin/) | ✅ | ✅ | |
| Map layers | ✅ | ❌ | Layer support isn't well tested yet. |
| [Chat logs](https://docs.xiv.zone/format/log/) | ✅ | ❌ | Not all chat categories are discovered yet. |
| [Models](https://docs.xiv.zone/format/mdl/) | ✅ | ✅ | Adding custom shape keys aren't fully supported yet. |
| [Materials](https://docs.xiv.zone/format/mtrl/) | ✅ | ❌ | |
| Patch files | ✅ | ❌ | |
| Pre bone deformers | ✅ | ❌ | |
| [Shader packages](https://docs.xiv.zone/format/shpk/) | ✅ | ❌ | |
| [Skeletons](https://docs.xiv.zone/format/sklb/) | ✅ | ❌ | |
| Terrain | ✅ | ❌ | |
| [Textures](https://docs.xiv.zone/format/tex/) | ✅ | ❌ | Only some formats are supported. |

Physis also supports doing some other useful things other than reading and writing file formats:

* Extract game data from SqPack files, and list file hashes from index/index2.
* Apply game patches. Indexed ZiPatch is not yet supported, though.
* Blockfish ciphers for encrypting and decrypting [SqexArg](https://docs.xiv.zone/concept/sqexarg/).
* Extract retail installer contents, useful on Linux for avoiding having to run the InstallShield installer.
* Construct paths to equipment, items, faces, and other useful models.
* Extract strings from executables.

## Usage

If you want to use Physis in your Rust project, you can simply add it as a dependency in `Cargo.toml`:

```toml
[dependencies]
physis = "0.2"
```

Documentation is availavble online at [docs.xiv.zone](https://docs.xiv.zone/docs/physis). It's automatically updated as new
commits are pushed to the main branch.

C# projects can use [PhysisSharp](https://github.com/redstrate/PhysisSharp) which exposes Physis in C#.

C/C++ projects (or anything that can interface with C libraries) can use [libphysis](https://github.com/redstrate/libphysis) which exposes Physis functionality under a C API.

## Building

Physis only has a few dependencies, and very little if nothing is turned on by default. You need to set up [Rust](https://www.rust-lang.org/learn/get-started) and then run `cargo build`.

If you want to build the `game_install` feature, you also need to install [unshield](https://github.com/twogood/unshield).

## Development

If you're interested to read how these formats work in more detail, see [xiv.dev](https://xiv.dev/) and
[docs.xiv.zone](https://docs.xiv.zone).

### Testing

One of the main goals of Physis is to avoid accidental regressions, this is especially important when handling game
data that might take hours to download.

#### Unit Testing

There are a set of basic unit tests you can run via `cargo test`. You can also find the relevant test resources in `resources/tests`.
This does **NOT** contain copyrighted material, but actually fake game data created by physis itself. These tests are
run automatically by the CI.

#### Retail Testing

There are some tests and benchmarks require the environment variable `FFXIV_GAME_DIR` to be set. By default, these are disabled
since they require a legitimate copy of the retail game data. These tests can be turned on via the `retail_game_testing`
feature.

#### Patch Testing

Patching is an extremely sensitive operation since it is not easily reversible if done wrong. Repairing the game files
is an option, but it's time-consuming and not yet implemented in physis. To prevent regressions in patching the
game, I have set up a testing bed for cross-checking our implementation with others. Currently, this is limited to XIVLauncher's implementation,
but I will eventually adopt a way to test the retail patch installer as well.

1. Enable the `patch_testing` feature.
2. Set a couple of environment variables:
   * `FFXIV_PATCH_DIR` is the directory of patches to install. It should be structured as `$FFXIV_PATCH_DIR/game/D2017.07.11.0000.0001.patch`.
   * `FFXIV_XIV_LAUNCHER_PATCHER` should be the path to the XIVLauncher patcher executable. If you're running on Linux, we will handle running Wine for you.
   * `FFXIV_INSTALLER` is the path to the installer executable. This will be installed using the usual InstallShield emulation physis already includes.

As you can see, you must have the previous patches downloaded first as well as the installer before running the tests.
This is left up to the developer to figure out how to download them legally.

**Note:** These tests create the `game_test` and `game_test_xivlauncher` folders in `$HOME` and does not
delete them on exit, in case you want to check on the results. You may want to remove these folders as they
are full game installations and take up a considerable amount of space.

#### Semver and Dependency Checks

Even though package management in Rust is easier, it's a double-edged sword. I try to prevent getting carried away
from including crates - but the ones we do include, have to get checked often. I use `cargo deny` to check my
dependencies for mismatched versions, deprecation warnings, updates and more. This is also run on the CI!

Making sure that the library is semver-compliant is also important, and I use `cargo semver` for this task. This is to ensure the API does not break when moving between patch
versions.

## Contributing & Support

The best way you can help is by [monetarily supporting me](https://redstrate.com/fund/) or by submitting patches to
help fix bugs or add functionality. Filing issues is appreciated, but I do this in my free time so please don't expect professional support.

## Credits & Thank You
- [goatcorp](https://goatcorp.github.io) (XIVQuickLauncher, docs.xiv.dev, and even more)
- [Ioncannon](http://ffxivexplorer.fragmenterworks.com/research.php) (FFXIV Data Explorer) for the first documenting the file formats
- [binrw team](https://binrw.rs) for an awesome Rust library!
- [sha1-smol](https://github.com/mitsuhiko/sha1-smol) for a dependency-free SHA1 implementation

And everyone else who writes FFXIV tools!

## License

![GPLv3](https://www.gnu.org/graphics/gplv3-127x51.png)

This project is licensed under the GNU General Public License 3. Some parts of the code or assets may be licensed differently, refer to the REUSE metadata.
