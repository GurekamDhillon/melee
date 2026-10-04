/* Optional client only. Build with GW_PROF_TRACY=1, never in release packages.
 * Default object is empty and adds no Tracy startup, allocation, thread or network activity. */
#if defined(GW_PROF_TRACY) && defined(GW_RELEASE_BUILD)
#error Tracy is development-only
#endif
#ifdef GW_PROF_TRACY
#include "../third_party/tracy/public/TracyClient.cpp"
#endif
