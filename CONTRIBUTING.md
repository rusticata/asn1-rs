# Contributing <!-- omit in toc -->

First of all, thank you for contributing to `asn1-rs`! The goal of this document is to provide everything you need to know in order to contribute to `asn1-rs` and its different integrations.

- [Assumptions](#assumptions)
- [How to Contribute](#how-to-contribute)
- [Development Workflow](#development-workflow)
- [Git Guidelines](#git-guidelines)
- [Release Process (for internal team only)](#release-process-for-internal-team-only)


## Assumptions

1. **You're familiar with [GitHub](https://github.com) and the [Pull Request](https://help.github.com/en/github/collaborating-with-issues-and-pull-requests/about-pull-requests)(PR) workflow.**
2. **You have some familiarity with ASN.1 BER and DER encodings (X.690)**

## How to Contribute

1. Make sure that the contribution you want to make is explained or detailed in a GitHub issue! Find an [existing issue](https://github.com/rusticata/asn1-rs/issues/) or [open a new one](https://github.com/rusticata/asn1-rs/issues/new).
2. Once done, [fork the asn1-rs repository](https://help.github.com/en/github/getting-started-with-github/fork-a-repo) in your own GitHub account. Ask a maintainer if you want your issue to be checked before making a PR.
3. [Create a new Git branch](https://help.github.com/en/github/collaborating-with-issues-and-pull-requests/creating-and-deleting-branches-within-your-repository).
4. Review the [Development Workflow](#development-workflow) section that describes the steps to maintain the repository.
5. Make the changes on your branch.
6. [Submit the branch as a PR](https://help.github.com/en/github/collaborating-with-issues-and-pull-requests/creating-a-pull-request-from-a-fork) pointing to the `master` branch of the main asn1-rs repository. A maintainer should comment and/or review your Pull Request within a few days. Although depending on the circumstances, it may take longer.<br>
 We do not enforce a naming convention for the PRs, but **please use something descriptive of your changes**, having in mind that the title of your PR will be automatically added to the next [release changelog](https://github.com/rusticata/asn1-rs/releases/).

## Development Workflow

You can set up your local environment natively.

To install dependencies:

```bash
cargo build --release
```

To ensure the same dependency versions in all environments, for example the CI, update the dependencies by running: `cargo update`.

### Tests <!-- omit in toc -->

To run the tests, run:

```bash
# Tests
cargo test --all-features # or --no-default-features, or other flags
```

There are two kinds of tests, documentation tests and unit tests.

Unit tests should be added for any feature request of bug fix, adding some test data (the `hex-literal` crate is used to embed hex data in the unit tests).

Each PR should pass the tests to be accepted.

### Clippy <!-- omit in toc -->

Each PR should pass [`clippy`](https://github.com/rust-lang/rust-clippy) (the linter) to be accepted.

```bash
cargo clippy -- -D warnings
```

If you don't have `clippy` installed on your machine yet, run:

```bash
rustup update
rustup component add clippy
```

⚠️ Also, if you have installed `clippy` a long time ago, you might need to update it:

```bash
rustup update
```

### Fmt

Each PR should pass the format test to be accepted.

Run the following to fix the formatting errors:

```
cargo fmt
```

and the following to test if the formatting is correct:
```
cargo fmt --all -- --check
```

### Update the README <!-- omit in toc -->

The README is generated. Please do not update manually the `README.md` file.

Instead, update the `README.tpl` and `src/lib.rs` files, and run:

```sh
cargo install cargo-rdme # install cargo-rdme if needed
cargo rdme
```

Then, push the changed files.

You can check the current `README.md` is up-to-date by running:

```sh
cargo rdme -c
```

If it's not, the CI will fail on your PR.

## Git Guidelines

### Git Branches <!-- omit in toc -->

All changes must be made in a branch and submitted as PR.
We do not enforce any branch naming style, but please use something descriptive of your changes.

### Git Commits <!-- omit in toc -->

As minimal requirements, your commit message should:
- be capitalized
- not finished by a dot or any other punctuation character (!,?)
- start with a verb so that we can read your commit message this way: "This commit will ...", where "..." is the commit message.
  e.g.: "Fix the home page button" or "Add more tests for create_index method"

We don't follow any other convention, but if you want to use one, we recommend [this one](https://chris.beams.io/posts/git-commit/).

In the future, a tool like [git-cliff](https://git-cliff.org/) may be used to either generate the full changelog (in the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format), or only the base file.

### GitHub Pull Requests <!-- omit in toc -->

Some notes on GitHub PRs:

- [Convert your PR as a draft](https://help.github.com/en/github/collaborating-with-issues-and-pull-requests/changing-the-stage-of-a-pull-request) if your changes are a work in progress: no one will review it until you pass your PR as ready for review.<br>
  The draft PR can be very useful if you want to show that you are working on something and make your work visible.
- The branch related to the PR must be **up-to-date with `master`** before merging.
- All PRs must be reviewed and approved.
- The PR title should be accurate and descriptive of the changes. The title of the PR will be indeed automatically added to the next [release changelogs](https://github.com/rusticata/asn1-rs/releases/).

## Release Process (for the internal team only)

`asn1-rs` tools follow the [Semantic Versioning Convention](https://semver.org/).

### How to Publish the Release <!-- omit in toc -->

Make a PR modifying the file [`Cargo.toml`](/Cargo.toml):

```toml
version = "X.X.X"
```

Verify that the current version does not change the MSRV (CI will check that). If the MSRV needs updating, the following files need to be changed:<br>
- `Cargo.toml` (`rust-version`)
- `src/lib.rs`
- `README.md` (badge)
- `.github/workflows/rust.yml` (test matrix)


After the changes on `lib.rs`, run the following command:

```bash
cargo rdme
```

Once the changes are merged on `main`, you can publish the current draft release via the [GitHub interface](https://github.com/rusticata/asn1-rs/releases): on this page, click on `Edit` (related to the draft release) > update the description > when you are ready, click on `Publish release`.

Publish on [crates.io](https://crates.io/crates/asn1-rs):
```sh
cargo package # test local package
cargo publish
```

Once released, a signed tag must be added and pushed to git with the released version:
```sh
git tag -s asn1-rs-0.7.1
git push --tags
```

<hr>

Thank you again for reading this through. We cannot wait to begin to work with you if you make your way through this contributing guide ❤️

Thanks to Meilisearch for the original `CONTRIBUTING.md` file.
