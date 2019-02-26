# # ------------------------------------------------------------------------------
# # Cargo Build Stage
# # ------------------------------------------------------------------------------

# FROM rustlang/rust:nightly as cargo-build

# WORKDIR /usr/src/ifttt-v0

# COPY Cargo.toml Cargo.toml

# RUN mkdir src/

# RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs

# RUN cargo build --release

# RUN rm -f target/release/deps/ifttt-v0*

# COPY . .

# RUN cargo build --release

# RUN cargo install --path .

# # ------------------------------------------------------------------------------
# # Final Stage
# # ------------------------------------------------------------------------------

# FROM alpine:latest

# COPY --from=cargo-build /usr/local/cargo/bin/ifttt-v0 /usr/local/bin/ifttt-v0

# CMD ["ifttt-v0"]


FROM rustlang/rust:nightly

WORKDIR /usr/src/ifttt-v0
COPY . .

EXPOSE 80

RUN cargo install --path .

CMD ["ifttt-v0"]