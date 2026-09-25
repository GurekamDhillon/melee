/* SPDX-License-Identifier: GPL-2.0-or-later */
#ifndef GW_SLIPPI_MODE_H
#define GW_SLIPPI_MODE_H
/* NULL with no effects when MELEE_SLIPPI_MODE is absent. Invalid experimental
 * settings fail before matchmaking; a configured run owns its process lifetime. */
const char *gw_SlippiMode_Scene(void);
void gw_SlippiMode_Tick(int online_frame);
#endif
