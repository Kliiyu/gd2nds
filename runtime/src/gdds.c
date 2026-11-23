// SPDX-License-Identifier: MIT

#include <stdio.h>

#include <nds.h>

#include "gdds/gdds.h"

#define TEST_SPRITE_SIZE 16

enum {
    PAL_TRANSPARENT = 0,
    PAL_FILL = 1,
    PAL_BORDER = 2,
};

void gdds_init(void)
{
    videoSetMode(MODE_0_2D);
    vramSetBankA(VRAM_A_MAIN_SPRITE);
    oamInit(&oamMain, SpriteMapping_1D_32, false);

    // The bottom screen shows a text console for logs.
    consoleDemoInit();

    SPRITE_PALETTE[PAL_FILL] = RGB15(9, 17, 23);   // Godot logo blue
    SPRITE_PALETTE[PAL_BORDER] = RGB15(31, 31, 31);
}

void gdds_frame_begin(void)
{
    swiWaitForVBlank();
    scanKeys();
}

void gdds_frame_end(void)
{
    oamUpdate(&oamMain);
}

uint16_t *gdds_test_sprite_create(void)
{
    uint16_t *gfx = oamAllocateGfx(&oamMain, SpriteSize_16x16,
                                   SpriteColorFormat_256Color);
    if (gfx == NULL)
        return NULL;

    // In 1D mapping a 16x16 sprite is four 8x8 tiles stored left to right,
    // top to bottom, each 64 bytes (one byte per pixel). VRAM only accepts
    // 16-bit writes, so pixels are written in pairs.
    for (int y = 0; y < TEST_SPRITE_SIZE; y++)
    {
        for (int x = 0; x < TEST_SPRITE_SIZE; x += 2)
        {
            uint8_t px[2];
            for (int i = 0; i < 2; i++)
            {
                int xx = x + i;
                bool edge = xx == 0 || y == 0 || xx == TEST_SPRITE_SIZE - 1 ||
                            y == TEST_SPRITE_SIZE - 1;
                px[i] = edge ? PAL_BORDER : PAL_FILL;
            }
            int tile = (y / 8) * 2 + (x / 8);
            int offset = tile * 64 + (y % 8) * 8 + (x % 8);
            gfx[offset / 2] = px[0] | (px[1] << 8);
        }
    }
    return gfx;
}

void gdds_sprite_show(int id, int x, int y, uint16_t *gfx)
{
    oamSet(&oamMain, id, x, y, 0, 0, SpriteSize_16x16,
           SpriteColorFormat_256Color, gfx, -1, false, false, false, false,
           false);
}
