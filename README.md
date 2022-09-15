# Physis

![Crates.io](https://img.shields.io/crates/v/physis)
[![builds.sr.ht status](https://builds.sr.ht/~redstrate/physis.svg)](https://builds.sr.ht/~redstrate/physis?)

Framework for interacting with FFXIV data, and successor to [libxiv](https://git.sr.ht/~redstrate/libxiv). This intended for 
developers writing modding tools, launchers and other programs.

## Goals
* Make it extremely easy for people to tinker around with game data. 
* Parsing data should be safe, and unit tested vigorously.
* Minimal dependencies ;-) All dependencies are also checked by `cargo deny`.

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

## Usage

**Note:** The API will not be stable until 1.0.

If you intend to use this in a Rust project, you can simply include this crate directly. You can view the documentation at [docs.xiv.zone](https://docs.xiv.zone/docs/physis)!

For other use in languages I maintain [libphysis](https://git.sr.ht/~redstrate/libphysis), which is a C wrapper
around the same functionality. This isn't for show, I actually use these bindings in [other projects](https://git.sr.ht/~redstrate/astra).
  
## Development

If you're interested to see how these formats work in more detail, see [xiv.dev](https://xiv.dev/) and [docs.xiv.zone](https://docs.xiv.zone)!
They explain the file formats in more detail, but I also encourage reading the library code as well if you can.

Some tests and benchmarks require the environment variable `FFXIV_GAME_DIR` to be set. By default, these are disabled
since they require a legitimate copy of the retail game data. These tests can be turned on via the `retail_game_testing`
feature.

## Contributing & Support

The best way you can help is by [monetarily supporting me](https://redstrate.com/about/) or by submitting patches to help fix bugs or add functionality!
