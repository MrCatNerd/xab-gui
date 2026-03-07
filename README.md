# xab-gui

<img src="res/logo.webp" alt="logo" style="width:25em;"/>

xab gui in rust

> [!WARNING]
> this is experimental and WIP! don't expect anything to work

## Instructions
1. compile xab with the experimental flag set to true:
```sh
# or when setting up
meson setup build -Dexperimental=true

# if already set up
meson configure build -Dexperimental=true
```

2. run xab with the --ipc=1 flag:
```sh
xab --ipc=1
```

3. run xab-gui and press connect
```sh
cargo run
```
