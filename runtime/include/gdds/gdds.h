// SPDX-License-Identifier: MIT
//
// libgdds: the gd2nds runtime for Nintendo DS.
//
// Milestone 1 only sets up the hardware and offers a test sprite. The scene
// tree, nodes and script support described in the architecture plan build on
// top of this.

#ifndef GDDS_H
#define GDDS_H

#include <stdint.h>

#define GDDS_SCREEN_WIDTH  256
#define GDDS_SCREEN_HEIGHT 192

// Set up the main engine for 2D sprites and the sub screen as a text console.
void gdds_init(void);

// Wait for VBlank and refresh the input state. Call once per frame.
void gdds_frame_begin(void);

// Copy shadow OAM to the hardware. Call after all sprites are set.
void gdds_frame_end(void);

// Allocate a 16x16 256-colour sprite in VRAM and fill it with a test pattern
// (a Godot-blue square with a white border). Returns NULL if VRAM is full.
uint16_t *gdds_test_sprite_create(void);

// Show a 16x16 256-colour sprite in OAM slot `id` at (x, y).
void gdds_sprite_show(int id, int x, int y, uint16_t *gfx);

#endif // GDDS_H
