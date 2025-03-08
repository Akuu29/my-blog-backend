.PHONY: dev run-local-container login-ghcr build-% push-% deploy-%

dev:
	RUST_LOG=debug \
	MASTER_KEY='${shell openssl rand -base64 64}' \
	cargo watch -x run

run-local-container:
	docker network create my-blog-network || true
	docker compose build
	docker compose up

login-ghcr:
	echo ${GHCR_PAT} | docker login ghcr.io -u ${GHCR_USER} --password-stdin

build-%:
	@$(if $(filter $*, dev stg),,$(error Invalid STAGE $*))
	docker build --platform linux/amd64 --target ${*} -f Docker/rust/Dockerfile -t ghcr.io/${GHCR_USER}/my-blog-backend/${*}-my-blog-web-api:latest .

push-%:
	@$(if $(filter $*, stg),,$(error Invalid STAGE $*))
	docker push ghcr.io/${GHCR_USER}/my-blog-backend/${*}-my-blog-web-api:latest

deploy-%: login-ghcr
	@$(if $(filter $*, stg),,$(error Invalid STAGE $*))
	$(MAKE) build-$*
	$(MAKE) push-$*
