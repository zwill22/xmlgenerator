FROM rust:latest
LABEL authors="zmwilliams"

RUN apt update
RUN apt install -y cmake clang

# Setup UV
RUN cargo install --locked uv

ENTRYPOINT ["top", "-b"]
