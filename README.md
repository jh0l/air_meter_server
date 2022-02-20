# Air Quality Sensor API Server for Raspberry Pi

_Over the Air Air Meters._
<br />

# Development
## Prerequisites
install `rustup` and `cargo` to use the rustlang platform

## Dev Frontend
The frontend is compiled statically from a nextjs app. During development you can run the nextjs development server for hotreloading. localhost:3000 is allowed through the API server's Cors setup for development.
```
npm run dev
```
By default the API server has a html template that loads the frontend javascript when available. You will need to build and pack the frontend to get interactivity in the site when accessing the site from the API server which is covered in `Prod Frontend` below. this will need to done before `cross_deploy`.

## Run Server
During development the sensor data will be placeholder values derived from unix time.
```
cargo run
```
Compiling on arm architecture will activate the production sensor code

## Cross compilation

### Prerequisites

install `cross` (requires `docker`)

```
cargo install cross
```

### Process

inside `cross_deploy.sh` replace value of `TARGET_HOST` with `ssh` address of
your device. Make sure the target and source paths are valid too.

Then run `cross_deploy.sh` to compile, `scp` the `server` binary, and run it
over `ssh`.

## Prod Frontend

Keep in mind you will need to build the frontend app.

### Prerequisites

install `npx`, `node` and `npm`

### Process

run the `pack_site.sh` script or follow its commands - whatever works for you

# TODO

## sensor client

-   [ ] sensor obtains publisher authorization id
-   [x] set `authorization` header to id - connect to webserver websocket
-   [x] on loop send sensor data and detect wether to reinitialise ccs811 assign
        to context property
-   [x] seperate thread that can delete and reinitialise sensor thread context
        property based on http GET request

## web server

-   [x] on startup start sensor thread
-   [x] setup actix server
-   [x] receive air_meter_node requests via websockets
    -   [ ] note - provision for db based config response for node (increment,
            restart)
-   [x] support websocket requests for air_meter
-   [x] save air_meter readings to db accessable by users
-   [x] serve web_client requests with Askama template
-   [ ] serve web_client with template that requests react_app
-   [ ] add to system startup (singleton)
-   [ ] adjustable reading increment
-   [ ] visually indicate sensor warmup based on sensor uptime
-   [ ] change heartbeat to ~30 minutes - then indicate sensor client may have
        crashed based no heartbeat from client after 5 minutes

## wifi hotspot config network (Captive Portal)

-   [ ] in factory config rasp pi starts with wifi hotspot with webpage for
        config
    -   [ ] user joins rasp pi wifi - navigates to config webpage
    -   [ ] enter wifi password - rasp pi will have to terminate hotspot then
            test wifi password
    -   [ ] sensor client obtains authorization id and server address?
    -   [ ] use websockets to notify of success with ip of rasp pi on user's
            wifi (user will have to join orig wifi manually)

## Captive Portal mode for sensor operation webapp

-   user can choose to access sensor app through captive portal instead of
    through existing local network
    -   mode can be chosen at config network or at sensor app
    -   https://raspberrypi.stackexchange.com/a/100118

## Battery Power
-   Display Battery levels and estimated lifetime for battery powered device
-   save energy https://core-electronics.com.au/tutorials/disable-features-raspberry-pi.html
