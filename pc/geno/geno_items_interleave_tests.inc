/* Requires a loaded item system for allocation; no assets are read here.
 * Native test adapter binds a generated GXMS asset in the real visual cache. */
int Geno_ItemInterleaveFixture(int def) {
 Article* saved[30];Article stale={0},fresh={0};ItemModelDesc model={0};
 ItemAttr attr={0};HSD_Joint joint={0};Item probe={0};HSD_GObj probe_g={0};
 int i,j,failed=0,drive=0,common=0;Item* ip;
 if(!HSD_GObjPLinkHead||!it_804D6D28||!it_804D6D24||!it_804D6D24[It_Kind_Capsule])return -1;
 if(!Geno_ItemActive(GENO_ITEM_KIND_BASE+def))return -2;
 for(i=0;i<30;++i)saved[i]=it_804A0F60[i];
 stale.x10_modelDesc=(ItemModelDesc*)0xFF73FF73;
 fresh.x0_common_attr=&attr;fresh.x10_modelDesc=&model;model.x0_joint=&joint;
 probe_g.user_data=&probe;probe.kind=It_Kind_Nokonoko;
 for(i=0;i<4;++i) {
  /* The exact failed lifetime: old non-null row survives heap retirement. */
  it_804A0F60[It_Kind_Nokonoko-It_Kind_Old_Kuri]=&stale;
  it_ResetStageArticleTable();
  for(j=0;j<30;++j)if(it_804A0F60[j]!=NULL)++failed;
  it_8026B40C(&fresh,It_Kind_Nokonoko);
  Item_80267978(&probe_g);
  if(probe.xC4_article_data!=&fresh) {++failed;break;}
  if(probe.xC4_article_data->x10_modelDesc->x0_joint!=&joint ||
     probe.xC4_article_data->x0_common_attr!=&attr)++failed;
  drive=Geno_ItemSpawn(def,0,geno_item_bits(60),0,0,0x74000000+i*2,701,0,1);
  if(drive<=0) {++failed;drive=0;break;}
  for(j=0;(ip=geno_item_index(j))!=NULL;++j)if(geno_item_vars(ip)->handle==drive)break;
  if(!ip || ip->xC4_article_data!=&geno_item_desc[def] ||
     ip->xC8_joint!=&geno_item_joint || ip->xCC_item_attr!=&geno_item_attr[def])++failed;
  /* Retail creation while the drive is live and after its slot is freed. */
  for(j=0;j<2;++j) {
   if(j==1){Geno_ItemDespawn(drive,701);drive=0;}
   common=Geno_ItemSpawnKind(It_Kind_Capsule,0,geno_item_bits(60),0,0,0x75000000+i*2+j,702,0,0);
   if(common<=0){++failed;common=0;break;}
   {
    int k;HSD_GObj* cur;
    for(cur=HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM];cur;cur=cur->next)
     if(Geno_ItemMeta(GET_ITEM(cur)->x1C,GENO_II_HANDLE)==common)break;
    if(!cur)++failed;
    else {
     Item* retail=GET_ITEM(cur);Item_80267978(cur);
     if(retail->xC4_article_data!=it_804D6D24[It_Kind_Capsule] ||
        retail->xC8_joint!=it_804D6D24[It_Kind_Capsule]->x10_modelDesc->x0_joint ||
        retail->xCC_item_attr!=it_804D6D24[It_Kind_Capsule]->x0_common_attr)++failed;
     for(k=0;k<GENO_ITEM_MAX_DEFS;++k)if(retail->xC4_article_data==&geno_item_desc[k])++failed;
    }
   }
   Geno_ItemDespawn(common,702);common=0;
  }
  if(drive){Geno_ItemDespawn(drive,701);drive=0;}
 }
 if(drive)Geno_ItemDespawn(drive,701);if(common)Geno_ItemDespawn(common,702);
 for(i=0;i<30;++i)it_804A0F60[i]=saved[i];
 return failed;
}
