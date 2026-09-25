#include "../platform/gw_slippi_peer.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <windows.h>

typedef struct Seen { int count,last_frame; uint8_t last_pad[8]; int checksum_frame; uint32_t checksum;
                      int selections,preps,selected; } Seen;
static void on_pad(void *user,int frame,const uint8_t pad[8]) {
  Seen *s=(Seen*)user; s->count++; s->last_frame=frame; memcpy(s->last_pad,pad,8);
}
static void on_checksum(void *user,int frame,uint32_t checksum) {
  Seen *s=(Seen*)user; s->checksum_frame=frame; s->checksum=checksum;
}
static void on_selections(void *user,const GwSlippiWireSelections *s) {
  Seen *seen=(Seen*)user; assert(s->player_idx==0); seen->selections++;
}
static void on_prep(void *user,const GwSlippiWirePrep *s) {
  Seen *seen=(Seen*)user; assert(s->step_idx==2); seen->preps++;
}
static void on_selected(void *user) { ((Seen*)user)->selected++; }
static int child(int role) {
  Seen seen={0}; GwSlippiPeerConfig cfg={0}; GwSlippiPeerStats st={0};
  GwSlippiPeer *p; uint8_t pad1[8]={1,2,3,4,5,6,7,8},pad2[8]={9,8,7,6,5,4,3,2};
  int i,sent=0;
  cfg.remote_host="127.0.0.1"; cfg.local_udp_port=(uint16_t)(49293+role);
  cfg.remote_udp_port=(uint16_t)(49294-role); cfg.local_port_idx=(uint8_t)role;
  cfg.remote_port_idx=(uint8_t)(1-role); cfg.match_id="two-process";
  cfg.loopback_mode=1; cfg.user=&seen; cfg.on_remote_pad=on_pad;
  cfg.on_remote_checksum=on_checksum;
  p=gw_slippi_peer_start(&cfg); if (!p) return 2;
  for (i=0;i<600;i++) {
    gw_slippi_peer_poll(p,2); gw_slippi_peer_stats(p,&st);
    if (role==0 && st.connected && !sent) {
      if (!gw_slippi_peer_send_pad(p,1,pad1,0,0x12345678) ||
          !gw_slippi_peer_send_pad(p,2,pad2,1,0x87654321)) return 3;
      sent=1;
    }
    if ((role==0 && sent && st.last_acked_frame==2) || (role==1 && seen.count==2)) break;
    Sleep(5);
  }
  if (role==1 && !(seen.count==2 && seen.last_frame==2 &&
                   memcmp(seen.last_pad,pad2,8)==0 && seen.checksum==0x87654321)) return 4;
  if (role==0 && st.last_acked_frame!=2) return 5;
  /* The receiver has queued an unsequenced ACK; let ENet service it before closing. */
  if (role==1) for (i=0;i<40;i++) { gw_slippi_peer_poll(p,2); Sleep(5); }
  gw_slippi_peer_close(p); return 0;
}
static void two_process(void) {
  char path[MAX_PATH],cmd_a[MAX_PATH+20],cmd_b[MAX_PATH+20];
  STARTUPINFOA si={0}; PROCESS_INFORMATION a={0},b={0}; DWORD rc;
  assert(GetModuleFileNameA(NULL,path,sizeof path)>0);
  sprintf(cmd_a,"\"%s\" --child-a",path); sprintf(cmd_b,"\"%s\" --child-b",path);
  si.cb=sizeof si;
  assert(CreateProcessA(path,cmd_a,NULL,NULL,FALSE,CREATE_NO_WINDOW,NULL,NULL,&si,&a));
  assert(CreateProcessA(path,cmd_b,NULL,NULL,FALSE,CREATE_NO_WINDOW,NULL,NULL,&si,&b));
  assert(WaitForSingleObject(a.hProcess,5000)==WAIT_OBJECT_0);
  assert(WaitForSingleObject(b.hProcess,5000)==WAIT_OBJECT_0);
  assert(GetExitCodeProcess(a.hProcess,&rc)); fprintf(stderr,"child A exit=%lu\n",(unsigned long)rc);
  assert(GetExitCodeProcess(b.hProcess,&rc)); fprintf(stderr,"child B exit=%lu\n",(unsigned long)rc);
  assert(GetExitCodeProcess(a.hProcess,&rc) && rc==0);
  assert(GetExitCodeProcess(b.hProcess,&rc) && rc==0);
  CloseHandle(a.hThread); CloseHandle(a.hProcess); CloseHandle(b.hThread); CloseHandle(b.hProcess);
}
int main(int argc,char **argv) {
  Seen a={0},b={0}; GwSlippiPeerConfig ca={0},cb={0}; GwSlippiPeer *pa,*pb;
  GwSlippiPeerStats sa,sb; uint8_t pad1[8]={1,2,3,4,5,6,7,8},pad2[8]={9,8,7,6,5,4,3,2};
  GwSlippiWireSelections sel={0}; GwSlippiWirePrep prep={0}; int i;
  if (argc==2 && strcmp(argv[1],"--child-a")==0) return child(0);
  if (argc==2 && strcmp(argv[1],"--child-b")==0) return child(1);
  ca.remote_host=cb.remote_host="127.0.0.1";
  ca.local_udp_port=49291; ca.remote_udp_port=49292; ca.local_port_idx=0; ca.remote_port_idx=1;
  cb.local_udp_port=49292; cb.remote_udp_port=49291; cb.local_port_idx=1; cb.remote_port_idx=0;
  ca.loopback_mode=cb.loopback_mode=1; ca.match_id=cb.match_id="local-test";
  ca.user=&a; cb.user=&b; ca.on_remote_pad=cb.on_remote_pad=on_pad;
  ca.on_remote_checksum=cb.on_remote_checksum=on_checksum;
  cb.on_selections=on_selections; cb.on_prep=on_prep; cb.on_connection_selected=on_selected;
  pa=gw_slippi_peer_start(&ca); pb=gw_slippi_peer_start(&cb); assert(pa && pb);
  for (i=0;i<200;i++) {
    gw_slippi_peer_poll(pa,1); gw_slippi_peer_poll(pb,1);
    gw_slippi_peer_stats(pa,&sa); gw_slippi_peer_stats(pb,&sb);
    if (sa.connected && sb.connected) break;
    Sleep(5);
  }
  assert(sa.connected && sb.connected);
  assert(gw_slippi_peer_send_pad(pa,1,pad1,0,0x12345678));
  assert(gw_slippi_peer_send_pad(pa,2,pad2,1,0x87654321));
  /* The receiver is deliberately not polled: unacknowledged history must resend. */
  for (i=0;i<12;i++) { gw_slippi_peer_poll(pa,2); Sleep(5); }
  gw_slippi_peer_stats(pa,&sa); assert(sa.retransmits>0 && sa.queued_local==2);
  for (i=0;i<200;i++) {
    gw_slippi_peer_poll(pa,2); gw_slippi_peer_poll(pb,2);
    gw_slippi_peer_stats(pa,&sa);
    if (b.count>=2 && sa.last_acked_frame>=2) break;
    Sleep(5);
  }
  assert(b.count==2 && b.last_frame==2 && memcmp(b.last_pad,pad2,8)==0);
  assert(b.checksum_frame==1 && b.checksum==0x87654321);
  assert(sa.last_acked_frame==2 && sa.queued_local==0);
  sel.player_idx=0; sel.character_id=2; sel.stage_id=31; sel.stage_selected=1;
  prep.step_idx=2; prep.character_id=2; prep.stage_selections[0]=31;
  assert(gw_slippi_peer_send_selections(pa,&sel));
  assert(gw_slippi_peer_send_prep(pa,&prep));
  assert(gw_slippi_peer_send_connection_selected(pa));
  for (i=0;i<100;i++) { gw_slippi_peer_poll(pa,2); gw_slippi_peer_poll(pb,2);
    if (b.selections && b.preps && b.selected) break; Sleep(5); }
  assert(b.selections==1 && b.preps==1 && b.selected==1);
  for (i=3;i<=200;i++) assert(gw_slippi_peer_send_pad(pa,i,pad1,i-1,(uint32_t)i));
  gw_slippi_peer_stats(pa,&sa); assert(sa.queued_local<=128);
  gw_slippi_peer_close(pa); gw_slippi_peer_close(pb);
  two_process();
  puts("slippi ENet loopback passed"); return 0;
}
