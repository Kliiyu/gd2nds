// SPDX-License-Identifier: MIT
//
// Milestone 1 test program. Later milestones replace this file with code
// generated from the project's scenes and scripts.

#include <stdio.h>

#include <nds.h>

#include "gdds/gdds.h"
#include "gd2nds_project.h"

int main(int argc, char **argv)
{
    (void)argc;
    (void)argv;

    gdds_init();

    printf("gd2nds %s\n\n", GD2NDS_VERSION);
    printf("Project: %s\n\n", GD2NDS_PROJECT_NAME);
    printf("D-pad: move the sprite\n");

    uint16_t *sprite = gdds_test_sprite_create();
    if (sprite == NULL)
    {
        printf("\nerror: out of sprite VRAM\n");
        while (1)
            swiWaitForVBlank();
    }

    int x = (GDDS_SCREEN_WIDTH - 16) / 2;
    int y = (GDDS_SCREEN_HEIGHT - 16) / 2;

    while (1)
    {
        gdds_frame_begin();

        uint32_t held = keysHeld();
        if (held & KEY_LEFT)
            x--;
        if (held & KEY_RIGHT)
            x++;
        if (held & KEY_UP)
            y--;
        if (held & KEY_DOWN)
            y++;

        if (x < 0)
            x = 0;
        if (x > GDDS_SCREEN_WIDTH - 16)
            x = GDDS_SCREEN_WIDTH - 16;
        if (y < 0)
            y = 0;
        if (y > GDDS_SCREEN_HEIGHT - 16)
            y = GDDS_SCREEN_HEIGHT - 16;

        gdds_sprite_show(0, x, y, sprite);
        gdds_frame_end();
    }

    return 0;
}
