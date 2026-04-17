compile-engine:
	cargo build --release

compile-py-module:
	cd crates/pyclient && maturin build --release

build:
	compile-engine
	compile-py-module