
//! CPU controller init: `ftCo_800A101C` (kinds/ftCommon/ftCo_0A01.c) and `ftCo_800B9704`
//! (ftcpuattack.c). `Fighter_Spawn` runs them for every fighter, human or
//! CPU, so their two `HSD_Randf` draws are part of every match start (and every respawn).
//!
//! Not ported here: `ftCo_800A0FB0`'s floor probe (`mpCheckFloor` under the fighter, which needs
//! the stage collision; TODO(group2 mp)). The caller passes its result as `floor` (`None` = no
//! floor found, the C's else branch). `FTAI_KIND` is the identity (vanilla kinds; the TARGET_PC
//! `ftAi_Kind` maps only m-ex kinds).

use melee_types::{Fighter, FighterKind, Ft_Kind_Donkey, Ft_Kind_GKoops, Ft_Kind_Koopa, Ft_Kind_Nana};

pub use crate::tu::ftCo_0A01::ftco_800a101c;
pub use crate::tu::ftcpuattack::ftco_800b9704;
