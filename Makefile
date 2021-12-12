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

.PHONY: ecr-login
ecr-login:
	aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin 936630031871.dkr.ecr.us-east-1.amazonaws.com

.PHONY: build-app
build-app:
	docker build -t oidc-test-app .

.PHONY: push-app
push-app:
	docker tag oidc-test-db:latest 936630031871.dkr.ecr.us-east-1.amazonaws.com/oidc-test-db:latest
	docker push 936630031871.dkr.ecr.us-east-1.amazonaws.com/oidc-test-db:latest

.PHONY: build-db
build-db:
	docker build . -f ./Dockerfile.mysql -t oidc-test-db

.PHONY: push-db
push-db:
	docker tag oidc-test-app:latest 936630031871.dkr.ecr.us-east-1.amazonaws.com/oidc-test-app:latest
	docker push 936630031871.dkr.ecr.us-east-1.amazonaws.com/oidc-test-app:latest
