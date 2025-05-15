#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct Results {
  const char *output;
  const char *error;
} Results;

SurtrOptions *init_options(void);

void destroy_options(SurtrOptions *inst_ref);

void set_option(SurtrOptions *inst_ref, const char *name, bool value);

struct Results generate_surt(const char *url);

struct Results generate_surt_with_options(const char *url, SurtrOptions *option_ref);

struct Results generate_surt_error(const char *_url);
