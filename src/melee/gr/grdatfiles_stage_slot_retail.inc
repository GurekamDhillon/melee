/* Same archive publication as grDatFiles_801C6038, using the private cached
 * visit instead of lbArchive/DVD/preload (which share fighter heap 4). */
void grDatFiles_StageSlotInstall(HSD_Archive* archive)
{
    UnkArchiveStruct* dst=grDatFiles_801C62B4();
    dst->unk0=archive;
    dst->unk4=HSD_ArchiveGetPublicAddress(archive,"map_head");
    dst->unk8=0;
    stage_info.coll_data=HSD_ArchiveGetPublicAddress(archive,"coll_data");
    stage_info.param=HSD_ArchiveGetPublicAddress(archive,"grGroundParam");
    stage_info.itemdata=HSD_ArchiveGetPublicAddress(archive,"itemdata");
    stage_info.ald_yaku_all=HSD_ArchiveGetPublicAddress(archive,"ALDYakuAll");
    stage_info.map_ptcl=HSD_ArchiveGetPublicAddress(archive,"map_ptcl");
    stage_info.map_texg=HSD_ArchiveGetPublicAddress(archive,"map_texg");
    stage_info.yakumono_param=HSD_ArchiveGetPublicAddress(archive,"yakumono_param");
    stage_info.map_plit=HSD_ArchiveGetPublicAddress(archive,"map_plit");
    stage_info.quake_model_set=HSD_ArchiveGetPublicAddress(archive,"quake_model_set");
    if(stage_info.map_ptcl && stage_info.map_texg)
        psInitDataBank(0x40,stage_info.map_ptcl,stage_info.map_texg,0,0);
    grDatFiles_801C6228(dst->unk4);
}
