<p align="center">
  <a href="https://gitpod.io/#https://github.com/gear-dapps/oracle" target="_blank">
    <img src="https://gitpod.io/button/open-in-gitpod.svg" width="240" alt="Gitpod">
  </a>
</p>

# Oracle

[![Build][build_badge]][build_href]
[![License][lic_badge]][lic_href]

[build_badge]: https://github.com/gear-dapps/oracle/workflows/Build/badge.svg
[build_href]: https://github.com/gear-dapps/oracle/actions/workflows/build.yml

[lic_badge]: https://img.shields.io/badge/License-MIT-success
[lic_href]: https://github.com/gear-dapps/oracle/blob/master/LICENSE

<!-- Description starts here -->

Example of centralized oracle which takes real-world off-chain data and submits immutable copy into blockchain space.

<!-- End of description -->

## Initial checklist after creating a new repo

- [x] Change app name in `Cargo.toml`
- [x] Fix Gitpod/badge/releases links in `README.md` (replace `gear-dapps/app` with `gear-dapps/<my-new-app>`)
- [x] Add description in `README.md`
- [ ] Fix dates, links, and initial commit hash in `CHANGELOG.md`
- [x] Remove this section

## Prebuilt Binaries

Raw, optimized, and meta WASM binaries can be found in the [Releases section](https://github.com/gear-dapps/app/releases).

## Building Locally

### ⚙️ Install Rust

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### ⚒️ Add specific toolchains

```shell
rustup toolchain add nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

... or ...

```shell
make init
```

### 🏗️ Build

```shell
cargo build --release
```

... or ...

```shell
make build
```

### ✅ Run tests

```shell
cargo test --release
```

... or ...

```shell
make test
```

### 🚀 Run everything with one command

```shell
make all
```

... or just ...

```shell
make
```

## License

The source code is licensed under [MIT license](LICENSE).
