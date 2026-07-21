# Pre-create ~/.conda/express owned by nonroot so Docker named volumes inherit
# correct ownership instead of defaulting to root.
FROM busybox:stable@sha256:73aaf090f3d85aa34ee199857f03fa3a95c8ede2ffd4cc2cdb5b94e566b11662 AS prep
RUN mkdir -p /home/nonroot/.conda/express && chown 65532:65532 /home/nonroot/.conda/express

# Pin by digest for reproducible builds (update via Renovate/Dependabot)
FROM gcr.io/distroless/cc-debian12:nonroot@sha256:fccdbb0a547c14e23fcf4ce8ad62ca5d43b4faae8d22cd292f490fef9946c96e
ARG TARGETARCH
COPY --from=prep --chown=nonroot:nonroot /home/nonroot/.conda/express /home/nonroot/.conda/express
COPY cx-linux-${TARGETARCH} /usr/local/bin/cx
ENV HOME=/home/nonroot
ENTRYPOINT ["/usr/local/bin/cx"]
