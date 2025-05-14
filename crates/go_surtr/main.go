package main

/*
#cgo LDFLAGS: ./target/libgo_surtr.a -ldl
typedef struct SurtrOptions SurtrOptions;
#include "./target/go_surtr.h"
#include <stdlib.h>
*/
import "C"
import "unsafe"

// func GenerateSurt(url string) string {
// 	p := C.CString(url)
// 	defer C.free(unsafe.Pointer(p))

// 	result := C.GenerateSurtFromURL(p)

// 	return C.GoString(result)
// }

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
	println(GenerateSurt("google.com", nil))

	println(GenerateSurt("https://www.google.com", map[string]bool{"surt": true, "trailing_comma": false}))

	println(GenerateSurt("http://visit.webhosting.yahoo.com/visit.gif?&r=http%3A//web.archive.org/web/20090517140029/http%3A//anthonystewarthead.electric-chi.com/&b=Netscape%205.0%20%28Windows%3B%20en-US%29&s=1366x768&o=Win32&c=24&j=true&v=1.2", nil))
}
