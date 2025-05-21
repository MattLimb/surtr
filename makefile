include crates/surtr/surtr.mk
include crates/py_surtr/py_surtr.mk
include crates/go_surtr/go_surtr.mk

build: build-surtr build-py-surtr build-go-surtr

build-release: build-surtr-release build-py-surtr build-go-surtr-release

test: test-surtr test-py-surtr test-go-surtr
