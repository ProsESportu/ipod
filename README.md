# ipod

ipod is a Rust userspace library implementation of the classic iPod Accessory Protocol.
It includes an example client for use with https://github.com/oandrew/ipod-gadget

This is a total rewrite of what was included with the  ipod-gadget project. 
It should work as a drop-in replacement for the old app.

New features:
- Storing and replaying traces
- Detailed verbose logging for debug
- Better codebase with message type definitions
- Tests

### update 03/2020
kernel module needs to be recompiled due to a breaking change (hid descriptor).  
This should fix the issue with hanging after `GetDevAuthenticationInfo` on some devices.  
At least it's finally working in my own car :)

# build and run
```
cargo build --release
# or cross compiling, after installing a Rust target/toolchain for the device
cargo build --release --target arm-unknown-linux-gnueabihf

# with debug logging
./target/release/ipod -d serve /dev/iap0

# save a trace file
./target/release/ipod -d serve -w ipod.trace /dev/iap0

# simulate incoming requests from a trace file
./target/release/ipod -d replay ./ipod.trace

# view a trace file
./target/release/ipod -d view ./ipod.trace

# replay accessory requests from a trace file to a device
./target/release/ipod -d send /dev/iap0 ./ipod.trace

# run tests
cargo test

```

Refer to https://github.com/oandrew/ipod-gadget for more info on how to get the kernel part working.




