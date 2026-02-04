.PHONY: run

CACHE_BUST ?= $(shell date +%s)

run:
	podman build -t memy-demo \
		--build-arg CACHE_BUST=$(CACHE_BUST) \
		.
	clear
	@podman run -it \
		--rm \
		--hostname memy-demo \
		memy-demo bash
