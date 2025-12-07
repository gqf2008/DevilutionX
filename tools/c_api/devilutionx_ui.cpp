/**
 * @file devilutionx_ui.cpp
 *
 * UI system C API implementation
 */

#include "devilutionx_c_api.h"

DvlxFocusSize dvlx_ui_select_focus_size(int32_t height)
{
	// Clear previous error
	dvlx_clear_error();

	// Validate height
	if (height <= 0) {
		dvlx_set_error(DVLX_ERROR_INVALID_PARAM);
		return DVLX_FOCUS_SMALL; // Default fallback
	}

	// Selection logic based on C++ implementation
	if (height < 30) {
		return DVLX_FOCUS_SMALL;
	} else if (height < 42) {
		return DVLX_FOCUS_MEDIUM;
	} else {
		return DVLX_FOCUS_LARGE;
	}
}

int32_t dvlx_ui_center_x(int32_t screen_width, int32_t element_width)
{
	dvlx_clear_error();

	if (screen_width < 0 || element_width < 0) {
		dvlx_set_error(DVLX_ERROR_INVALID_PARAM);
		return 0;
	}

	return (screen_width - element_width) / 2;
}

int32_t dvlx_ui_center_y(int32_t screen_height, int32_t element_height)
{
	dvlx_clear_error();

	if (screen_height < 0 || element_height < 0) {
		dvlx_set_error(DVLX_ERROR_INVALID_PARAM);
		return 0;
	}

	return (screen_height - element_height) / 2;
}

int32_t dvlx_ui_calculate_center(int32_t container_size, int32_t element_size)
{
	dvlx_clear_error();

	if (container_size < 0 || element_size < 0) {
		dvlx_set_error(DVLX_ERROR_INVALID_PARAM);
		return 0;
	}

	return (container_size - element_size) / 2;
}

DvlxErrorCode dvlx_ui_calculate_focus_position(
	const DvlxRect* rect,
	int32_t sprite_width,
	int32_t sprite_height,
	DvlxFocusPosition* out_position)
{
	dvlx_clear_error();

	// Validate pointers
	if (rect == nullptr || out_position == nullptr) {
		dvlx_set_error(DVLX_ERROR_NULL_POINTER);
		return DVLX_ERROR_NULL_POINTER;
	}

	// Validate dimensions
	if (sprite_width <= 0 || sprite_height <= 0) {
		dvlx_set_error(DVLX_ERROR_INVALID_PARAM);
		return DVLX_ERROR_INVALID_PARAM;
	}

	// Calculate positions based on C++ implementation
	out_position->left_x = rect->x;
	out_position->right_x = rect->x + rect->w - sprite_width;
	out_position->y = rect->y + (rect->h - sprite_height) / 2;

	return DVLX_OK;
}
