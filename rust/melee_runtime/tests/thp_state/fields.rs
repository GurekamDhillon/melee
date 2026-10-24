// Generated from THPFileInfo by oracle/build_thp.py.
unsafe fn state_fields(info: &THPFileInfo, base: usize) -> std::vec::Vec<(String,u64)> {
let mut out = std::vec::Vec::new();
out.push(("THPFileInfo.file".into(), if (*info).file.is_null() {0} else {((*info).file as usize).wrapping_sub(base) as u64}));
out.push(("THPFileInfo.currByte".into(), (*info).currByte as u64));
out.push(("THPFileInfo.cnt".into(), (*info).cnt as u64));
out.push(("THPFileInfo.dataStart".into(), if (*info).dataStart.is_null() {0} else {((*info).dataStart as usize).wrapping_sub(base) as u64}));
for i0 in 0..16 {
out.push(("THPFileInfo.mcuBuffer[]".into(), if (*info).mcuBuffer[i0].is_null() {0} else {((*info).mcuBuffer[i0] as usize).wrapping_sub(base) as u64}));
}
out.push(("THPFileInfo.xSize".into(), (*info).xSize as u64));
out.push(("THPFileInfo.ySize".into(), (*info).ySize as u64));
for i0 in 0..24 {
out.push(("THPFileInfo.pad54[]".into(), (*info).pad54[i0] as u64));
}
out.push(("THPFileInfo.scanStart".into(), if (*info).scanStart.is_null() {0} else {((*info).scanStart as usize).wrapping_sub(base) as u64}));
out.push(("THPFileInfo.xPixelSize".into(), (*info).xPixelSize as u64));
out.push(("THPFileInfo.yPixelSize".into(), (*info).yPixelSize as u64));
out.push(("THPFileInfo.x74".into(), (*info).x74 as u64));
out.push(("THPFileInfo.decompressedY".into(), (*info).decompressedY as u64));
out.push(("THPFileInfo.validHuffmanTabs".into(), (*info).validHuffmanTabs as u64));
out.push(("THPFileInfo.validQuantTabs".into(), (*info).validQuantTabs as u64));
out.push(("THPFileInfo.samplingHMax".into(), (*info).samplingHMax as u64));
out.push(("THPFileInfo.samplingVMax".into(), (*info).samplingVMax as u64));
out.push(("THPFileInfo.nComponents".into(), (*info).nComponents as u64));
out.push(("THPFileInfo.x7D".into(), (*info).x7D as u64));
for i0 in 0..10 {
out.push(("THPFileInfo.pad7E[]".into(), (*info).pad7E[i0] as u64));
}
for i0 in 0..4 {
for i1 in 0..32 {
out.push(("THPFileInfo.huffmanTabs[].quick[]".into(), (*info).huffmanTabs[i0].quick[i1] as u64));
}
for i1 in 0..32 {
out.push(("THPFileInfo.huffmanTabs[].increment[]".into(), (*info).huffmanTabs[i0].increment[i1] as u64));
}
out.push(("THPFileInfo.huffmanTabs[].Vij".into(), if (*info).huffmanTabs[i0].Vij.is_null() {0} else {((*info).huffmanTabs[i0].Vij as usize).wrapping_sub(base) as u64}));
for i1 in 0..18 {
out.push(("THPFileInfo.huffmanTabs[].maxCode[]".into(), (*info).huffmanTabs[i0].maxCode[i1] as u64));
}
for i1 in 0..18 {
out.push(("THPFileInfo.huffmanTabs[].valPtr[]".into(), (*info).huffmanTabs[i0].valPtr[i1] as u64));
}
out.push(("THPFileInfo.huffmanTabs[].Vij1".into(), (*info).huffmanTabs[i0].Vij1 as u64));
for i1 in 0..3 {
out.push(("THPFileInfo.huffmanTabs[].padD5[]".into(), (*info).huffmanTabs[i0].padD5[i1] as u64));
}
out.push(("THPFileInfo.huffmanTabs[].bits".into(), if (*info).huffmanTabs[i0].bits.is_null() {0} else {((*info).huffmanTabs[i0].bits as usize).wrapping_sub(base) as u64}));
out.push(("THPFileInfo.huffmanTabs[].sizeTab".into(), if (*info).huffmanTabs[i0].sizeTab.is_null() {0} else {((*info).huffmanTabs[i0].sizeTab as usize).wrapping_sub(base) as u64}));
out.push(("THPFileInfo.huffmanTabs[].codeTab".into(), if (*info).huffmanTabs[i0].codeTab.is_null() {0} else {((*info).huffmanTabs[i0].codeTab as usize).wrapping_sub(base) as u64}));
out.push(("THPFileInfo.huffmanTabs[].numCodes".into(), (*info).huffmanTabs[i0].numCodes as u64));
out.push(("THPFileInfo.huffmanTabs[].numVij".into(), (*info).huffmanTabs[i0].numVij as u64));
out.push(("THPFileInfo.huffmanTabs[].padEA".into(), (*info).huffmanTabs[i0].padEA as u64));
}
for i0 in 0..3 {
for i1 in 0..64 {
out.push(("THPFileInfo.quantTabs[][]".into(), (*info).quantTabs[i0][i1].to_bits() as u64));
}
}
for i0 in 0..256 {
out.push(("THPFileInfo.pad738[]".into(), (*info).pad738[i0] as u64));
}
for i0 in 0..3 {
out.push(("THPFileInfo.components[].componentID".into(), (*info).components[i0].componentID as u64));
out.push(("THPFileInfo.components[].samplingH".into(), (*info).components[i0].samplingH as u64));
out.push(("THPFileInfo.components[].samplingV".into(), (*info).components[i0].samplingV as u64));
out.push(("THPFileInfo.components[].quantizationTableSelector".into(), (*info).components[i0].quantizationTableSelector as u64));
out.push(("THPFileInfo.components[].DCTableSelector".into(), (*info).components[i0].DCTableSelector as u64));
out.push(("THPFileInfo.components[].ACTableSelector".into(), (*info).components[i0].ACTableSelector as u64));
out.push(("THPFileInfo.components[].predDC".into(), (*info).components[i0].predDC as u64));
out.push(("THPFileInfo.components[].x08".into(), (*info).components[i0].x08 as u64));
out.push(("THPFileInfo.components[].x0C".into(), (*info).components[i0].x0C as u64));
out.push(("THPFileInfo.components[].x10".into(), if (*info).components[i0].x10 == 0 {0} else {(*info).components[i0].x10.wrapping_sub(base as u32) as u64}));
out.push(("THPFileInfo.components[].x14".into(), (*info).components[i0].x14 as u64));
out.push(("THPFileInfo.components[].x18".into(), (*info).components[i0].x18 as u64));
out.push(("THPFileInfo.components[].x1C".into(), (*info).components[i0].x1C as u64));
out.push(("THPFileInfo.components[].x20".into(), (*info).components[i0].x20 as u64));
out.push(("THPFileInfo.components[].x24".into(), (*info).components[i0].x24 as u64));
out.push(("THPFileInfo.components[].x28".into(), (*info).components[i0].x28 as u64));
}
for i0 in 0..16 {
out.push(("THPFileInfo.x8BC[]".into(), (*info).x8BC[i0] as u64));
}
out.push(("THPFileInfo.MCUsPerRow".into(), (*info).MCUsPerRow as u64));
out.push(("THPFileInfo.x8CE".into(), (*info).x8CE as u64));
out.push(("THPFileInfo.x8D0".into(), (*info).x8D0 as u64));
out.push(("THPFileInfo.x8D2".into(), (*info).x8D2 as u64));
out.push(("THPFileInfo.x8D4".into(), (*info).x8D4 as u64));
for i0 in 0..18 {
out.push(("THPFileInfo.pad8D6[]".into(), (*info).pad8D6[i0] as u64));
}
out.push(("THPFileInfo.x8E8".into(), (*info).x8E8 as u64));
out.push(("THPFileInfo.x8EA".into(), (*info).x8EA as u64));
out.push(("THPFileInfo.x8EC".into(), (*info).x8EC as u64));
out.push(("THPFileInfo.x8EE".into(), (*info).x8EE as u64));
out.push(("THPFileInfo.tileY".into(), if (*info).tileY.is_null() {0} else {((*info).tileY as usize).wrapping_sub(base) as u64}));
out.push(("THPFileInfo.tileU".into(), if (*info).tileU.is_null() {0} else {((*info).tileU as usize).wrapping_sub(base) as u64}));
out.push(("THPFileInfo.tileV".into(), if (*info).tileV.is_null() {0} else {((*info).tileV as usize).wrapping_sub(base) as u64}));
out.push(("THPFileInfo.nMCU".into(), (*info).nMCU as u64));
out.push(("THPFileInfo.currMCU".into(), (*info).currMCU as u64));
out.push(("THPFileInfo.RST".into(), (*info).RST as u64));
for i0 in 0..3 {
out.push(("THPFileInfo.pad901[]".into(), (*info).pad901[i0] as u64));
}
out.push(("THPFileInfo.scratch".into(), if (*info).scratch.is_null() {0} else {((*info).scratch as usize).wrapping_sub(base) as u64}));
out
}
