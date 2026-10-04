/* Included after geno_items_registry.inc. Uses production parser and accessors;
 * preserves the entire immutable registry even when an assertion fails. */
static int gn_items_registry_test(void)
{
 static const char* const invalid[]={
  "{}", "[]", "{\"name\":\"Bad\"}",
  "{\"name\":\"test-drive\",}", "{\"name\":\"test-drive\",\"lifetime\":0300}",
  "{\"name\":\"test-drive\",\"lifetime\":0x100}", "{\"name\":\"test\\-drive\"}",
  "{\"name\":\"test-drive\",\"unknown\":1}",
  "{\"name\":\"test-drive\",\"name\":\"again\"}",
  "{\"name\":\"test-drive\",\"version\":2}",
  "{\"name\":\"test-drive\",\"physics\":[]}",
  "{\"name\":\"test-drive\",\"physics\":{\"gravity\":-1}}",
  "{\"name\":\"test-drive\",\"physics\":{\"bounce\":1.1}}",
  "{\"name\":\"test-drive\",\"physics\":{\"friction\":\"bad\"}}",
  "{\"name\":\"test-drive\",\"collection\":\"grab\"}",
  "{\"name\":\"test-drive\",\"held\":true}",
  "{\"name\":\"test-drive\",\"thrown\":true}",
  "{\"name\":\"test-drive\",\"ports\":64}",
  "{\"name\":\"test-drive\",\"teams\":16}",
  "{\"name\":\"test-drive\",\"lifetime\":2.5}",
  "{\"name\":\"test-drive\",\"lifetime\":2,\"blink\":3}",
  "{\"name\":\"test-drive\",\"radius\":1e999}",
  "{\"name\":\"test-drive\",\"payload\":{\"colour\":\"purple\"}}",
  "{\"name\":\"test-drive\",\"payload\":{\"amount\":1000001}}",
  "{\"name\":\"test-drive\",\"visual\":{\"model\":\"../escape\"}}",
  "{\"name\":\"test-drive\",\"visual\":{\"model\":\"/absolute\"}}",
  "{\"name\":\"test-drive\",\"visual\":{\"scale\":0}}",
  "{\"name\":\"test-drive\",\"visual\":{\"tint\":4294967296}}",
  "{\"name\":\"test-drive\",\"visual\":{\"hover\":-1}}",
  "{\"name\":\"test-drive\",\"visual\":{\"tilt\":91}}",
  "{\"name\":\"test-drive\",\"visual\":{\"height\":-1}}",
  "{\"name\":\"test-drive\",\"visual\":{\"spin\":361}}",
  NULL
 };
 static const char valid[]="{\"name\":\"test-drive\",\"version\":1,"
  "\"physics\":{\"gravity\":0.25,\"terminal_velocity\":6,\"bounce\":0.3,"
  "\"friction\":0.75,\"floor_rest\":true,\"stage_collision\":true},"
  "\"collection\":\"touch\",\"ports\":1,\"teams\":3,\"radius\":8,"
  "\"lifetime\":300,\"blink\":60,\"count_against_cap\":true,"
  "\"held\":false,\"thrown\":false,\"payload\":{\"colour\":\"blue\",\"amount\":20},"
  "\"visual\":{\"asset_mod\":\"test-assets\",\"model\":\"models/drive_blue\","
  "\"glass\":\"models/drive_glass\",\"scale\":2.5,\"hover\":6,\"height\":4.55,\"tilt\":12,\"spin\":6,\"bob\":1,\"bob_speed\":2,\"tint\":4294967295}}";
 GnItemDef* saved=malloc(sizeof gn_items); GnItemDef defined;
 int saved_count=gn_item_count,i,result=0; char error[160];
 if(!saved) { gw_test_fail("standalone registry fixture allocation failed"); return 1; }
 memcpy(saved,gn_items,sizeof gn_items); memset(gn_items,0,sizeof gn_items); gn_item_count=0;
#define GNI_CHECK(c,msg) do { if(!(c)) { gw_test_fail("standalone registry: %s",msg); result=1; goto done; } } while(0)
 for(i=0;invalid[i];++i) {
  error[0]=0;
  GNI_CHECK(gw_Geno_ItemDefineText(invalid[i],"fixture","items/drive/item.json",error,sizeof error)<0,"invalid definition accepted");
  GNI_CHECK(gn_item_count==0 && error[0],"refusal mutated registry or omitted reason");
 }
 if(!gw_Mex_ItemRangeAvailable(GENO_ITEM_KIND_BASE,GENO_ITEM_MAX_DEFS)) {
  GNI_CHECK(gw_Geno_ItemDefineText(valid,"fixture","items/drive/item.json",error,sizeof error)<0,"overlapping m-ex range accepted");
  GNI_CHECK(gn_item_count==0,"range refusal mutated registry"); goto done;
 }
 GNI_CHECK(gw_Geno_ItemDefineText(valid,"fixture","items/drive/item.json",error,sizeof error)==0,"valid definition refused");
 GNI_CHECK(gw_Geno_ItemCount()==1 && gw_Geno_ItemFind("test-drive")==0,"name/count mapping");
 GNI_CHECK(gw_Geno_ItemPayload(0,0)==2 && gw_Geno_ItemPayload(0,1)==20,"payload mapping");
 GNI_CHECK(gw_Geno_ItemParam(0,GENO_IP_CAP)==1 && gw_Geno_ItemParam(0,GENO_IP_PORT_MASK)==1,"cap and port masks");
 GNI_CHECK(gw_Geno_ItemParam(0,GENO_IP_GRAVITY)==gn_item_bits(.25f),"float scalar encoding");
 GNI_CHECK(gw_Geno_ItemParam(0,GENO_IP_COLLECT)==1 && !strcmp(gw_Geno_ItemName(0),"test-drive"),"touch/name readback");
 GNI_CHECK(gw_Geno_ItemParam(0,GENO_IP_SCALE)==gn_item_bits(2.5f) &&
  gw_Geno_ItemParam(0,GENO_IP_HOVER)==gn_item_bits(6) &&
  gw_Geno_ItemParam(0,GENO_IP_HEIGHT)==gn_item_bits(4.55f) &&
  gw_Geno_ItemParam(0,GENO_IP_TILT)==gn_item_bits(12) &&
  gw_Geno_ItemParam(0,GENO_IP_SPIN)==gn_item_bits(6),"hover/height/tilt/scale/spin encoding");
 defined=gn_items[0];
 GNI_CHECK(gw_Geno_ItemDefineText(valid,"fixture","items/drive/item.json",error,sizeof error)==0,"identical registration not idempotent");
 GNI_CHECK(!memcmp(&defined,&gn_items[0],sizeof defined)&&gn_item_count==1,"idempotent registration mutation");
 GNI_CHECK(gw_Geno_ItemDefineText(valid,"other","items/drive/item.json",error,sizeof error)<0,"different origin reused name");
 GNI_CHECK(gw_Geno_ItemDefineText("{\"name\":\"test-drive\"}","fixture","items/drive/item.json",error,sizeof error)<0,"changed definition reused name");
 GNI_CHECK(!memcmp(&defined,&gn_items[0],sizeof defined)&&gn_item_count==1,"duplicate refusal changed definition");
 for(i=1;i<GENO_ITEM_MAX_DEFS;++i) {
  char json[80]; snprintf(json,sizeof json,"{\"name\":\"fixture-%d\"}",i);
  GNI_CHECK(gw_Geno_ItemDefineText(json,"fixture","items/fixture/item.json",error,sizeof error)==i,"dense definition numbering/pool fill");
 }
 GNI_CHECK(gw_Geno_ItemParam(1,GENO_IP_HOVER)==gn_item_bits(0) &&
  gw_Geno_ItemParam(1,GENO_IP_HEIGHT)==gn_item_bits(0) &&
  gw_Geno_ItemParam(1,GENO_IP_TILT)==gn_item_bits(0) &&
  gw_Geno_ItemParam(1,GENO_IP_SCALE)==gn_item_bits(1),"legacy definition defaults");
 GNI_CHECK(gw_Geno_ItemDefineText("{\"name\":\"overflow\"}","fixture","items/overflow/item.json",error,sizeof error)<0,"full registry accepted definition");
 GNI_CHECK(gn_item_count==GENO_ITEM_MAX_DEFS,"overflow changed count");
 GNI_CHECK(!gw_Geno_ItemParam(-1,0)&&!gw_Geno_ItemParam(0,GENO_IP_COUNT)&&!gw_Geno_ItemPayload(GENO_ITEM_MAX_DEFS,0),"invalid scalar read not inert");
done:
 memcpy(gn_items,saved,sizeof gn_items);gn_item_count=saved_count;free(saved);
#undef GNI_CHECK
 return result;
}
