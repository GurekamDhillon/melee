static inline u32 bitset_mask(u32 bit)
{
    return 1 << (bit % 32);
}

static inline void bitset_clear(u32* words, u32 bit)
{
    words[bit / 32] &= ~bitset_mask(bit);
}

struct GmCardData* gmMainLib_GetCardData(void)
{
    return &gmMainLib_804D3EE0->thing;
}

struct FighterData* GetPersistentFighterData(SelectableCharacterKind selkind)
{
    struct FighterData* base = gmMainLib_GetCardData()->save_data.x1F2C;
    return &base[selkind];
}

void gmMainLib_8015D8B0(u32 arg0)
{
    bitset_clear(gmMainLib_GetCardData()->save_data.x1B40, arg0);
}

u32* gmMainLib_8015CCF0(void)
{
    return &gmMainLib_GetCardData()->save_data.x1A48;
}

struct gmm_x0_528_t* gmMainLib_8015CDC8(void)
{
    return &gmMainLib_804D3EE0->vs.unk_51C;
}

struct gmm_x0_528_t* gmMainLib_8015CDD4(void)
{
    return &gmMainLib_804D3EE0->vs.unk_522;
}

struct gmm_x0_528_t* gmMainLib_8015CDE0(void)
{
    return &gmMainLib_804D3EE0->vs.unk_528;
}

s32* gmMainLib_8015D06C(u8 arg0)
{
    return &GetPersistentFighterData(arg0)->x7C.x84;
}

bool gmMainLib_8015D48C(u8 arg0)
{
    return GetPersistentFighterData(arg0)->x7C.b0;
}

void gmMainLib_8015D4E8(u8 arg0, s32 arg1)
{
    GetPersistentFighterData(arg0)->x7C.b0 = arg1;
}

u32* gmMainLib_GetBonusMatchTotal(void)
{
    return &gmMainLib_GetCardData()->save_data.bonus_matches;
}

u32* gmMainLib_GetCoinMatchTotal(void)
{
    return &gmMainLib_GetCardData()->save_data.coin_matches;
}

s32* gmMainLib_GetKOTotal(void)
{
    return &gmMainLib_GetCardData()->save_data.x1A5C;
}

u32* gmMainLib_GetStockMatchTotal(void)
{
    return &gmMainLib_GetCardData()->save_data.stock_matches;
}

u32* gmMainLib_GetTimeMatchTotal(void)
{
    return &gmMainLib_GetCardData()->save_data.time_matches;
}

