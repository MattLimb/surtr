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

type SurtError struct {
	s string
}

func (e SurtError) Error() string {
	return e.s
}

func checkString(url string) (string, error) {
	if url == "" {
		return "", SurtError{s: "URL is empty"}
	}

	return url, nil
}

func GenerateSurtFromURL(url string, options ...map[string]bool) (string, error) {
	url, err := checkString(url)
	if err != nil {
		return "", err
	}

	if len(options) > 0 {
		return generateSurtFromURLOptions(url, options[0])
	}

	url_cstr := C.CString(url)
	defer C.free(unsafe.Pointer(url_cstr))

	res := C.generate_surt(url_cstr)

	if res.error != nil {
		return "", SurtError{s: C.GoString(res.error)}
	}

	return C.GoString(res.output), nil
}

func generateSurtFromURLOptions(url string, options map[string]bool) (string, error) {
	url, err := checkString(url)

	if err != nil {
		return "", err
	}

	url_cstr := C.CString(url)
	defer C.free(unsafe.Pointer(url_cstr))

	if options != nil {
		// Setup Options
		option_struct := C.init_options()
		defer C.destroy_options(option_struct)

		for key, value := range options {
			key_cstr := C.CString(key)
			defer C.free(unsafe.Pointer(key_cstr))

			C.set_option(option_struct, key_cstr, C.bool(value))
		}

		// Generate Surt
		res := C.generate_surt_with_options(url_cstr, option_struct)

		if res.error != nil {
			return "", SurtError{s: C.GoString(res.error)}
		}

		return C.GoString(res.output), nil
	} else {
		res := C.generate_surt(url_cstr)

		if res.error != nil {
			return "", SurtError{s: C.GoString(res.error)}
		}

		return C.GoString(res.output), nil
	}
}
