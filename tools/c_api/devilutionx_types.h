/**
 * @file devilutionx_types.h
 *
 * C-compatible type definitions for DevilutionX C API
 */
#pragma once

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/*******************************************************************************
 * Basic Types
 ******************************************************************************/

/**
 * @brief Error codes for C API functions
 */
typedef enum {
	DVLX_OK = 0,                   /**< Success */
	DVLX_ERROR_NULL_POINTER = 1,   /**< Null pointer argument */
	DVLX_ERROR_INVALID_PARAM = 2,  /**< Invalid parameter value */
	DVLX_ERROR_OUT_OF_RANGE = 3,   /**< Parameter out of valid range */
	DVLX_ERROR_NOT_INITIALIZED = 4, /**< System not initialized */
	DVLX_ERROR_INTERNAL = 5,        /**< Internal error */
	DVLX_ERROR_UNKNOWN = 99         /**< Unknown error */
} DvlxErrorCode;

/**
 * @brief Point structure (2D coordinate)
 */
typedef struct {
	int32_t x;
	int32_t y;
} DvlxPoint;

/**
 * @brief Rectangle structure
 */
typedef struct {
	int32_t x;
	int32_t y;
	int32_t w;
	int32_t h;
} DvlxRect;

/*******************************************************************************
 * Animation System Types
 ******************************************************************************/

/**
 * @brief Animation parameters
 */
typedef struct {
	int32_t frames;       /**< Total number of frames */
	int32_t fps;          /**< Frames per second */
	int32_t ticks;        /**< Current time tick */
} DvlxAnimationParams;

/**
 * @brief Animation result
 */
typedef struct {
	int32_t current_frame; /**< Current frame index (0-based) */
	DvlxErrorCode error;   /**< Error code */
} DvlxAnimationResult;

/*******************************************************************************
 * UI System Types
 ******************************************************************************/

/**
 * @brief Focus sprite size selection
 */
typedef enum {
	DVLX_FOCUS_SMALL = 0,  /**< Small focus (< 30px height) */
	DVLX_FOCUS_MEDIUM = 1, /**< Medium focus (30-41px height) */
	DVLX_FOCUS_LARGE = 2   /**< Large focus (>= 42px height) */
} DvlxFocusSize;

/**
 * @brief Focus position (for rendering Focus sprite)
 */
typedef struct {
	int32_t left_x;   /**< Left X position */
	int32_t right_x;  /**< Right X position */
	int32_t y;        /**< Y position (vertically centered) */
} DvlxFocusPosition;

/**
 * @brief UI flags (matches C++ UiFlags enum)
 */
typedef enum {
	DVLX_UI_ALIGN_CENTER = 1 << 0,
	DVLX_UI_VERTICAL_CENTER = 1 << 1,
	DVLX_UI_FONT_SIZE_12 = 1 << 2,
	DVLX_UI_FONT_SIZE_24 = 1 << 3,
	DVLX_UI_FONT_SIZE_30 = 1 << 4,
	DVLX_UI_FONT_SIZE_42 = 1 << 5,
	DVLX_UI_FONT_SIZE_46 = 1 << 6
} DvlxUiFlags;

/**
 * @brief Color structure (RGBA)
 */
typedef struct {
	uint8_t r;
	uint8_t g;
	uint8_t b;
	uint8_t a;
} DvlxColor;

/*******************************************************************************
 * Sprite System Types
 ******************************************************************************/

/**
 * @brief Opaque handle to sprite data
 */
typedef struct DvlxSprite_Opaque* DvlxSpriteHandle;

/**
 * @brief Sprite metadata
 */
typedef struct {
	int32_t width;
	int32_t height;
	int32_t frames;
	DvlxErrorCode error;
} DvlxSpriteInfo;

/**
 * @brief Pixel format
 */
typedef enum {
	DVLX_PIXEL_FORMAT_RGBA8888,
	DVLX_PIXEL_FORMAT_ARGB8888,
	DVLX_PIXEL_FORMAT_RGB888,
	DVLX_PIXEL_FORMAT_RGB565,
	DVLX_PIXEL_FORMAT_PAL8
} DvlxPixelFormat;

/*******************************************************************************
 * Game Data Types
 ******************************************************************************/

/**
 * @brief Player/Hero class
 */
typedef enum {
	DVLX_CLASS_WARRIOR = 0,
	DVLX_CLASS_ROGUE = 1,
	DVLX_CLASS_SORCERER = 2,
	DVLX_CLASS_MONK = 3,
	DVLX_CLASS_BARD = 4,
	DVLX_CLASS_BARBARIAN = 5
} DvlxHeroClass;

/**
 * @brief Magic type
 */
typedef enum {
	DVLX_MAGIC_FIRE = 0,
	DVLX_MAGIC_LIGHTNING = 1,
	DVLX_MAGIC_MAGIC = 2
} DvlxMagicType;

/**
 * @brief Player statistics
 */
typedef struct {
	int32_t strength;
	int32_t magic;
	int32_t dexterity;
	int32_t vitality;
	int32_t level;
	DvlxHeroClass hero_class;
} DvlxPlayerStats;

#ifdef __cplusplus
}
#endif
