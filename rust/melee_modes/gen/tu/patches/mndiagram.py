"""Preserve retail overlays after native pointer widening (no game-logic changes).
CleanupData overlays PopupData.text; MainOverlay indexes Diagram's GC layout;
AnimData.jobj is Diagram.jobjs[1]. Native padding/pointer sizes separate the aliases.
"""
def patch(t):
    t=t.replace("mnDiagram_CleanupData* data", "mnDiagram_PopupData* data")
    # PopupData was originally declared after Cleanup; move its declaration earlier.
    a=t.index("typedef struct mnDiagram_PopupData {")
    b=t.index("ASSERT_SIZE(mnDiagram_PopupData, 0x4C);",a)+len("ASSERT_SIZE(mnDiagram_PopupData, 0x4C);")
    decl=t[a:b]; t=t[:a]+t[b:]; p=t.index("void mnDiagram_PopupCleanup("); t=t[:p]+decl+"\n"+t[p:]
    a=t.index("void mnDiagram_ClearGrid("); b=t.index("void mnDiagram_RefreshGrid(",a)
    s=t[a:b].replace("mnDiagram_MainOverlay*", "Diagram*")
    for x,y in [("jobjs[9]","jobjs[7]"),("jobjs[11]","jobjs[9]"),("jobjs[13]","jobjs[11]"),("text[4]","col_header_text"),("text[5]","row_header_text")]: s=s.replace(x,y)
    t=t[:a]+s+t[b:]
    a=t.index("void mnDiagram_ExitAnimProc("); b=t.index("void mnDiagram_UpdateScrollArrowVisibility(",a)
    s=t[a:b].replace("mnDiagram_AnimData*", "Diagram*").replace("data->jobj;", "data->jobjs[1];")
    return t[:a]+s+t[b:]
