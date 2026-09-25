/* SPDX-License-Identifier: GPL-2.0-or-later
 * Slippi peer wire encoding. Derived from Dolphin SlippiNetplay.cpp at
 * 41a7a3a110ed52999486ae1901c8fbb9a63d4f13 (GPL-2.0-or-later).
 * https://github.com/project-slippi/dolphin/blob/41a7a3a110ed52999486ae1901c8fbb9a63d4f13/Source/Core/Core/Slippi/SlippiNetplay.cpp
 */
#include "gw_slippi_wire.h"
#include <string.h>

static void put32(uint8_t *p, uint32_t v) {
  p[0]=(uint8_t)(v>>24); p[1]=(uint8_t)(v>>16); p[2]=(uint8_t)(v>>8); p[3]=(uint8_t)v;
}
static uint32_t get32(const uint8_t *p) {
  return ((uint32_t)p[0]<<24)|((uint32_t)p[1]<<16)|((uint32_t)p[2]<<8)|p[3];
}

size_t gw_slippi_wire_encode_pad(uint8_t *out, size_t cap, const GwSlippiWirePad *p) {
  size_t len;
  if (!out || !p || p->player_idx>3 || p->count==0 || p->count>GW_SLIPPI_MAX_PADS || p->frame<1)
    return 0;
  len=GW_SLIPPI_PAD_HEADER+(size_t)p->count*GW_SLIPPI_PAD_BYTES;
  if (cap<len || (int64_t)p->frame-p->count+1<1) return 0;
  out[0]=GW_SLIPPI_WIRE_PAD; put32(out+1,(uint32_t)p->frame); out[5]=p->player_idx;
  put32(out+6,(uint32_t)p->checksum_frame); put32(out+10,p->checksum);
  memcpy(out+14,p->pads,(size_t)p->count*8);
  return len;
}

int gw_slippi_wire_decode_pad(const uint8_t *data, size_t len, uint8_t expected,
                              int32_t min_frame, int32_t max_frame, GwSlippiWirePad *out) {
  GwSlippiWirePad tmp;
  size_t count;
  if (!data || !out || expected>3 || len<22 || len>GW_SLIPPI_MAX_PACKET ||
      data[0]!=GW_SLIPPI_WIRE_PAD || data[5]!=expected || (len-14)%8) return 0;
  count=(len-14)/8;
  if (!count || count>GW_SLIPPI_MAX_PADS) return 0;
  tmp.frame=(int32_t)get32(data+1);
  if (min_frame<1 || max_frame<min_frame || tmp.frame<min_frame || tmp.frame>max_frame ||
      (int64_t)tmp.frame-(int64_t)count+1<1) return 0;
  tmp.player_idx=data[5]; tmp.checksum_frame=(int32_t)get32(data+6);
  tmp.checksum=get32(data+10); tmp.count=(uint8_t)count;
  memcpy(tmp.pads,data+14,count*8);
  *out=tmp;
  return 1;
}

size_t gw_slippi_wire_encode_ack(uint8_t *out, size_t cap, int32_t frame, uint8_t idx) {
  if (!out || cap<6 || idx>3 || frame<1) return 0;
  out[0]=GW_SLIPPI_WIRE_ACK; put32(out+1,(uint32_t)frame); out[5]=idx; return 6;
}
int gw_slippi_wire_decode_ack(const uint8_t *data, size_t len, uint8_t expected,
                              int32_t min_frame, int32_t max_frame, int32_t *frame) {
  int32_t f;
  if (!data || !frame || len!=6 || data[0]!=GW_SLIPPI_WIRE_ACK || data[5]!=expected) return 0;
  f=(int32_t)get32(data+1);
  if (f<min_frame || f>max_frame) return 0;
  *frame=f; return 1;
}

size_t gw_slippi_wire_encode_selections(uint8_t *out,size_t cap,const GwSlippiWireSelections *s) {
  if (!out || !s || cap<14 || s->player_idx>3) return 0;
  out[0]=GW_SLIPPI_WIRE_SELECTIONS; out[1]=s->character_id; out[2]=s->character_color;
  out[3]=!!s->character_selected; out[4]=s->player_idx;
  out[5]=(uint8_t)(s->stage_id>>8); out[6]=(uint8_t)s->stage_id;
  out[7]=!!s->stage_selected; put32(out+8,s->rng_offset);
  out[12]=s->team_id; out[13]=s->alt_stage_mode; return 14;
}
int gw_slippi_wire_decode_selections(const uint8_t *d,size_t n,GwSlippiWireSelections *s) {
  if (!d || !s || n!=14 || d[0]!=GW_SLIPPI_WIRE_SELECTIONS || d[4]>3 || d[3]>1 || d[7]>1)
    return 0;
  s->character_id=d[1]; s->character_color=d[2]; s->character_selected=d[3];
  s->player_idx=d[4]; s->stage_id=(uint16_t)(((uint16_t)d[5]<<8)|d[6]);
  s->stage_selected=d[7]; s->rng_offset=get32(d+8); s->team_id=d[12];
  s->alt_stage_mode=d[13]; return 1;
}
size_t gw_slippi_wire_encode_prep(uint8_t *d,size_t cap,const GwSlippiWirePrep *s) {
  if (!d || !s || cap<6) return 0;
  d[0]=GW_SLIPPI_WIRE_PREP; d[1]=s->step_idx; d[2]=s->character_id;
  d[3]=s->character_color; d[4]=s->stage_selections[0]; d[5]=s->stage_selections[1]; return 6;
}
int gw_slippi_wire_decode_prep(const uint8_t *d,size_t n,GwSlippiWirePrep *s) {
  if (!d || !s || n!=6 || d[0]!=GW_SLIPPI_WIRE_PREP) return 0;
  s->step_idx=d[1]; s->character_id=d[2]; s->character_color=d[3];
  s->stage_selections[0]=d[4]; s->stage_selections[1]=d[5]; return 1;
}
size_t gw_slippi_wire_encode_conn_selected(uint8_t *d,size_t cap) {
  if (!d || !cap) return 0; d[0]=GW_SLIPPI_WIRE_CONN_SELECTED; return 1;
}
int gw_slippi_wire_decode_conn_selected(const uint8_t *d,size_t n) {
  return d && n==1 && d[0]==GW_SLIPPI_WIRE_CONN_SELECTED;
}
size_t gw_slippi_wire_encode_string(uint8_t *out,size_t cap,const char *str,size_t len) {
  if (!out || (!str && len) || len>UINT32_MAX || cap<4 || len>cap-4) return 0;
  put32(out,(uint32_t)len); if (len) memcpy(out+4,str,len); return len+4;
}
int gw_slippi_wire_decode_string(const uint8_t *data,size_t len,char *out,size_t cap,
                                 size_t *consumed) {
  uint32_t n;
  if (!data || !out || !consumed || len<4 || !cap) return 0;
  n=get32(data);
  if (n>len-4 || n>=cap) return 0;
  if (n) memcpy(out,data+4,n);
  out[n]=0; *consumed=(size_t)n+4; return 1;
}
