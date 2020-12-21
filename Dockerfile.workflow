FROM gcr.io/distroless/cc

COPY ./target/release/backend /
COPY ./dist /static

ENV DIST_DIR="/static"

CMD ["./backend"]
