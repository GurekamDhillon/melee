#ifndef SCRIPT_ECHO_LIMITS_H
#define SCRIPT_ECHO_LIMITS_H
#define ECHO_ENTITIES 12
#define ECHO_DEPTH 61
#define ECHO_RULES 8
#define ECHO_HITS 5
#define ECHO_COLLISION_CAPS (4 + ECHO_RULES * ECHO_HITS)
/* One victim pass can accept every virtual capsule from every fighter.
 * Retain the original twenty-entry allowance for non-fighter contacts. */
#define ECHO_COLLISION_LOG_CAPACITY (20 + ECHO_ENTITIES * ECHO_COLLISION_CAPS)
#endif
