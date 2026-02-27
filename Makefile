run:
	bun run tauri dev

build:
	bun run tauri build
clean:
	cd src-tauri && cargo clean
test:
	cd src-tauri && cargo test -- --test-threads=1
bun-check:
	bun run check
cargo-check:
	cargo check
