# Raspberry Pi WS2811 controller written in rust.

Written entirely in Rust, including the web page, written with [Yew][1], and
compiled to webassembly. This is still under heavy development, please don't
attempt to use this unless you are comfortable filing bug reports or making
submitting pull-requests.

That said, pull requests are welcome.

## Requirements

The project requires the following dependencies installed on the build machine:

- `cargo-make`

## Running on the Raspberry Pi:

If you have your pi registered with the hostname `pilights.local` and the user
`pi`, you can just run `./execute.sh`. Otherwise you need to build it and copy
the files on your own. Or edit the execute script with your own values.

In order to build for ARM to run on the pi:

```sh
cargo make build_release_rpi
```

Then copy the files to the pi.

```sh
scp ./target/armv7-unknown-linux-gnueabihf/release/backend <raspberrypi>:~raspylights
scp ./frontend/index.html <raspberrypi>:~/frontend/index.html
scp -r ./frontend/pkg/ <raspberrypi>:~/frontend/pkg
scp -r ./frontend/static/ <raspberrypi>:~/frontend/static
```

You will also need a `db` directory for the LMDB database to get created.

```sh
ssh <raspberrypi> 'mkdir ~/db'
```

Then in order to run the script from the pi, run `sudo raspylights`, it requires
`sudo`, because we need access to the io pins

## Running locally:

Running the code on non arm architecture is possible, but the ws2811 lights will
not work. This will however still run the web-server with the preview of the
lights running in the top of the page

```sh
cargo make start
```

For building while editing and seeing changes to the webassembly live, in a
separate terminal, run:

```sh
cargo make watch
```

[1]: https://yew.rs/
