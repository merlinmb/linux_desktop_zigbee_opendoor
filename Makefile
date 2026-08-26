.PHONY: install dev build clean help

help:
	@echo "Open Door Monitor - Development Commands"
	@echo ""
	@echo "install       - Install dependencies"
	@echo "dev           - Run in development mode"
	@echo "build         - Build production AppImage"
	@echo "clean         - Clean build artifacts"

install:
	npm install

dev:
	npm run tauri:dev

build:
	npm run tauri:build

clean:
	rm -rf dist/ src-tauri/target/ node_modules/
