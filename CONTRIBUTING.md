# Contributing

If you're interested to read how these formats work in more detail, see [xiv.dev](https://xiv.dev/) and
[docs.xiv.zone](https://docs.xiv.zone).

## Testing

One of the main goals of Physis is to avoid accidental regressions, this is especially important when handling game
data that might take hours to download.

### Unit Testing

There are a set of basic unit tests you can run via `cargo test`. You can also find the relevant test resources in `resources/tests`.
This does **NOT** contain copyrighted material, but actually fake game data created by Physis itself. These tests are
run automatically by the CI.

### Retail Testing

There are some tests and benchmarks require the environment variable `FFXIV_GAME_DIR` to be set. By default, these are disabled
since they require a legitimate copy of the retail game data. These tests can be turned on via the `retail_testing`
feature.

I have a testing platform that tests Physis against multiple game versions. Currently it has to be manually run, and it's lacking a results web page.

### Patch Testing

Patching is an extremely sensitive operation since it is not easily reversible if done wrong. Repairing the game files
is an option, but it's time-consuming and not yet implemented in Physis. To prevent regressions in patching the
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

### Semver and Dependency Checks

Even though package management in Rust is easier, it's a double-edged sword. I try to prevent getting carried away
from including crates - but the ones we do include, have to get checked often. I use `cargo deny` to check my
dependencies for mismatched versions, deprecation warnings, updates and more. This is also run on the CI!

Making sure that the library is semver-compliant is also important, and I use `cargo semver` for this task. This is to ensure the API does not break when moving between patch
versions.
