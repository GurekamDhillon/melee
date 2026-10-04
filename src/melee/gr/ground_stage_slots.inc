/* Included by ground.c: stage_datas is intentionally private to that TU.
 * Game callers receive a game pointer, never a native shim/data pointer. */
const char* Ground_StageSlotDataFile(int kind)
{
    if (kind < 0 || (unsigned) kind >= sizeof(stage_datas) / sizeof(stage_datas[0]) ||
        stage_datas[kind] == NULL) return NULL;
    return stage_datas[kind]->data1;
}

/* Only the audited, no-hazard FD group-0 callbacks are exposed as the Phase C
 * context proof. This is not permission to run arbitrary stage modules. */
#ifndef STAGE_SLOT_FIXTURE
const GrJoint* Ground_StageSlotJoints(int kind, unsigned* count)
{
    *count=0;
    if(kind<0 || (unsigned)kind>=sizeof(stage_datas)/sizeof(stage_datas[0]) || !stage_datas[kind])return NULL;
    *count=stage_datas[kind]->joint_count;
    return stage_datas[kind]->joints;
}
#endif
