build-darwin-arm64:
	@cargo build -p go_surtr --target aarch64-apple-darwin
	@cp ../../target/aarch64-apple-darwin/debug/libgo_surtr.a ./libgo_surtr_darwin_arm64.a

build-darwin-arm64-release:
	@cargo build -p go_surtr --target aarch64-apple-darwin --release
	@cp ../../target/aarch64-apple-darwin/release/libgo_surtr.a ./libgo_surtr_darwin_arm64.a

build-linux-arm64:
	@cargo build -p go_surtr --target aarch64-unknown-linux-gnu
	@cp ../../target/aarch64-unknown-linux-gnu/debug/libgo_surtr.a ./libgo_surtr_linux_arm64.a

build-linux-arm64-release:
	@cargo build -p go_surtr --target aarch64-unknown-linux-gnu --release
	@cp ../../target/aarch64-unknown-linux-gnu/release/libgo_surtr.a ./libgo_surtr_linux_arm64.a


build-linux-amd64:
	@cargo build -p go_surtr --target x86_64-unknown-linux-gnu
	@cp ../../target/x86_64-unknown-linux-gnu/debug/libgo_surtr.a ./libgo_surtr_linux_amd64.a

build-linux-amd64-release:
	@cargo build -p go_surtr --target x86_64-unknown-linux-gnu --release
	@cp ../../target/x86_64-unknown-linux-gnu/release/libgo_surtr.a ./libgo_surtr_linux_amd64.a


build: build-darwin-arm64 build-linux-arm64 build-linux-amd64

build-release: build-darwin-arm64-release build-linux-arm64-release build-linux-amd64-release