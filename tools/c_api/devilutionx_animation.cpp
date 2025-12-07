/**
 * @file devilutionx_animation.cpp
 *
 * Animation system C API implementation
 */

#include "devilutionx_c_api.h"
#include <limits.h>

// Parameter validation
static bool validate_animation_params(int32_t ticks, int32_t fps, int32_t frames)
{
	if (ticks < 0) {
		dvlx_set_error(DVLX_ERROR_INVALID_PARAM);
		return false;
	}

	if (fps <= 0) {
		dvlx_set_error(DVLX_ERROR_INVALID_PARAM);
		return false;
	}

	if (frames <= 0) {
		dvlx_set_error(DVLX_ERROR_INVALID_PARAM);
		return false;
	}

	return true;
}

int32_t dvlx_get_animation_frame(int32_t ticks, int32_t fps, int32_t frames)
{
	// Clear previous error
	dvlx_clear_error();

	// Validate parameters
	if (!validate_animation_params(ticks, fps, frames)) {
		return -1;
	}

	// Core animation frame calculation
	// Formula: frame = (ticks / fps) % frames
	int32_t frame = (ticks / fps) % frames;

	return frame;
}

DvlxAnimationResult dvlx_get_animation_frame_ex(const DvlxAnimationParams* params)
{
	DvlxAnimationResult result;
	result.current_frame = -1;
	result.error = DVLX_OK;

	// Validate params pointer
	if (params == nullptr) {
		result.error = DVLX_ERROR_NULL_POINTER;
		dvlx_set_error(DVLX_ERROR_NULL_POINTER);
		return result;
	}

	// Validate parameters
	if (!validate_animation_params(params->ticks, params->fps, params->frames)) {
		result.error = dvlx_get_last_error();
		return result;
	}

	// Calculate frame
	result.current_frame = (params->ticks / params->fps) % params->frames;
	result.error = DVLX_OK;

	return result;
}
