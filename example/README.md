## Run example on windows/linux

```bash
cd example
cargo run
```

## Run example on android

1. install xbuild
   ```bash
   cargo install xbuild
   ```
2. run the example
   ```bash
   # get device id
   x devices # let's say the device id is `adb:823c4f8b`
   x run -p example --arch arm64 --device adb:823c4f8b
   ```
3. optional: build the apk
   ```bash
   x build -p example --platform android --arch arm64 --format apk
   ```
