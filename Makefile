export RUSTFLAGS := -Zinstrument-coverage
export LLVM_PROFILE_FILE := ymtdzzz-%p-%m.profraw

.PHONY: coverage-init
coverage-init:
	@rustup component add llvm-tools-preview

.PHONY: coverage
coverage: clean
	@cargo build
	@cargo test
	grcov . --binary-path ./target/debug -s . -t html --branch --ignore-not-existing -o ./coverage/

.PHONY: clean
clean:
	-rm *.profraw
	-rm -rf ./coverage
