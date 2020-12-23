# THIS FILE IS ONLY FOR THE WORKFLOW CI
# DO NOT USE IT UNLESS YOU KNOW WHAT YOU'RE DOING

FROM gcr.io/distroless/cc

COPY ./artifacts/backend /
COPY ./dist /static

ENV DIST_DIR="/static"

CMD ["./backend"]
