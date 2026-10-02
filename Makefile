.PHONY: dev build test seed release-patch release-minor release-major aur-update clean

dev:
	cargo run

build:
	cargo build --release

test:
	cargo test

seed:
	cargo run --bin seed

release-patch:
	./scripts/bump-version.sh patch

release-minor:
	./scripts/bump-version.sh minor

release-major:
	./scripts/bump-version.sh major

aur-update:
	cd aur && makepkg --printsrcinfo > .SRCINFO

clean:
	cargo clean
