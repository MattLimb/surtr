GO_WORKING_DIR := crates/go_surtr


build-go-darwin:
	@cargo build -p go_surtr --target aarch64-apple-darwin

	@cp ./target/aarch64-apple-darwin/debug/libgo_surtr.a ./$(GO_WORKING_DIR)/libgo_surtr_darwin_arm64.a


build-go-linux:
	@cargo build -p go_surtr --target aarch64-unknown-linux-gnu
	@cargo build -p go_surtr --target x86_64-unknown-linux-gnu

	# Copy the results into the working directory
	@cp ./target/aarch64-unknown-linux-gnu/debug/libgo_surtr.a ./$(GO_WORKING_DIR)/libgo_surtr_linux_arm64.a
	@cp ./target/x86_64-unknown-linux-gnu/debug/libgo_surtr.a ./$(GO_WORKING_DIR)/libgo_surtr_linux_amd64.a


build-go-darwin-release:
	@cargo build -p go_surtr --target aarch64-apple-darwin --release

	@cp ./target/aarch64-apple-darwin/release/libgo_surtr.a ./$(GO_WORKING_DIR)/libgo_surtr_darwin_arm64.a


build-go-linux-release:
	@cargo build -p go_surtr --target aarch64-unknown-linux-gnu --release
	@cargo build -p go_surtr --target x86_64-unknown-linux-gnu --release

	# Copy the results into the working directory
	@cp ./target/aarch64-unknown-linux-gnu/release/libgo_surtr.a ./$(GO_WORKING_DIR)/libgo_surtr_linux_arm64.a
	@cp ./target/x86_64-unknown-linux-gnu/release/libgo_surtr.a ./$(GO_WORKING_DIR)/libgo_surtr_linux_amd64.a


build-go-surtr: build-go-darwin build-go-linux

build-go-surtr-release: build-go-darwin-release build-go-linux-release