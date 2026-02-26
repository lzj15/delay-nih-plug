#!/usr/bin/sh
cargo build --release

cc -shared -o target/delay-nih-plug.so /usr/lib/libc.a -lunwind \
-Wl,--whole-archive target/release/libdelay_nih_plug.a

mkdir -p ~/.vst3/delay-nih-plug.vst3/Contents/x86_64-linux/
cp target/delay-nih-plug.so ~/.vst3/delay-nih-plug.vst3/Contents/x86_64-linux/delay-nih-plug.so
cp target/delay-nih-plug.so ~/.clap/delay-nih-plug.clap
