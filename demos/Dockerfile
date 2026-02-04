FROM rust:latest
RUN apt-get update && apt-get install -y \
    fzf \
    git \
    neovim \
    sqlite3

RUN useradd -m demo
USER demo
WORKDIR /home/demo
ENV HOME=/home/demo

ARG CACHE_BUST
RUN git clone https://github.com/andrewferrier/memy.git
WORKDIR /home/demo/memy
RUN cargo build

# Dummy directories and files
RUN mkdir -p /home/demo/docs /home/demo/projects/my_project /home/demo/tmp && \
    echo "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua." > /home/demo/docs/report.txt && \
    echo "Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat." > /home/demo/docs/notes.md && \
    echo "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur." > /home/demo/projects/my_project/README.md && \
    echo "int main() { printf(\"Hello, World!\"); return 0; }" > /home/demo/projects/my_project/main.c && \
    head -c 1K </dev/urandom > /home/demo/tmp/binary_file.bin && \
    echo "Temporary log file content." > /home/demo/tmp/temp_log.txt

ENV PATH="/home/demo/memy/target/debug:${PATH}"
RUN echo "export PATH=/home/demo/memy/target/debug:\$PATH" >> ~/.bashrc
