LIB_FILES := $(shell find pdp12_emulator/src -name '*.rs' -not -path '*/bin/*')
FRONTEND_RESOURCES := $(shell find frontend/resources)
JS_FILES := $(shell find frontend -name '*.js')
FRONTEND_FILES := $(JS_FILES) frontend/index.html frontend/style.css $(FRONTEND_RESOURCES)

frontend/dist: frontend/node_modules pdp12_web/pkg $(FRONTEND_FILES)
	cd frontend/ && npm run build

frontend/node_modules: frontend/package.json frontend/package-lock.json
	cd frontend/ && npm install

node_modules: package.json package-lock.json
	npm install

pdp12_web/pkg: pdp12_web/src/lib.rs $(LIB_FILES) node_modules
	npm run build-rust
