#include "atlas_check.h"
#include "../platform/gw_ui_tokens.h"

int main(void)
{
    CHECK(AT_C_EMBER == 0xFF7A3DFFu);
    CHECK(AT_C_JADE == 0x4FD6AAFFu);
    CHECK(AT_C_GROUND == 0x0D1015FFu);
    CHECK(AT_C_SCRIM == 0x05070AB8u);
    CHECK(AT_PX_T_CAP == 12 && AT_PX_T_DISPLAY == 64);
    CHECK(AT_PX_CH == 8 && AT_PX_CH_S == 5 && AT_PX_CH_XS == 3);
    CHECK(AT_PX_S1 == 4 && AT_PX_S5 == 24);
    CHECK(AT_MS_FOCUS == 80 && AT_MS_MODAL == 140 && AT_MS_TURN == 12000);
    ATLAS_DONE("atlas tokens");
}
