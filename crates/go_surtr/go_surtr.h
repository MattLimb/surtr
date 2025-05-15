#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

SurtrOptions *options_init(void);

void options_set(SurtrOptions *inst_ref, const char *name, bool value);

void options_destroy(SurtrOptions *inst_ref);

const char *GenerateSurtFromURL(const char *url);

const char *GenerateSurtFromURLWithOptions(const char *url, SurtrOptions *option_ref);
