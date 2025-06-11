# Build stage
FROM ubuntu:24.04 AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    libssl-dev \
    pkg-config \
    git \
    cmake \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Set working directory for the build
WORKDIR /build

# Copy the entire project
COPY . .

# Build only the specific binary
RUN cargo build --release --features=load_dev_certs --bin dev_client

# Runtime stage
FROM ubuntu:24.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
RUN mkdir "/app"

# Copy the binary from builder stage
COPY --from=builder /build/target/release/dev_client /app/dev_client

ENV RUST_LOG="info"

# Make the binary executable
RUN chmod +x /app/dev_client

# Create simple wrapper script
RUN echo '#!/bin/bash' > /app/wrapper.sh && \
    echo 'while true; do' >> /app/wrapper.sh && \
    echo '  ./dev_client' >> /app/wrapper.sh && \
    echo '  sleep 1' >> /app/wrapper.sh && \
    echo 'done' >> /app/wrapper.sh && \
    chmod +x /app/wrapper.sh

# Set working directory
WORKDIR /app

# Run the wrapper script
CMD ["./wrapper.sh"]