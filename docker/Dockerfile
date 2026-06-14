# Pre-create ~/.conda/express owned by nonroot so Docker named volumes inherit
# correct ownership instead of defaulting to root.
FROM busybox:stable@sha256:73aaf090f3d85aa34ee199857f03fa3a95c8ede2ffd4cc2cdb5b94e566b11662 AS prep
RUN mkdir -p /home/nonroot/.conda/express && chown 65532:65532 /home/nonroot/.conda/express

# Pin by digest for reproducible builds (update via Renovate/Dependabot)
FROM gcr.io/distroless/cc-debian12:nonroot@sha256:b0ae8e989418b458e0f25489bc3be523718938a2b70864cc0f6a00af1ddbd985
ARG TARGETARCH
COPY --from=prep --chown=nonroot:nonroot /home/nonroot/.conda/express /home/nonroot/.conda/express
COPY cx-linux-${TARGETARCH} /usr/local/bin/cx
ENV HOME=/home/nonroot
ENTRYPOINT ["/usr/local/bin/cx"]
