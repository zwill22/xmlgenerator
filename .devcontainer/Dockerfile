FROM mcr.microsoft.com/devcontainers/rust

RUN apt-get update
RUN apt-get install git curl build-essential cmake libclang-dev rustup libssl-dev zsh pkg-config libxml2-dev -y --no-install-recommends

# Setup UV
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="$PATH:/root/.local/bin/" 
