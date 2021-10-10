BeaconShip
==========

## Beacon

To run and serve as the beacon,

```powershell
$env:RUST_LOG="beacon=debug"; $env:ROCKET_PROFILE="release"; .\target\debug\beacon -a tesat -r 323c 23bd
```

To change the configuration used by [Rocket](https://api.rocket.rs/master/rocket/config/struct.Config.html) (the http server), modify `Rocket.toml`.