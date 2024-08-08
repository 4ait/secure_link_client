rustup update && \
cargo install cross --git https://github.com/cross-rs/cross && \
cross build --target aarch64-unknown-linux-gnu --release && \
cross build --target x86_64-pc-windows-gnu --release --features aws-lc-sys && \
mkdir -p target/secure_link_releases/linux64 && \
mkdir -p target/secure_link_releases/win64 && \
cp target/aarch64-unknown-linux-gnu/release/secure_link target/secure_link_releases/linux64/secure_link && \
cp target/x86_64-pc-windows-gnu/release/secure_link.exe target/secure_link_releases/win64/secure_link.exe