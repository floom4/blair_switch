BIN=target/debug/blair_switch

build:
	cargo build

init:
	sudo scripts/init-topology host1 host2 host3 host4

run: build
	sudo scripts/host-exec sw ${BIN} if1-sw if2-sw if3-sw if4-sw

debug: build
	sudo scripts/host-exec sw "RUST_BACKTRACE=1 ${BIN} if1-sw if2-sw if3-sw if4-sw"
