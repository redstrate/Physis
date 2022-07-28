# Physis

Framework for interacting with FFXIV data, and successor to [libxiv](https://git.sr.ht/~redstrate/libxiv). This intended for 
developers writing modding tools, launchers and other programs.

**Note:** This library is still experimental, and no releases are planned. I'm currently busy with bringing all of libxiv's
features over.

## Goals
* Make it extremely easy for people to tinker around with game data. 
* Parsing data should be safe, and unit tested vigorously.
* Minimal dependencies ;-) All dependencies are also checked by `cargo deny`.

## Features

* Apply game patches, enabling custom launchers to patch the game.
* Blockfish ciphers for encrypting and decrypting SqexArg.
* Parse various game formats:
  * INDEX
  * DAT
  * ZiPatch
  
## Development

If you're interested to see how these formats work in more detail, see [xiv.dev](https://xiv.dev/) and [docs.xiv.zone](https://docs.xiv.zone)!

Some tests and benchmarks require the environment variable `FFXIV_GAME_DIR` to be set. By default, these are disabled
since they require a legitimate copy of the retail game data. These tests can be turned on via the `retail_game_testing`
feature.

## Contributing

The best way you can help is by [monetarily supporting me](https://redstrate.com/about/) or by submitting patches to help fix bugs or add functionality!
