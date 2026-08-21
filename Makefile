.PHONY: build run test lint fmt test_new_client

build:
	cargo build

run: build
	cargo run

test:
	cargo test

fmt:
	cargo fmt

lint:
	cargo clippy --all-targets -- -D warnings

test_new_client: build
	@./target/debug/prex-challenge & PID=$$!; sleep 2; \
	bash scripts/new_client.sh; rc=$$?; \
	kill $$PID; exit $$rc

test_new_credit_tx: build
	@./target/debug/prex-challenge & PID=$$!; sleep 2; \
	bash scripts/new_credit_tx.sh; rc=$$?; \
	kill $$PID; exit $$rc

test_new_debit_tx: build
	@./target/debug/prex-challenge & PID=$$!; sleep 2; \
	bash scripts/new_debit_tx.sh; rc=$$?; \
	kill $$PID; exit $$rc

test_client_balance: build
	@./target/debug/prex-challenge & PID=$$!; sleep 2; \
	bash scripts/client_balance.sh; rc=$$?; \
	kill $$PID; exit $$rc
