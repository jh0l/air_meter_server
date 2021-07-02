# Air Quality Sensor API Server for Raspberry Pi

*Get air quality readings over a local websocket connection.*  
<br />
  
# Development
## Cross compilation
### Prequisites
install `cross` (requires `docker`)
```
cargo install cross
```
inside `cross_deploy.sh` replace value of `TARGET_HOST` with `ssh` address of your device. Make sure the target and source paths are valid too.
  
Then run `cross_deploy.sh` to compile, `scp` the `server` binary, and run it over `ssh`.
