# Dockerfile para el backend como servicio web
FROM ubuntu:22.04

# Instalar dependencias
RUN apt-get update && apt-get install -y curl build-essential pkg-config libssl-dev

# Instalar Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:$PATH"

WORKDIR /app

# Copiar Cargo.toml y construir dependencias primero (para cache)
COPY src-tauri/Cargo.toml src-tauri/Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src target/release/deps/tauri_app_fodorian*

# Copiar código fuente
COPY src-tauri/ .

# Construir el backend
RUN cargo build --release

# Exponer puerto para el servidor web
EXPOSE 8080

# Ejecutar el binario
CMD ["./target/release/tauri-app-fodorian"]