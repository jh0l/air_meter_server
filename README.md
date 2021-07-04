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
  
# TODO
## sensor client
- [ ] sensor obtains publisher authorization id
- [x] set `authorization` header to id - connect to webserver websocket
- [x] on loop send sensor data and detect wether to reinitialise ccs811 assign to context property
- [x] seperate thread that can delete and reinitialise sensor thread context property based on http GET request
## web server
- [x] on startup start sensor thread
- [x] setup actix server
- [x] receive air_meter_node requests via websockets
  - [ ]  note - provision for db based config response for node (increment, restart)
- [x] support websocket requests for air_meter
- [ ] save air_meter readings to db accessable by users
- [ ] serve web_client requests with Askama template
- [ ] serve web_client with template that requests react_app
- [ ] add to system startup (singleton)
- [ ] adjustable reading increment
- [ ] visually indicate sensor warmup based on sensor uptime
- [ ] change heartbeat to ~30 minutes - then indicate sensor client may have crashed based no heartbeat from client after 5 minutes
## wifi hotspot config network
- [ ] in factory config rasp pi starts with wifi hotspot with webpage for config
  - [ ] user joins rasp pi wifi - navigates to config webpage
  - [ ] enter wifi password - rasp pi will have to terminate hotspot then test wifi password
  - [ ] sensor client obtains authorization id and server address? 
  - [ ] use websockets to notify of success with ip of rasp pi on user's wifi (user will have to join orig wifi manually)
