FROM ghcr.io/charmbracelet/vhs
RUN apt-get update && \
    apt-get install -y --no-install-recommends openssh-client git && \
    rm -rf /var/lib/apt/lists/*
