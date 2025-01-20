# Builder stage
FROM rustlang/rust:nightly AS builder

WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Runtime stage
FROM rustlang/rust:nightly AS runtime

WORKDIR /app
# Copy the compiled binary from the builder environment
# to our runtime environment
COPY --from=builder /app/target/release/zero2prod zero2prod
# We need the configuration
COPY configuration configuration
ENV APP_ENVIRONMENT=production
# When `docker run` is executed, launch the binary!
ENTRYPOINT ["./zero2prod"]
