build:
	docker compose build

run:
	docker compose up

dev:
	sqlx db create
	sqlx migrate run
	cargo watch -x run