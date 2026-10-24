SPECS = {
'sislib': dict(src='src/sysdolphin/baselib/sislib.c', crate='melee_runtime', types=['HSD_Text','SisBlock','sislib_UnkAlloc3','HSD_GObj'], skip_stubs=['OSPanic'], ret={
'HSD_MemAlloc': ('ARENA','(&raw mut super::ARENA).cast()'),
'GObj_Create': ('&G[3]','&raw mut super::G[3]'),
'HSD_CObjLoadDesc': ('kcase%5?&CAM:0','if super::KCASE%5!=0 {&raw mut super::CAM} else {core::ptr::null_mut()}'),
'HSD_SisLib_803A945C': ('AR','(&raw mut super::AR).cast()'),
'HSD_ArchiveGetPublicAddress': ('TABLE','(&raw mut super::TABLE).cast()'),
'View_AspectBits': ('0x3f800000+(fv()%0x1000000)','(0x3f800000+(fv()%0x1000000)) as i32'),
}, effects={
'GObj_InitUserData': ('a0->user_data_kind=a1; a0->user_data_remove_func=a2; a0->user_data=a3;', '(*a0).user_data_kind=a1;(*a0).user_data_remove_func=a2;(*a0).user_data=a3;'),
'HSD_GObjObject_80390A70': ('a0->obj_kind=a1;a0->hsd_obj=a2;', '(*a0).obj_kind=a1;(*a0).hsd_obj=a2;'),
'GObj_SetupGXLink': ('a0->render_cb=a1;a0->gx_link=a2;a0->render_priority=a3;', '(*a0).render_cb=a1;(*a0).gx_link=a2;(*a0).render_priority=a3 as u8;'),
'GObj_SetupGXLinkMax': ('a0->render_cb=a1;a0->render_priority=a2;', '(*a0).render_cb=a1;(*a0).render_priority=a2 as u8;'),
}),
'sobjlib': dict(src='src/sysdolphin/baselib/sobjlib.c', crate='melee_runtime', types=['HSD_SObj','HSD_GObj','HSD_SObjDesc','HSD_ImageDesc','HSD_Tlut','HSD_ObjAllocData'], skip_stubs=['sinf','cosf'],
ret={
'HSD_ObjAlloc': ('&S[3]','(&raw mut super::S[3]).cast()'),
'HSD_CObjAlloc': ('&CAM','&raw mut super::CAM'),
'HSD_CObjGetCurrent': ('&CAM','&raw mut super::CAM'),
'HSD_GObjObject_80390ADC': ('0','core::ptr::null_mut()'),
'GXGetTexObjFmt': ('fv()%15','fv()%15'),
'GXGetTexObjWidth': ('1+fv()%256','(1+fv()%256) as u16'),
'GXGetTexObjHeight': ('1+fv()%256','(1+fv()%256) as u16'),
}, effects={
'HSD_GObjObject_80390ADC': ('a0->hsd_obj=0;', '(*a0).hsd_obj=core::ptr::null_mut();'),
'HSD_GObjObject_80390A70': ('a0->obj_kind=a1; a0->hsd_obj=a2;', '(*a0).obj_kind=a1; (*a0).hsd_obj=a2;'),
'HSD_CObjGetViewingMtx': ('for(int j=0;j<12;j++) ((float*)a1)[j]=rw();','for j in 0..12 { *(a1 as *mut f32).add(j)=rw(); }'),
'GXLoadPosMtxImm': ('for(int j=0;j<12;j++) T(B(((float*)a0)[j]));','for j in 0..12 { t((*(a0 as *const f32).add(j)).to_bits()); }'),
'HSD_CObjSetEyePosition': ('T(B(a1->x));T(B(a1->y));T(B(a1->z));','t((*a1).x.to_bits());t((*a1).y.to_bits());t((*a1).z.to_bits());'),
'HSD_CObjSetInterest': ('T(B(a1->x));T(B(a1->y));T(B(a1->z));','t((*a1).x.to_bits());t((*a1).y.to_bits());t((*a1).z.to_bits());'),
'HSD_CObjSetViewport': ('T(a1->xmin);T(a1->xmax);T(a1->ymin);T(a1->ymax);','t((*a1).xmin as u32);t((*a1).xmax as u32);t((*a1).ymin as u32);t((*a1).ymax as u32);'),
'HSD_CObjSetScissor': ('T(a1->left);T(a1->right);T(a1->top);T(a1->bottom);','t((*a1).left as u32);t((*a1).right as u32);t((*a1).top as u32);t((*a1).bottom as u32);'),
}),
}



# Stable Rust represents the optional %s argument explicitly; successful routines use no other format shapes.
for spec in SPECS.values():
    spec['custom_stubs'] = {'OSReport': {
        'c': 'void OSReport(char*f,...){T(@ID@);T(SH(f));char*a=0;for(char*p=f;*p;p++){if(p[0]==37&&p[1]==115){__builtin_va_list v;__builtin_va_start(v,f);a=__builtin_va_arg(v,char*);__builtin_va_end(v);break;}}T(SH(a));}',
        'ext': 'fn OSReport(a0: *mut core::ffi::c_char, a1: *mut core::ffi::c_char);',
        'hook': '|a0,a1| {t(@ID@);t(sh(a0 as *const u8));t(sh(a1 as *const u8));}',
    }}

