TU = "src/melee/if/ifcoget.c"
RUST_MOD = "melee_world::tu::ifcoget"
SIZE_OK = ["HSD_JObj"]
SKIP_LEAF_TYPES = ["HSD_IDKey"]
RUST_TYPES = {"un_803F9E08_t": "melee_world::tu::ifcoget::un_803F9E08_t"}
CASES = {
    "ifCoget_RenderGuarded": dict(n=40, args=["p:HSD_GObj"]),
    "fn_802FED14": dict(n=600, args=["p:HSD_GObj"]),
    "un_802FEFAC": dict(n=30),
    "un_802FF128": dict(n=300),
    "un_802FF190": dict(n=10),
    "un_802FF1B4": dict(n=30),
}
OUTS = {"lbArchive_80016DBC": [1]}
VARIADIC = {
    "lbArchive_80016DBC": dict(ret="p", ret_rust="*mut melee_types::HSD_Archive", link="buf:64",
                               args=[("s", "*const i8"), ("p", "*mut core::ffi::c_void"), ("s", "*const i8"), ("i", "i32")]),
}
DOM = {
    "arg:un_802FF128#0": [-100, 0, 100, 320, 640, 100000],
    "arg:un_802FF128#1": [-50, 0, 10, 240, 480],
    "arg:un_802FF128#2": [-5, 0, 1, 9, 10, 50, 99, 100, 101, 1000],
    "arg:un_802FF128#3": [-3, 0, 1, 2, 3, 7, 8, 9, 100],
    "ret:Ui_RetailHidden": [0, 1],
    "g:un_803F9E08.x1": [0, 1, 5, 9, 10, 19, 20, 99, 100, 255],
}
LINKS = {
    ".hsd_obj": "new:HSD_JObj",
    "g:un_804D6DA4": "new:SceneDesc",
    "out:lbArchive_80016DBC#1": "new:SceneDesc",
    ".models": "ptrs:DynamicModelDesc:1",
    ".cameras": "ptrs:SceneCameraDesc:1",
    "ret:GObj_Create": "new:HSD_GObj",
    "ret:HSD_JObjLoadJoint": "new:HSD_JObj",
}
FORCE_FILL = ["un_803F9E08"]
