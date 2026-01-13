# Tessera UI Framework Example

This example demonstrates the capabilities of the Tessera UI framework with a comprehensive component showcase featuring

## Run example on windows/linux

```bash
cd example
cargo tessera dev
```

## Run example on android

1. make sure Android SDK/NDK are installed and `adb` is available in PATH

2. run the example

   ```bash
   # get device id
   adb devices # let's say the device id is `8cd1353b`
   cargo tessera android dev --device 8cd1353b
   ```

3. optional: build the apk

   ```bash
   cargo tessera android build --format apk
   ```
