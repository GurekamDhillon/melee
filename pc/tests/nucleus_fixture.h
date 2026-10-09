/* nucleus_fixture.h - SYNTHETIC responses shaped like the SSBM Nucleus public API v1 (https://ssbmnucleus.net/developers). Every title, author,
 * id and URL here is invented; nothing is copied from a real post, and no third-party file or image is referenced by content. */
#ifndef NUCLEUS_FIXTURE_H
#define NUCLEUS_FIXTURE_H

#define FX_M "https://media.ssbmnucleus.net/posts/2026/01/post_"

/* page 1 of the full pass: a costume post with two Fox files (one colour only in the file name), a stage skin, a Sheik costume */
static const char *const fx_page1 =
"{\"data\":["
"{\"id\":101,\"title\":\"Neon Fox\",\"description\":\"A synthetic recolour.\\nSecond line.\",\"author\":\"Tester One\",\"type\":\"costume\",\"stage\":null,"
 "\"tags\":[\"Character Costume\",\"Fox\",\"Neon\"],\"created_at\":\"2026-01-02T03:04:05.000000Z\",\"updated_at\":\"2026-02-03T04:05:06.000000Z\","
 "\"download_count\":12,\"like_count\":3,\"page_url\":\"https://ssbmnucleus.net/post/101/neon-fox\","
 "\"thumbnail_url\":\"" FX_M "101/screenshot_0__thumb.webp\",\"screenshots\":[],\"download_url\":\"https://ssbmnucleus.net/api/public/v1/mods/101/download\","
 "\"zip_url\":\"" FX_M "101/files.zip\",\"files\":["
  "{\"id\":9001,\"filename\":\"Neon Fox - PlFxLa (alt).dat\",\"file_type\":\"character_dat\",\"character\":\"Fox\",\"color\":\"Lavender\",\"slippi_safe\":null,"
   "\"csp_url\":\"" FX_M "101/Neon%20Fox_GENERATED_csp.png\",\"stock_url\":\"" FX_M "101/Neon%20Fox_GENERATED_stock.png\",\"preview_url\":null,"
   "\"download_url\":\"https://ssbmnucleus.net/api/public/v1/mods/101/download?file=9001\",\"file_url\":\"" FX_M "101/Neon%20Fox.dat\"},"
  "{\"id\":9002,\"filename\":\"PlFxGr.dat\",\"file_type\":\"character_dat\",\"character\":\"Fox\",\"color\":\"Green\",\"slippi_safe\":true,"
   "\"csp_url\":null,\"stock_url\":null,\"preview_url\":null,"
   "\"download_url\":\"https://ssbmnucleus.net/api/public/v1/mods/101/download?file=9002\",\"file_url\":\"" FX_M "101/PlFxGr.dat\"}]},"
"{\"id\":102,\"title\":\"Blue Battlefield\",\"description\":\"\",\"author\":\"Tester Two\",\"type\":\"stage_skin\",\"stage\":\"Battlefield\",\"tags\":[],"
 "\"created_at\":\"2026-01-05T00:00:00.000000Z\",\"updated_at\":\"2026-01-06T00:00:00.000000Z\",\"download_count\":40,\"like_count\":9,"
 "\"page_url\":\"https://ssbmnucleus.net/post/102/blue-battlefield\",\"thumbnail_url\":null,\"screenshots\":[],\"files\":["
  "{\"id\":9010,\"filename\":\"GrNBa.dat\",\"file_type\":\"stage_dat\",\"character\":null,\"color\":null,\"csp_url\":null,\"stock_url\":null}]},"
"{\"id\":103,\"title\":\"Sheik Noir\",\"description\":\"Sheik only.\",\"author\":\"Tester One\",\"type\":\"costume\",\"tags\":[\"Sheik\"],"
 "\"created_at\":\"2026-01-07T00:00:00.000000Z\",\"updated_at\":\"2026-01-07T00:00:00.000000Z\",\"download_count\":1,\"like_count\":0,"
 "\"page_url\":\"https://ssbmnucleus.net/post/103/sheik-noir\",\"files\":["
  "{\"id\":9020,\"filename\":\"PlSkBu.dat\",\"file_type\":\"character_dat\",\"character\":\"Sheik\",\"color\":\"Blue\",\"csp_url\":null,\"stock_url\":null}]}"
"],\"next_cursor\":\"AB+/==\",\"total\":6}";

