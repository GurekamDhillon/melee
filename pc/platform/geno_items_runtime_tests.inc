/* Native suite adapters. Runtime execution needs the normal engine test binary;
 * these fixtures do not launch a game or load disc art themselves. */
extern int gw_Geno_ItemFixture(int def);
extern int gw_Geno_ItemSnapshotFixture(int def);
static int gn_items_runtime_test(int snapshot) {
 GnItemDef* saved=malloc(sizeof gn_items);int count=gn_item_count,d,rc=1;char error[256];
 if(!saved) {gw_test_fail("item fixture registry backup allocation failed");return 1;}
 memcpy(saved,gn_items,sizeof gn_items);gn_item_count=0;
 if(!gw_Mex_ItemRangeAvailable(GENO_ITEM_KIND_BASE,GENO_ITEM_MAX_DEFS)) {
  gw_log("geno item fixture: native range conflicts with m-ex; runtime fixture skipped");rc=0;goto done;
 }
 d=gw_Geno_ItemDefineText("{\"name\":\"fixture\",\"collection\":\"touch\",\"lifetime\":120,\"blink\":16,\"visual\":{\"bob\":0,\"scale\":2.5,\"hover\":6,\"height\":4.55,\"tilt\":12,\"spin\":6}}","fixture","items/fixture/item.json",error,sizeof error);
 if(d<0) {gw_test_fail(error);goto done;}
 rc=snapshot?gw_Geno_ItemSnapshotFixture(d):gw_Geno_ItemFixture(d);
 if(rc)gw_test_fail(snapshot?"item snapshot/list/payload/visual round trip failed":"item floor/touch/expiry/payload fixture failed");
done:
 memcpy(gn_items,saved,sizeof gn_items);gn_item_count=count;free(saved);return rc?1:0;
}
static int gn_items_physics_test(void) {return gn_items_runtime_test(0);}
static int gn_items_snapshot_test(void) {return gn_items_runtime_test(1);}
