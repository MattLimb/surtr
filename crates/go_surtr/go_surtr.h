#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct Results {
  const char *output;
  const char *error;
} Results;

SurtrOptions *init_options(void);

/**
 * # Safety
 *
 * This function is unsafe because it takes a pointer to a SurtrOptions struct and returns a pointer to a SurtrOptions struct.
 * The caller is responsible for ensuring that the pointer to the SurtrOptions struct is valid and that the SurtrOptions struct is not null.
 */
void destroy_options(SurtrOptions *inst_ref);

/**
 * # Safety
 *
 * This function is unsafe because it takes a pointer to a SurtrOptions struct and a pointer to a c_char and a bool.
 * The caller is responsible for ensuring that the pointer to the SurtrOptions struct is valid and that the pointer to the c_char is valid.
 */
void set_option(SurtrOptions *inst_ref,
                const char *name,
                bool value);

/**
 * # Safety
 *
 * This function is unsafe because it takes a pointer to a c_char and returns a pointer to a Results struct.
 * The caller is responsible for ensuring that the pointer to the c_char is valid and that the Results struct is not null.
 */
struct Results generate_surt(const char *url);

/**
 * # Safety
 *
 * This function is unsafe because it takes a pointer to a SurtrOptions struct and returns a pointer to a Results struct.
 * The caller is responsible for ensuring that the pointer to the SurtrOptions struct is valid and that the Results struct is not null.
 */
struct Results generate_surt_with_options(const char *url,
                                          SurtrOptions *option_ref);

struct Results generate_surt_error(const char *_url);
