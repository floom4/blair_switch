BIN=/app/blair_switch
VENV=.env
PYTHON=$(VENV)/bin/python3
PYTEST=$(VENV)/bin/pytest

build:
	cargo build

init:
	mkdir -p target/debug
	sudo containerlab deploy --reconfigure
	python -m venv $(VENV)
	$(PYTHON) -m pip install -r requirements.txt

run: build
	docker exec -it bs-lab-sw ${BIN} if1-sw if2-sw if3-sw if4-sw if5-sw

debug: build
	docker exec -it bs-lab-sw "RUST_BACKTRACE=1 ${BIN} if1-sw if2-sw if3-sw if4-sw if5-sw"

clean:
	$(RM) -vrf target/

dist-clean: clean
	sudo containerlab destroy
	$(RM) -vrf $(VENV) clab-bs-lab/

test: build
	@if [ ! -d "$(VENV)" ]; then \
		echo "Error: Virtual environment not found at '$(VENV)'."; \
		echo "Please run 'make init' to create it"; \
		exit 1; \
	fi
	$(PYTEST)

.PHONY: build init run debug test clean dist-clean
