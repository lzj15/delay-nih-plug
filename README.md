# Delay nih-plug
A very simple delay effect audio plugin built using the [nih-plug](https://github.com/robbert-vdh/nih-plug) framework.
## Build
```bash
cargo install --git https://github.com/robbert-vdh/nih-plug.git cargo-nih-plug
git clone https://codeberg.org/lzj15/delay-nih-plug.git
cd delay-nih-plug
cargo nih-plug bundle delay-nih-plug --release
