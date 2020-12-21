FROM gcr.io/distroless/cc

COPY ./target/x86_64-unknown-linux-musl/release/backend /
COPY ./dist /static

ENV DIST_DIR="/static"

CMD ["./backend"]