/* page 2: a Zelda costume, a post whose file type is "other" but whose name is a slot code, a title with non-ASCII text */
static const char *const fx_page2 =
"{\"data\":["
"{\"id\":104,\"title\":\"Zelda Noir\",\"description\":\"d\",\"author\":\"Tester Three\",\"type\":\"costume\",\"tags\":[\"Zelda\"],"
 "\"created_at\":\"2026-01-08T00:00:00.000000Z\",\"updated_at\":\"2026-01-09T00:00:00.000000Z\",\"download_count\":7,\"like_count\":2,"
 "\"page_url\":\"https://ssbmnucleus.net/post/104/zelda-noir\",\"files\":["
  "{\"id\":9030,\"filename\":\"PlZdBu.dat\",\"file_type\":\"character_dat\",\"character\":\"Zelda\",\"color\":\"Blue\",\"csp_url\":null,\"stock_url\":null}]},"
"{\"id\":105,\"title\":\"G&W flat\",\"description\":\"\",\"author\":\"Tester Four\",\"type\":\"costume\",\"tags\":[],"
 "\"created_at\":\"2026-01-10T00:00:00.000000Z\",\"updated_at\":\"2026-01-10T00:00:00.000000Z\",\"download_count\":0,\"like_count\":0,"
 "\"page_url\":\"https://ssbmnucleus.net/post/105/gw\",\"files\":["
  "{\"id\":9040,\"filename\":\"PlGw.dat\",\"file_type\":\"other\",\"character\":null,\"color\":null,\"csp_url\":null,\"stock_url\":null}]},"
"{\"id\":106,\"title\":\"Caf\\u00e9 \\ud83d\\ude00 Marth\",\"description\":\"x\",\"author\":\"Zo\\u00eb\",\"type\":\"costume\",\"tags\":[\"Marth\"],"
 "\"created_at\":\"2026-01-11T00:00:00.000000Z\",\"updated_at\":\"2026-01-12T00:00:00.000000Z\",\"download_count\":100,\"like_count\":50,"
 "\"page_url\":\"https://ssbmnucleus.net/post/106/cafe\",\"files\":["
  "{\"id\":9050,\"filename\":\"PlMsRe.dat\",\"file_type\":\"character_dat\",\"character\":\"Marth\",\"color\":\"Red\",\"csp_url\":null,\"stock_url\":null}]}"
"],\"next_cursor\":null,\"total\":6}";

/* the delta after the full pass: 101 changed, 107 is new */
static const char *const fx_delta =
"{\"data\":["
"{\"id\":101,\"title\":\"Neon Fox v2\",\"description\":\"updated\",\"author\":\"Tester One\",\"type\":\"costume\",\"tags\":[\"Fox\"],"
 "\"created_at\":\"2026-01-02T03:04:05.000000Z\",\"updated_at\":\"2026-03-03T04:05:06.000000Z\",\"download_count\":15,\"like_count\":4,"
 "\"page_url\":\"https://ssbmnucleus.net/post/101/neon-fox\",\"files\":["
  "{\"id\":9001,\"filename\":\"Neon Fox - PlFxLa (alt).dat\",\"file_type\":\"character_dat\",\"character\":\"Fox\",\"color\":\"Lavender\",\"csp_url\":null,\"stock_url\":null}]},"
"{\"id\":107,\"title\":\"Falco Ember\",\"description\":\"\",\"author\":\"Tester Five\",\"type\":\"costume\",\"tags\":[\"Falco\"],"
 "\"created_at\":\"2026-03-01T00:00:00.000000Z\",\"updated_at\":\"2026-03-02T00:00:00.000000Z\",\"download_count\":0,\"like_count\":0,"
 "\"page_url\":\"https://ssbmnucleus.net/post/107/falco-ember\",\"files\":["
  "{\"id\":9060,\"filename\":\"PlFcRe.dat\",\"file_type\":\"character_dat\",\"character\":\"Falco\",\"color\":\"Red\",\"csp_url\":null,\"stock_url\":null}]}"
"],\"next_cursor\":null,\"total\":2}";

static const char *const fx_removed = "{\"data\":[{\"id\":102,\"removed_at\":\"2026-03-02T10:00:00Z\"}],\"next_cursor\":null}";
static const char *const fx_error429 = "{\"error\":{\"code\":\"rate_limited\",\"message\":\"slow down\"}}";


#endif
