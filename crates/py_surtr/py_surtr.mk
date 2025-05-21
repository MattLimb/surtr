PY_WORKING_DIR := crates/py_surtr


py_surtr_env:
	@cd $(PY_WORKING_DIR); \
		python -m venv .venv; \
		.venv/bin/python -m pip install -r requirements-dev.txt

build-py-surtr:
	@cd $(PY_WORKING_DIR); \
		.venv/bin/python -m maturin build --release --target aarch64-apple-darwin

test-py-surtr:
	@cd $(PY_WORKING_DIR); \
		source .venv/bin/activate; \
		python -m maturin develop; \
		python -m pytest tests
