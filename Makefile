.PHONY: dev build test test-watch release-patch release-minor release-major aur-update clean

dev:
	npm run tauri dev

build:
	npm run build
	cd src-tauri && cargo build --release

test:
	npm run test
	cd src-tauri && cargo test

test-watch:
	npm run test:watch

release-patch:
	./scripts/bump-version.sh patch

release-minor:
	./scripts/bump-version.sh minor

release-major:
	./scripts/bump-version.sh major

aur-update:
	cd aur && makepkg --printsrcinfo > .SRCINFO

clean:
	rm -rf dist src-tauri/target node_modules
