TARGET_DIR := target
STUB_DIR := $(TARGET_DIR)/stubs

dev: compile-engine-dev build-py-client-dev 
release: compile-engine-release build-py-client-release

setup-deps:
	pip install -r requirements.txt

compile-engine-dev:
	cargo build

compile-engine-release: setup-deps
	cargo build --release

build-py-client-dev:
	cd crates/pyclient && maturin develop
	mkdir -p $(STUB_DIR)
	pyo3-stubgen pyclient $(STUB_DIR)

build-py-client-release: setup-deps
	cd crates/pyclient && maturin build --release
	python3 -m pip install --force-reinstall $(TARGET_DIR)/wheels/pyclient-*.whl
	mkdir -p $(STUB_DIR)
	pyo3-stubgen pyclient $(STUB_DIR)
