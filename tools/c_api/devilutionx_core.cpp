/**
 * @file devilutionx_core.cpp
 *
 * Core C API implementation (initialization, version, etc.)
 */

#include "devilutionx_c_api.h"

#define DVLX_API_VERSION "1.0.0"

static bool g_initialized = false;

const char* dvlx_get_version(void)
{
	return DVLX_API_VERSION;
}

DvlxErrorCode dvlx_init(void)
{
	if (g_initialized) {
		return DVLX_OK;
	}

	dvlx_clear_error();
	g_initialized = true;

	return DVLX_OK;
}

void dvlx_cleanup(void)
{
	g_initialized = false;
	dvlx_clear_error();
}
