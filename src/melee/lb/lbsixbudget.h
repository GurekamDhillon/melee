#ifndef MELEE_LB_SIXBUDGET_H
#define MELEE_LB_SIXBUDGET_H
/* Conservative admission for the deduplicated preload request set. Includes
 * 32-byte alignment and two allocation handles plus archive storage per file.
 * The reserve is admission headroom, not a claim about runtime peak usage. */
static int lbSixBudget_Add(unsigned* used, unsigned capacity, unsigned bytes,
                          unsigned archive_bytes)
{
    unsigned cost;
    if (!bytes || bytes > 0x7FFFFF00U) return 0;
    cost = ((bytes + 31U) & ~31U) + ((archive_bytes + 31U) & ~31U) + 64U;
    if (*used > capacity || cost > capacity - *used) return 0;
    *used += cost;
    return 1;
}
#endif
