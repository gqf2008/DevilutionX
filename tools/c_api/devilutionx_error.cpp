/**
 * @file devilutionx_error.cpp
 *
 * Error handling implementation
 */

#include "devilutionx_error.h"
#include <string.h>

// Thread-local storage for last error
#ifdef _MSC_VER
__declspec(thread) static DvlxErrorCode g_last_error = DVLX_OK;
#else
static __thread DvlxErrorCode g_last_error = DVLX_OK;
#endif

DvlxErrorCode dvlx_get_last_error(void)
{
	return g_last_error;
}

const char* dvlx_get_error_message(DvlxErrorCode error)
{
	switch (error) {
	case DVLX_OK:
		return "Success";
	case DVLX_ERROR_NULL_POINTER:
		return "Null pointer argument";
	case DVLX_ERROR_INVALID_PARAM:
		return "Invalid parameter value";
	case DVLX_ERROR_OUT_OF_RANGE:
		return "Parameter out of valid range";
	case DVLX_ERROR_NOT_INITIALIZED:
		return "System not initialized";
	case DVLX_ERROR_INTERNAL:
		return "Internal error";
	case DVLX_ERROR_UNKNOWN:
	default:
		return "Unknown error";
	}
}

void dvlx_clear_error(void)
{
	g_last_error = DVLX_OK;
}

void dvlx_set_error(DvlxErrorCode error)
{
	g_last_error = error;
}
