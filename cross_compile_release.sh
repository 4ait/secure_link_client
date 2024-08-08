rustup update && \
cargo install cross --git https://github.com/cross-rs/cross && \
cross build --target aarch64-unknown-linux-gnu --release && \
cross build --target x86_64-pc-windows-gnu --release --features aws-lc-sys && \
mkdir -p target/secure_link_release/linux64 && \
mkdir -p target/secure_link_release/win64 && \
cp target/aarch64-unknown-linux-gnu/release/secure_link target/release_build/linux64/secure_link && \
cp target/x86_64-pc-windows-gnu/release/secure_link.exe target/release_build/win64/secure_link.exe