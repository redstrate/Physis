# Contributing

Contributions of any kind is welcome, from code to reviews or filing an issue you found while using Physis.

## Pull Requests

PRs are welcome, from adding new formats to fixing bugs. Please ensure you:

* Run the test suite frequently with `cargo test`.
* Check lints using `cargo clippy`.
* Format your code before committing with `cargo fmt`.
* When adding new functionality, I highly encourage adding new test cases but this is not a hard requirement.

## Testing & Checks

We have a sizable test suite to ensure Physis doesn't regress in obvious ways. Not only that, but there's further checks to ensure we don't break semantic versioning and so on

### Unit Testing

Our standalone tests are run with `cargo test`. These tests are run automatically by the CI, and whoever broke them is expected to fix the failure.

Most of our tests use resources from `resources/tests`. **Test data should be recreatable with Physis, and not simply copied from the retail game**. If possible, you can zero out the irrelevant parts instead. (We do this for some `.mdl` files.)

### Retail Testing

I have a testing platform that tests Physis against multiple game versions. Currently it has to be manually run, and it's lacking a results web page. I don't expect you to keep this working until that's available.

### Patch Testing

The current patch testing code has bit-rotten, and was removed. It will be replaced with a better version eventually.

### Semantic Versioning and Dependency Checks

Physis uses `cargo deny` to check my dependencies for mismatched versions, deprecation warnings, updates and more. This is run on the CI, so you will know if you made it unhappy.

Making sure that the library is semver-compliant is important, and we use `cargo semver` for this. This is to ensure the API does not break when releasing new minor versions. This is run on the CI as well.

## Dependencies

Please try to keep our dependencies to the bare minimum. If you include a new dependency, there should be a clear benefit in doing so. For example: if we need to use gzip - we probably do _not_ want to implement that ourselves, so that's a justifiable dependency. But if there's a new variant of CRC (which might be 100 lines of code) please copy it into our source tree.

* Consumers of our library usually have their own set of large dependencies, and we don't want to make their compile times worse.
* Every new crate adds another failure point in terms of bugs.
* It's an additional 3rd party we have to trust.

If you absolutely must include a new dependency, their own dependency tree should be up-to-date. For example: don't make us (and our library consumers) compile _both_ Syn 1.x and 2.x.

Due to the complexity of compiling non-Rust libraries (this is from experience: InstallShield, Zlib, etc) refrain from using crates that compile their own C or C++ libraries by default. If we do need to do this, it needs to be well-tested across our supported target platforms.
