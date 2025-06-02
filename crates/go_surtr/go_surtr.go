// go_surtr package provides Go Bindings for the Surtr library.
package go_surtr

/*
#cgo LDFLAGS: -L${SRCDIR} -ldl
#cgo darwin,arm64 LDFLAGS: -lgo_surtr_darwin_arm64
#cgo linux,arm64 LDFLAGS: -lgo_surtr_linux_arm64
#cgo linux,amd64 LDFLAGS: -lgo_surtr_linux_amd64
typedef struct SurtrOptions SurtrOptions;
#include "go_surtr.h"
#include <stdlib.h>
*/
import "C"
import (
	"unsafe"
)

// SurtError is a Go Error wrapper around any error which can be produed by Surtr.
// These errors are found in 3 main types. They are exposed as a String and
// are prefixed with one of the following Surtr Error names:
// 1. UrlParseError
// 2. NoSchemeFoundError
// 3. CanonicalizerError
//
// These errors are mapped into a Go Error.
type SurtError struct {
	// The raw error string returned from Surtr.
	s string
}

// Error satisfies the Go Error Interface and returns the raw error string
// returned from Surtr. This does no processing.
func (e SurtError) Error() string {
	return e.s
}

// checkString is a helper function to check if the URL is empty.
// If the URL is empty, an error is returned.
//
// This is used to prevent unnecessary calls out of Go and into C.
// This also serves to maintain compatiability in the interface with IA's Python
// Surt library.
func checkString(url string) (string, error) {
	if url == "" {
		return "", SurtError{s: "URL is empty"}
	}

	return url, nil
}

// setupOptions is a helper function to correctly create a SurtrOptions struct within the Surtr library.
// This function returns a Pointer to the Struct in Rust.
// Please ensure that the Struct is destroyed correctly, by calling defer on C.destroy_options(<pointer>)
func setupOptions(options map[string]bool) *C.SurtrOptions {
	option_struct := C.init_options()

	for key, value := range options {
		key_cstr := C.CString(key)
		defer C.free(unsafe.Pointer(key_cstr))

		C.set_option(option_struct, key_cstr, C.bool(value))
	}

	return option_struct
}

// GenerateSurtFromURL is the main function for generating a Surt from a URL.
// Optionally include a Mapping of the supported Surtr Options.
func GenerateSurtFromURL(url string, options ...map[string]bool) (string, error) {
	url, err := checkString(url)
	if err != nil {
		return "", err
	}

	url_cstr := C.CString(url)
	defer C.free(unsafe.Pointer(url_cstr))

	var res C.Results

	if len(options) > 0 {
		options_struct := setupOptions(options[0])
		defer C.destroy_options(options_struct)

		res = C.generate_surt_with_options(url_cstr, options_struct)
	} else {
		res = C.generate_surt(url_cstr)
	}

	if res.error != nil {
		return "", SurtError{s: C.GoString(res.error)}
	}

	return C.GoString(res.output), nil
}
