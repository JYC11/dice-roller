release-m1:
	cargo build --release

release-windows:
	cargo build --release --target x86_64-pc-windows-gnu
