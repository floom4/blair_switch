BIN=/app/blair_switch

build:
	cargo build

init:
	sudo containerlab deploy --reconfigure

run: build
	docker exec -it bs-lab-sw ${BIN} if1-sw if2-sw if3-sw if4-sw if5-sw

debug: build
	docker exec -it bs-lab-sw "RUST_BACKTRACE=1 ${BIN} if1-sw if2-sw if3-sw if4-sw if5-sw"

test:
	for test in $$( echo ./tests/*_tests.py ) ; do \
		echo "========== $${test} ================"; \
		$${test}; \
	done

.PHONY: build init run debug test
