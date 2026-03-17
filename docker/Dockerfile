# Pin by digest for reproducible builds (update via Renovate/Dependabot)
FROM gcr.io/distroless/cc-debian12:nonroot@sha256:7e5b8df2f4d36f5599ef4ab856d7d444922531709becb03f3368c6d797d0a5eb
ARG TARGETARCH
COPY cx-linux-${TARGETARCH} /usr/local/bin/cx
ENTRYPOINT ["/usr/local/bin/cx"]
