# Physis

[![Crates.io](https://img.shields.io/crates/v/physis)](https://crates.io/crates/physis)
[![builds.sr.ht status](https://builds.sr.ht/~redstrate/physis.svg)](https://builds.sr.ht/~redstrate/physis?)

Physis is a framework for interacting with FFXIV data. It can read and write lots of game formats, and is designed for tooling to be built on top of it. Even though this library works best with Rust, [libphysis](https://git.sr.ht/~redstrate/libphysis) is a C API wrapper designed for interfacing through other languages.

## Goals
* Make it easy for people to tinker around with game data. 
* Documenting game formats for other people to develop their own libraries and applications for.
* Parsing data should be safe, and unit tested vigorously.
* Aim to have minimal dependencies, and those dependencies should be checked via `cargo deny`.

## Features

* Easily extract game data from SqPack files.
* Apply game and boot updates, enabling custom launchers to patch the game easily.
* Blockfish ciphers for encrypting and decrypting SqexArg.
* Parse various game formats:
  * SqPack index and dat files
  * ZiPatch files
  * All three Excel data types (EXD, EXH, EXL)
  * Models
  * Havok Packfile/TexTool skeletons
  * Textures
  * Materials
  * and more!

## Usage

**Warning:** The API will not be truly stable until 1.0. However, the API is stable between patch versions.

If you intend to use this in your Rust project, you can simply include this crate directly.

```
[dependencies]
physis = "0.1"
```

You can view the documentation at [docs.xiv.zone](https://docs.xiv.zone/docs/physis)! It's automatically updated as new
commits are pushed to the main branch.

For other use in languages I maintain [libphysis](https://git.sr.ht/~redstrate/libphysis), which is a C wrapper
around the same functionality. I use these bindings in [other projects](https://git.sr.ht/~redstrate/astra).
  
## Development

If you're interested to see how these formats work in more detail, see [xiv.dev](https://xiv.dev/) and
[docs.xiv.zone](https://docs.xiv.zone)! They explain the file formats in more detail, but I also encourage reading the
ibrary code as well if you can.

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

The best way you can help is by [monetarily supporting me](https://redstrate.com/about/) or by submitting patches to
help fix bugs or add functionality.

## Credits
- [goatcorp](https://goatcorp.github.io) (XIVQuickLauncher, docs.xiv.dev, and even more)
- [Ioncannon](http://ffxivexplorer.fragmenterworks.com/research.php) (FFXIV Data Explorer) for the first documenting the file formats
- [binrw team](https://binrw.rs) for an awesome Rust library!
- [sha1-smol](https://github.com/mitsuhiko/sha1-smol) for a dependency-free SHA1 implementation

And everyone else who writes FFXIV tools!

## License

![GPLv3](https://www.gnu.org/graphics/gplv3-127x51.png)

This project is licensed under the GNU General Public License 3. Some parts of the code or assets may be licensed differently, refer to the REUSE metadata.