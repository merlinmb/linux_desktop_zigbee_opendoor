.PHONY: install dev build package deploy clean help

help:
	@echo "Open Door Monitor - Development Commands"
	@echo ""
	@echo "install       - Install dependencies"
	@echo "dev           - Run in development mode"
	@echo "build         - Build production AppImage/deb"
	@echo "package       - Build and stage .deb/.AppImage in release/"
	@echo "deploy HOST=user@host [ARGS='--config'] - scp+install package on HOST"
	@echo "clean         - Clean build artifacts"

install:
	npm install

dev:
	npm run tauri:dev

build:
	npm run tauri:build

package:
	bash scripts/package.sh

deploy:
	@if [ -z "$(HOST)" ]; then echo "Usage: make deploy HOST=user@host [ARGS='--config']" >&2; exit 1; fi
	bash scripts/deploy.sh $(HOST) $(ARGS)

clean:
	rm -rf dist/ src-tauri/target/ node_modules/ release/
