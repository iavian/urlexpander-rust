FROM rust:1.62

COPY ./ ./

# Build your program for release
RUN cargo build --release

# Run the binary
CMD ["./target/release/url-resolver"]