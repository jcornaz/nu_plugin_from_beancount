set dotenv-load
set shell := ["nu", "-c"]

@_choose:
	just --list --unsorted

# Perform all verifications (compile, test, lint, etc.)
verify: test && check-msrv lint

# Watch the source files and run `just verify` when source changes
watch:
	cargo watch --delay 0.1 --clear --why -s 'just verify' -s 'cargo deny check'

# Run the tests
test:
	cargo hack test --feature-powerset --optional-deps

# Run the static code analysis
lint:
	cargo fmt -- --check
	cargo hack clippy --feature-powerset --all-targets --optional-deps
	cargo deny check licenses

# Make sure the MSRV is satisfiable
check-msrv:
	cargo msrv verify

# Clean up compilation output
clean:
	rm -rf target
	rm -f Cargo.lock
	rm -rf node_modules

# Install to $CARGO_HOME/bin (But does not register to the plugin nushell)
install:
	cargo install --path .

# Install cargo dev-tools used by the `verify` recipe (requires rustup to be already installed)
install-dev-tools:
	rustup install stable
	rustup override set stable
	cargo install cargo-hack cargo-watch cargo-msrv

# Install a git hook to run tests before every commits
install-git-hooks:
	"#!/usr/bin/env sh\njust verify" | save .git/hooks/pre-commit
	chmod +x .git/hooks/pre-commit

# run the release process in dry run mode (requires npm and a `GITHUB_TOKEN`)
release-dry-run: (release "--dry-run")

# Run the release process (requires `npm`, a `GITHUB_TOKEN` and a `CARGO_REGISTRY_TOKEN`)
release *args:
	npm install --no-save conventional-changelog-conventionalcommits@5 @semantic-release/exec@6 @semantic-release/changelog@6 @semantic-release/git@10
	npx semantic-release@20 {{args}}
