
install: target/release/jed
	cp target/release/jed ~/.local/bin

.PHONY: target/release/jed
target/release/jed:
	cargo build --release
