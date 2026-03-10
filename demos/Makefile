.PHONY: run

run: build
	clear
	@podman run -it --rm --hostname memy-demo memy-demo bash

build:
	podman build -f demos/Dockerfile -t memy-demo ..
