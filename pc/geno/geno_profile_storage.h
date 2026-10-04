/* Descriptor arena arithmetic, shared with disc-free native tests. The caller
 * owns both buffer and cursor in snapshot-covered game globals. No game heap. */
#ifndef GENO_PROFILE_STORAGE_H
#define GENO_PROFILE_STORAGE_H
#define GENO_PROFILE_STORAGE_BYTES (2u * 1024u * 1024u)
static inline void* geno_storage_take(unsigned char* buffer, unsigned int cap,
                                     unsigned int* used, int bytes)
{
    unsigned int n;
    void* result;
    if (bytes <= 0 || !buffer || !used || *used > cap) return 0;
    n = ((unsigned int)bytes + 7u) & ~7u;
    if (n < (unsigned int)bytes || n > cap - *used) return 0;
    result = buffer + *used;
    *used += n;
    return result;
}
#endif
