BeaconShip
==========

## Beacon

To run and serve as the beacon,

```powershell
$env:RUST_LOG="beacon=debug"; $env:ROCKET_PROFILE="release"; .\target\debug\beacon -a tesat -r 323c 23bd
```

To change the configuration used by [Rocket](https://api.rocket.rs/master/rocket/config/struct.Config.html) (the http server), modify `Rocket.toml`.

# Build

## armv7

This works for old device that runs on `glibc>=2.15` and old linux kernels like `~3.10`

```bash
cargo install cross
docker build . -t cross-openssl:latest
OPENSSL_STATIC=0 OPENSSL_LIB_DIR=/opt/openssl/lib \
    OPENSSL_INCLUDE_DIR=/opt/openssl/include \
    cross build --target armv7-unknown-linux-gnueabihf 
```

## aarch64

On Ubuntu 20.04,

```bash
rustup target add aarch64-unknown-linux-gnu
sudo apt-get install gcc-9-aarch64-linux-gnu
# if building openssl
CC=aarch64-linux-gnu-gcc-9 ./config --prefix=/opt/openssl linux-aarch64
# then build with cargo
OPENSSL_STATIC=1 OPENSSL_LIB_DIR=/opt/openssl/lib \
    OPENSSL_INCLUDE_DIR=/opt/openssl/include \
    cargo build --target aarch64-unknown-linux-gnu
```