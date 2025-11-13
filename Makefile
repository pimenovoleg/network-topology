.PHONY: help build test clean format

dev-daemon:
	cargo run --bin daemon -- --server-target http://127.0.0.1 --server-port 60072 --log-level debug


setup-db:
	@echo "Setting up PostgreSQL..."
	@docker run -d \
		--name netvisor-postgres \
		-e POSTGRES_USER=postgres \
		-e POSTGRES_PASSWORD=password \
		-e POSTGRES_DB=netvisor \
		-p 5432:5432 \
		postgres:17-alpine || echo "Already running"
	@sleep 3
	@echo "PostgreSQL ready at localhost:5432"


clean-db:
	docker stop netvisor-postgres || true
	docker rm netvisor-postgres || true

clean-daemon:
	rm -rf ~/Library/Application\ Support/com.netvisor.daemon

dev-server:
	@export DATABASE_URL="postgresql://postgres:password@localhost:5432/netvisor" && \
	cargo run --bin server -- --log-level debug --integrated-daemon-url http://localhost:60073

dev-container:
	docker compose -f docker-compose.dev.yml up