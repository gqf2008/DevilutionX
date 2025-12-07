/**
 * @file devilutionx_error.h
 *
 * Error handling for DevilutionX C API
 */
#pragma once

#include "devilutionx_types.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Get the last error code
 * @return Last error code that occurred
 */
DvlxErrorCode dvlx_get_last_error(void);

/**
 * @brief Get human-readable error message for error code
 * @param error Error code
 * @return Null-terminated error message string (statically allocated)
 */
const char* dvlx_get_error_message(DvlxErrorCode error);

/**
 * @brief Clear the last error
 */
void dvlx_clear_error(void);

/**
 * @brief Set the last error code (internal use)
 * @param error Error code to set
 */
void dvlx_set_error(DvlxErrorCode error);

#ifdef __cplusplus
}
#endif
