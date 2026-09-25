#include "../platform/gw_slippi_wire.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

static void pad_vector(void) {
  GwSlippiWirePad p = {0}, q = {0};
  uint8_t buf[GW_SLIPPI_MAX_PACKET];
  const uint8_t expected[] = {0x80,0,0,0,3,1,0,0,0,2,0x12,0x34,0x56,0x78,
                              1,2,3,4,5,6,7,8, 9,10,11,12,13,14,15,16};
  int i;
  p.frame=3; p.player_idx=1; p.checksum_frame=2; p.checksum=0x12345678; p.count=2;
  for (i=0;i<8;i++) { p.pads[0][i]=(uint8_t)(i+1); p.pads[1][i]=(uint8_t)(i+9); }
  assert(gw_slippi_wire_encode_pad(buf,sizeof buf,&p)==sizeof expected);
  assert(memcmp(buf,expected,sizeof expected)==0);
  assert(gw_slippi_wire_decode_pad(buf,sizeof expected,1,1,10,&q));
  assert(q.count==2 && q.frame==3 && memcmp(q.pads,p.pads,16)==0);
  assert(!gw_slippi_wire_decode_pad(buf,sizeof expected-1,1,1,10,&q));
  assert(!gw_slippi_wire_decode_pad(buf,sizeof expected,0,1,10,&q));
  assert(!gw_slippi_wire_decode_pad(buf,sizeof expected,1,4,10,&q));
  assert(!gw_slippi_wire_decode_pad(buf,sizeof expected,1,1,2,&q));
  assert(!gw_slippi_wire_decode_pad(buf,14,1,1,10,&q));
}
static void ack_vector(void) {
  uint8_t buf[6]; int32_t frame=0;
  const uint8_t expected[]={0x81,0,0,0,3,0};
  assert(gw_slippi_wire_encode_ack(buf,sizeof buf,3,0)==6);
  assert(memcmp(buf,expected,6)==0);
  assert(gw_slippi_wire_decode_ack(buf,6,0,1,10,&frame) && frame==3);
  assert(!gw_slippi_wire_decode_ack(buf,5,0,1,10,&frame));
  assert(!gw_slippi_wire_decode_ack(buf,6,1,1,10,&frame));
}
static void string_vector(void) {
  uint8_t buf[10]; char value[4]; size_t used=0;
  const uint8_t expected[]={0,0,0,3,'a','b','c'};
  assert(gw_slippi_wire_encode_string(buf,sizeof buf,"abc",3)==7);
  assert(memcmp(buf,expected,7)==0);
  assert(gw_slippi_wire_decode_string(buf,7,value,sizeof value,&used));
  assert(used==7 && strcmp(value,"abc")==0);
  assert(!gw_slippi_wire_decode_string(buf,6,value,sizeof value,&used));
  assert(!gw_slippi_wire_decode_string(buf,7,value,3,&used));
  memset(buf,0xff,4);
  assert(!gw_slippi_wire_decode_string(buf,4,value,sizeof value,&used));
}
int main(void) { pad_vector(); ack_vector(); string_vector(); puts("slippi wire vectors passed"); return 0; }
