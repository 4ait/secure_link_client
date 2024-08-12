rustup update && \
cargo install cross --git https://github.com/cross-rs/cross && \
mkdir -p target/secure_link_releases/win64 && \
mkdir -p target/secure_link_releases/linux64 && \
cross build --target x86_64-pc-windows-gnu --release --features aws-lc-sys && \
cp target/x86_64-pc-windows-gnu/release/secure_link.exe target/secure_link_releases/win64/secure_link.exe &&\
cross build --target aarch64-unknown-linux-gnu --release && \
cp target/aarch64-unknown-linux-gnu/release/secure_link target/secure_link_releases/linux64/secure_link

