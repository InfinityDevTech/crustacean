
## init
```sh
# Install CLI dependency:
cargo install cargo-screeps

# Clone the starter
git clone https://github.com/rustyscreeps/screeps-starter-rust.git
cd screeps-starter-rust

# Copy the example config, and set up at least one deployment mode
cp example-screeps.toml screeps.toml
nano screeps.toml
# configure credentials (API key) if you'd like to upload directly,
# or a directory to copy to if you'd prepfer to use the game client to deploy

# build tool:
cargo screeps --help
# compile the module without deploying anywhere
cargo screeps build
# compile plus deploy to the configured 'upload' mode; any section name you
# set up in your screeps.toml for different environments and servers can be used
cargo screeps deploy -m upload
# or if you've set a default mode in your configuration, simply use:
cargo screeps deploy
```

[screeps]: https://screeps.com/
[`wasm-pack`]: https://rustwasm.github.io/wasm-pack/
[`cargo-screeps`]: https://github.com/rustyscreeps/cargo-screeps/
[`screeps-game-api`]: https://github.com/rustyscreeps/screeps-game-api/
[rustyscreeps]: https://github.com/rustyscreeps/
