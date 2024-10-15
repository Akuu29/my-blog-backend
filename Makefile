.PHONY: dev local-container-run login-ghcr build-% push-% deploy-%

dev:
	cargo watch -x run

local-container-run:
	docker build --target stg -f Docker/rust/Dockerfile -t local-my-blog-backend .
	docker run --network my-blog-migration-tools_default -p 8080:80 --env-file .env local-my-blog-backend

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
