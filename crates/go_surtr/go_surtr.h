#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * A CStyle Struct to pass errors back to Go.
 */
typedef struct Results {
  /**
   * A C Pointer to the successful SURT output string.
   */
  const char *output;
  /**
   * A C Pointer to the error description.
   */
  const char *error;
} Results;

/**
 * Initialize the SurtrOptions Struct internally. This passes a Pointer back to the Caller.
 *
 * # Returns
 *
 * A Pointer to the SurtrOptions struct.
 *
 * # Safety
 *
 * This function is considered unsafe because is passes a pointer back to the caller. This pointer is intended to be used with
 * the destroy_options and set_option functions. Always use destroy_options after all uses of this pointer are used. This will help
 * to prevent memory leaks.
 */
SurtrOptions *init_options(void);

/**
 * Destroy the SurtrOptions Struct internally. This frees the memory allocated for the SurtrOptions struct.
 *
 * # Arguments
 *
 * * `inst_ref` - A Pointer to the SurtrOptions struct to be destroyed.
 *
 * # Safety
 *
 * This function is unsafe because it takes a pointer to a SurtrOptions struct and returns a pointer to a SurtrOptions struct.
 * The caller is responsible for ensuring that the pointer to the SurtrOptions struct is valid and that the SurtrOptions struct is not null.
 */
void destroy_options(SurtrOptions *inst_ref);

/**
 * Set an option within the SurtrOptions struct.
 *
 * # Arguments
 *
 * * `inst_ref` - A Pointer to the SurtrOptions struct to be modified.
 * * `name` - A Pointer to the c_char containing the name of the option to be set.
 * * `value` - A bool containing the value of the option to be set.
 *
 * # Safety
 *
 * This function is unsafe because it takes a pointer to a SurtrOptions struct and a pointer to a c_char and a bool.
 * The caller is responsible for ensuring that the pointer to the SurtrOptions struct is valid and that the pointer to the c_char is valid.
 */
void set_option(SurtrOptions *inst_ref,
                const char *name,
                bool value);

/**
 * Generate a SURT from a URL.
 *
 * # Arguments
 *
 * * `url` - A Pointer to the c_char containing the URL to be transformed.
 *
 * # Returns
 *
 * A Results struct in the successful Output configuration, or an error configuration if the URL is invalid.
 *
 * # Safety
 *
 * This function is unsafe because it takes a pointer to a c_char and returns a pointer to a Results struct.
 * The caller is responsible for ensuring that the pointer to the c_char is valid and that the Results struct is not null.
 */
struct Results generate_surt(const char *url);

/**
 * Generate a SURT from a URL with custom options.
 *
 * # Arguments
 *
 * * `url` - A Pointer to the c_char containing the URL to be transformed.
 * * `option_ref` - A Pointer to the SurtrOptions struct to be used for the transformation.
 *
 * # Returns
 *
 * A Results struct in the successful Output configuration, or an error configuration if the URL is invalid.
 *
 * # Safety
 *
 * This function is unsafe because it takes a pointer to a SurtrOptions struct and returns a pointer to a Results struct.
 * The caller is responsible for ensuring that the pointer to the SurtrOptions struct is valid and that the Results struct is not null.
 */
struct Results generate_surt_with_options(const char *url,
                                          SurtrOptions *option_ref);
