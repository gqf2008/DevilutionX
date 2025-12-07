/**
 * @file devilutionx_c_api.h
 *
 * Main C API header for DevilutionX
 */
#pragma once

#include "devilutionx_types.h"
#include "devilutionx_error.h"

#ifdef __cplusplus
extern "C" {
#endif

/*******************************************************************************
 * Animation System API
 ******************************************************************************/

/**
 * @brief Calculate the current frame index for an animation
 *
 * This is the core animation frame calculation function. It determines which
 * frame should be displayed based on the current time tick, framerate, and
 * total number of frames.
 *
 * Formula: frame = (ticks / fps) % frames
 *
 * @param ticks Current time tick (must be >= 0)
 * @param fps Frames per second (must be > 0)
 * @param frames Total number of frames in animation (must be > 0)
 * @return Current frame index (0 to frames-1), or -1 on error
 *
 * @note Call dvlx_get_last_error() to get error details if return value is -1
 */
int32_t dvlx_get_animation_frame(int32_t ticks, int32_t fps, int32_t frames);

/**
 * @brief Calculate animation frame with full result structure
 *
 * Same as dvlx_get_animation_frame but returns error code in result structure.
 *
 * @param params Animation parameters
 * @return Result containing current frame and error code
 */
DvlxAnimationResult dvlx_get_animation_frame_ex(const DvlxAnimationParams* params);

/*******************************************************************************
 * UI System API
 ******************************************************************************/

/**
 * @brief Select appropriate Focus sprite size based on item height
 *
 * Logic:
 * - height < 30: SMALL
 * - 30 <= height < 42: MEDIUM
 * - height >= 42: LARGE
 *
 * @param height Item height in pixels (must be > 0)
 * @return Focus size enumeration value
 */
DvlxFocusSize dvlx_ui_select_focus_size(int32_t height);

/**
 * @brief Calculate centered X position for UI element
 *
 * Formula: x = (screen_width - element_width) / 2
 *
 * @param screen_width Screen width in pixels
 * @param element_width Element width in pixels
 * @return Centered X position
 */
int32_t dvlx_ui_center_x(int32_t screen_width, int32_t element_width);

/**
 * @brief Calculate centered Y position for UI element
 *
 * Formula: y = (screen_height - element_height) / 2
 *
 * @param screen_height Screen height in pixels
 * @param element_height Element height in pixels
 * @return Centered Y position
 */
int32_t dvlx_ui_center_y(int32_t screen_height, int32_t element_height);

/**
 * @brief Calculate centered position in a container (generic)
 *
 * Formula: position = (container_size - element_size) / 2
 *
 * @param container_size Container dimension (width or height)
 * @param element_size Element dimension
 * @return Centered position
 */
int32_t dvlx_ui_calculate_center(int32_t container_size, int32_t element_size);

/**
 * @brief Calculate Focus sprite position within a rectangle (vertically centered)
 * 
 * Formula:
 * - left_x = rect.x
 * - right_x = rect.x + rect.w - sprite_width
 * - y = rect.y + (rect.h - sprite_height) / 2
 * 
 * @param rect Target rectangle
 * @param sprite_width Focus sprite width
 * @param sprite_height Focus sprite height
 * @param out_position Output: calculated position
 * @return Error code
 */
DvlxErrorCode dvlx_ui_calculate_focus_position(
	const DvlxRect* rect,
	int32_t sprite_width,
	int32_t sprite_height,
	DvlxFocusPosition* out_position
);/*******************************************************************************
 * Version and Info API
 ******************************************************************************/

/**
 * @brief Get API version string
 * @return Version string (e.g., "1.0.0")
 */
const char* dvlx_get_version(void);

/**
 * @brief Initialize the C API (call before using any other functions)
 * @return Error code
 */
DvlxErrorCode dvlx_init(void);

/**
 * @brief Cleanup the C API (call when done)
 */
void dvlx_cleanup(void);

#ifdef __cplusplus
}
#endif
