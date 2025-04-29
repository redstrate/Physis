# Physis

[![Crates.io](https://img.shields.io/crates/v/physis)](https://crates.io/crates/physis)

Physis is a library for reading and writing FFXIV data.

## Features

Here is a list of supported formats and their status:

| File Format | Read | Write | Note                                                                                               |
| --- | --- |-------|----------------------------------------------------------------------------------------------------|
| [Configuration files](https://docs.xiv.zone/format/cfg/) | ✅ | ✅     |                                                                                                    |
| [Saved character data](https://docs.xiv.zone/format/chardat/) | ✅ | ✅     |                                                         |
| [Chara make params](https://docs.xiv.zone/format/cmp/) | ✅ | ❌     |                                                                                                    |
| Dictionaries | ✅ | ❌     |                                                                                                    |
| [Excel data](https://docs.xiv.zone/format/exd/) | ✅ | ❌     |                                                                                                    |
| [File infos](https://docs.xiv.zone/format/fiin/) | ✅ | ✅     |                                                                                                    |
| Map layers | ✅ | ❌     | Layer support isn't well tested yet.                                                               |
| [Chat logs](https://docs.xiv.zone/format/log/) | ✅ | ❌     | Not all chat categories are discovered yet.                                                        |
| [Models](https://docs.xiv.zone/format/mdl/) | ✅ | ✅     | Adding custom shape keys aren't fully supported yet.                                               |
| [Materials](https://docs.xiv.zone/format/mtrl/) | ✅ | ❌     |                                                                                                    |
| Patch files | ✅ | ~     | ZiPatch writing support is currently being worked on, but many operations are not yet implemented. |
| Pre bone deformers | ✅ | ❌     |                                                                                                    |
| [Shader packages](https://docs.xiv.zone/format/shpk/) | ✅ | ❌     |                                                                                                    |
| [Skeletons](https://docs.xiv.zone/format/sklb/) | ✅ | ❌     |                                                                                                    |
| Terrain | ✅ | ❌     |                                                                                                    |
| [Textures](https://docs.xiv.zone/format/tex/) | ✅ | ❌     | Only some formats are supported.                                                                   |

Physis also supports doing some other useful things other than reading and writing file formats:

* Extract game data from SqPack files, and list file hashes from index/index2.
* Apply game patches. Indexed ZiPatch is not yet supported, though.
* Blockfish ciphers for encrypting and decrypting [SqexArg](https://docs.xiv.zone/concept/sqexarg/).
* Construct paths to equipment, items, faces, and other useful models.

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
