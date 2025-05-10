# Contributing

If you're interested to read how these formats work in more detail, see [xiv.dev](https://xiv.dev/) and
[docs.xiv.zone](https://docs.xiv.zone).

## Pull Requests

PRs are welcome, from adding new formats to fixing bugs. Please ensure you:

* Run the test suite frequently with `cargo test`.
* Check lints using `cargo clippy`.
* Format your code before committing with `cargo fmt`.

## Testing

We have several tests to ensure Physis is able to read and write the game's various formats.

### Unit Testing

Our standalone tests are run with `cargo test`. You can find the relevant resources that it reads under `resources/tests`. Test data should be recreatable with Physis, and not simply copied from the retail game. These tests are run automatically by the CI, and you are expected to keep them working. (Unless the test itself is wrong, of course!)

When adding new functionality, I highly encourage adding new test cases but this is not a hard requirement.

### Retail Testing

I have a testing platform that tests Physis against multiple game versions. Currently it has to be manually run, and it's lacking a results web page. I don't expect you to keep this working until that's available.

### Patch Testing

The current patch testing code has bit-rotten, and was removed. It will be replaced with a better version eventually.

## Dependencies

Please keep our dependencies to the bare minimum. If you include a new dependency then there should be a clear benefit in doing so. For example, if there is a new format that's uses gzip - we do _not_ want to implement our own decompression algorithm. But if there's a new variant of CRC - which might only take 100 lines of Rust code - please just copy it into our source tree.

If you absolutely must include a new dependency, it's own dependencies should be up-to-date. Don't make Physis (and by extension, any library consumers) compile _both_ Syn 1 & 2 for example.

We want to keep dependencies to a minimum because:
* Consumers of our library usually have their own set of large dependencies, and we don't want to make their compile times worse.
* Every new crate adds another failure point in terms of bugs.
* It's an additional 3rd party we have to trust.

## Semantic Versioning and Dependency Checks

Physis uses `cargo deny` to check my dependencies for mismatched versions, deprecation warnings, updates and more. This is also run on the CI, so you will know if you made it unhappy.

Making sure that the library is semver-compliant is important, and we use `cargo semver` for this. This is to ensure the API does not break when releasing new minor versions.
