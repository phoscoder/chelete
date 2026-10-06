.PHONY: dev build install-local test seed release-patch release-minor release-major aur-update clean

dev:
	cargo run

build:
	cargo build --release

# Install the release build with its icon, launcher entry and hourly backup timer under ~/.local
install-local: build
	install -Dm755 target/release/chelete $(HOME)/.local/bin/chelete
	install -Dm644 assets/icons/128x128@2x.png $(HOME)/.local/share/icons/hicolor/256x256/apps/chelete.png
	install -Dm644 assets/chelete.desktop $(HOME)/.local/share/applications/chelete.desktop
	install -Dm755 scripts/chelete-backup $(HOME)/.local/bin/chelete-backup
	install -Dm644 assets/systemd/chelete-backup.service $(HOME)/.config/systemd/user/chelete-backup.service
	install -Dm644 assets/systemd/chelete-backup.timer $(HOME)/.config/systemd/user/chelete-backup.timer
	-systemctl --user daemon-reload
	-systemctl --user enable --now chelete-backup.timer
	-gtk-update-icon-cache -q $(HOME)/.local/share/icons/hicolor
	-update-desktop-database $(HOME)/.local/share/applications

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
