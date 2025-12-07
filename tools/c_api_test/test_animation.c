/**
 * @file test_animation.c
 *
 * Animation system tests using exported test data
 */

#include "../c_api/devilutionx_c_api.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

// Simple JSON parser for test data (minimal implementation)
// In real implementation, use a proper JSON library like cJSON

typedef struct {
    int frames;
    int fps;
    int ticks;
    int expected_frame;
} AnimationTestCase;

// Test helper functions
static int test_count = 0;
static int test_passed = 0;
static int test_failed = 0;

void assert_equals(const char* test_name, int actual, int expected)
{
    test_count++;
    if (actual == expected) {
        test_passed++;
        printf("[PASS] %s: expected=%d, actual=%d\n", test_name, expected, actual);
    } else {
        test_failed++;
        printf("[FAIL] %s: expected=%d, actual=%d\n", test_name, expected, actual);
    }
}

void test_basic_animation_frame()
{
    printf("\n=== Testing Basic Animation Frame Calculation ===\n");

    // Test case 1: Simple case
    int frame = dvlx_get_animation_frame(0, 20, 16);
    assert_equals("Frame at tick 0", frame, 0);

    // Test case 2: Mid-animation
    frame = dvlx_get_animation_frame(100, 20, 16);
    assert_equals("Frame at tick 100 (fps=20, frames=16)", frame, (100/20) % 16);

    // Test case 3: Loop around
    frame = dvlx_get_animation_frame(320, 20, 16);
    assert_equals("Frame at tick 320 (full cycle)", frame, 0);

    // Test case 4: Multiple cycles
    frame = dvlx_get_animation_frame(500, 20, 16);
    assert_equals("Frame at tick 500", frame, (500/20) % 16);
}

void test_focus_size_selection()
{
    printf("\n=== Testing Focus Size Selection ===\n");

    // Test case 1: Small focus (height < 30)
    DvlxFocusSize size = dvlx_ui_select_focus_size(20);
    assert_equals("Height 20 -> SMALL", size, DVLX_FOCUS_SMALL);

    // Test case 2: Boundary (height = 29)
    size = dvlx_ui_select_focus_size(29);
    assert_equals("Height 29 -> SMALL", size, DVLX_FOCUS_SMALL);

    // Test case 3: Medium focus (height = 30)
    size = dvlx_ui_select_focus_size(30);
    assert_equals("Height 30 -> MEDIUM", size, DVLX_FOCUS_MEDIUM);

    // Test case 4: Medium focus (height = 41)
    size = dvlx_ui_select_focus_size(41);
    assert_equals("Height 41 -> MEDIUM", size, DVLX_FOCUS_MEDIUM);

    // Test case 5: Large focus (height = 42)
    size = dvlx_ui_select_focus_size(42);
    assert_equals("Height 42 -> LARGE", size, DVLX_FOCUS_LARGE);

    // Test case 6: Large focus (height = 100)
    size = dvlx_ui_select_focus_size(100);
    assert_equals("Height 100 -> LARGE", size, DVLX_FOCUS_LARGE);
}

void test_error_handling()
{
    printf("\n=== Testing Error Handling ===\n");

    // Test case 1: Negative ticks
    int frame = dvlx_get_animation_frame(-1, 20, 16);
    assert_equals("Negative ticks returns -1", frame, -1);
    assert_equals("Error code is INVALID_PARAM",
        dvlx_get_last_error(), DVLX_ERROR_INVALID_PARAM);

    // Test case 2: Zero fps
    frame = dvlx_get_animation_frame(100, 0, 16);
    assert_equals("Zero fps returns -1", frame, -1);
    assert_equals("Error code is INVALID_PARAM",
        dvlx_get_last_error(), DVLX_ERROR_INVALID_PARAM);

    // Test case 3: Negative frames
    frame = dvlx_get_animation_frame(100, 20, -1);
    assert_equals("Negative frames returns -1", frame, -1);
    assert_equals("Error code is INVALID_PARAM",
        dvlx_get_last_error(), DVLX_ERROR_INVALID_PARAM);

    // Test case 4: Error message
    const char* msg = dvlx_get_error_message(DVLX_ERROR_INVALID_PARAM);
    printf("Error message for INVALID_PARAM: \"%s\"\n", msg);

    // Clear error
    dvlx_clear_error();
    assert_equals("After clear, error is OK",
        dvlx_get_last_error(), DVLX_OK);
}

void test_ui_centering()
{
    printf("\n=== Testing UI Centering ===\n");

    // Test case 1: Center element on 640x480 screen
    int x = dvlx_ui_center_x(640, 320);
    assert_equals("Center 320px width on 640px screen", x, 160);

    // Test case 2: Center vertically
    int y = dvlx_ui_center_y(480, 240);
    assert_equals("Center 240px height on 480px screen", y, 120);

    // Test case 3: Focus position calculation
    DvlxRect rect = { 100, 50, 200, 80 };
    DvlxFocusPosition pos;

    DvlxErrorCode err = dvlx_ui_calculate_focus_position(
        &rect, 72, 17, &pos);

    assert_equals("Focus position calculation succeeded", err, DVLX_OK);
    assert_equals("Focus left_x", pos.left_x, 100);
    assert_equals("Focus right_x", pos.right_x, 100 + 200 - 72);
    assert_equals("Focus y (centered)", pos.y, 50 + (80 - 17) / 2);
}

void test_animation_ex()
{
    printf("\n=== Testing Animation Ex API ===\n");

    DvlxAnimationParams params = { 16, 20, 100 };
    DvlxAnimationResult result = dvlx_get_animation_frame_ex(&params);

    assert_equals("Ex API returns correct frame", result.current_frame, (100/20) % 16);
    assert_equals("Ex API returns OK", result.error, DVLX_OK);

    // Test with NULL pointer
    result = dvlx_get_animation_frame_ex(NULL);
    assert_equals("Ex API with NULL returns error",
        result.error, DVLX_ERROR_NULL_POINTER);
    assert_equals("Ex API with NULL returns -1", result.current_frame, -1);
}

int main(int argc, char** argv)
{
    printf("DevilutionX C API Test Suite\n");
    printf("=============================\n");

    // Initialize API
    DvlxErrorCode err = dvlx_init();
    if (err != DVLX_OK) {
        printf("Failed to initialize API: %s\n", dvlx_get_error_message(err));
        return 1;
    }

    printf("API Version: %s\n", dvlx_get_version());

    // Run tests
    test_basic_animation_frame();
    test_focus_size_selection();
    test_error_handling();
    test_ui_centering();
    test_animation_ex();

    // Summary
    printf("\n=============================\n");
    printf("Test Summary:\n");
    printf("  Total:  %d\n", test_count);
    printf("  Passed: %d\n", test_passed);
    printf("  Failed: %d\n", test_failed);
    printf("=============================\n");

    // Cleanup
    dvlx_cleanup();

    return (test_failed == 0) ? 0 : 1;
}
