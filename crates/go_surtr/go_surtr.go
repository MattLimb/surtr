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
	"fmt"
	"unsafe"
)


func GenerateSurt(url string, options map[string]bool) string {
	url_cstr := C.CString(url)
	defer C.free(unsafe.Pointer(url_cstr))

	if options != nil {
		option_struct := C.options_init()
		defer C.options_destroy(option_struct)

		for key, value := range options {
			key_cstr := C.CString(key)
			defer C.free(unsafe.Pointer(key_cstr))

			C.options_set(option_struct, key_cstr, C.bool(value))
		}

		result := C.GenerateSurtFromURLWithOptions(url_cstr, option_struct)
		return C.GoString(result)
	} else {
		result := C.GenerateSurtFromURL(url_cstr)
		return C.GoString(result)
	}
}

func main() {
	fmt.Println(GenerateSurt("google.com", nil))
}