/* SPDX-License-Identifier: GPL-2.0-or-later
 * Single-threaded ENet transport for Slippi peer packets. Wire behavior follows
 * Project Slippi Dolphin SlippiNetplay.cpp at 41a7a3a110ed52999486ae1901c8fbb9a63d4f13.
 * https://github.com/project-slippi/dolphin/blob/41a7a3a110ed52999486ae1901c8fbb9a63d4f13/Source/Core/Core/Slippi/SlippiNetplay.cpp
 * ENet is vendored separately under its MIT license in extern/enet.
 */
#include "gw_slippi_peer.h"
#include "gw_slippi_enet.h"
#include <stdlib.h>
#include <string.h>

typedef struct PadEntry { int32_t frame, checksum_frame; uint32_t checksum; uint8_t bytes[8]; } PadEntry;
struct GwSlippiPeer {
  GwSlippiPeerConfig cfg;
  ENetHost *host;
  ENetPeer *active;
  ENetAddress remote;
  PadEntry queue[GW_SLIPPI_MAX_PADS]; /* newest first */
  unsigned queued;
  enet_uint32 last_resend_ms;
  GwSlippiPeerStats stats;
};
static unsigned enet_refs;

static int send_bytes(GwSlippiPeer *p,const uint8_t *data,size_t len,uint8_t channel,int reliable) {
  ENetPacket *pkt;
  if (!p || !p->active || p->active->state!=ENET_PEER_STATE_CONNECTED || !data || !len) return 0;
  pkt=enet_packet_create(data,len,reliable?ENET_PACKET_FLAG_RELIABLE:ENET_PACKET_FLAG_UNSEQUENCED);
  if (!pkt) return 0;
  if (enet_peer_send(p->active,channel,pkt)<0) { enet_packet_destroy(pkt); return 0; }
  enet_host_flush(p->host);
  p->stats.packets_sent++;
  return 1;
}

static int send_queued(GwSlippiPeer *p,int resend) {
  GwSlippiWirePad wire;
  uint8_t bytes[GW_SLIPPI_MAX_PACKET];
  size_t n; unsigned i;
  if (!p || !p->queued || !p->active) return 0;
  memset(&wire,0,sizeof wire);
  wire.frame=p->queue[0].frame; wire.player_idx=p->cfg.local_port_idx;
  wire.checksum_frame=p->queue[0].checksum_frame; wire.checksum=p->queue[0].checksum;
  wire.count=(uint8_t)p->queued;
  for (i=0;i<p->queued;i++) memcpy(wire.pads[i],p->queue[i].bytes,8);
  n=gw_slippi_wire_encode_pad(bytes,sizeof bytes,&wire);
  if (!n || !send_bytes(p,bytes,n,1,0)) return 0;
  if (resend) p->stats.retransmits++;
  p->last_resend_ms=enet_time_get();
  return 1;
}

GwSlippiPeer *gw_slippi_peer_start(const GwSlippiPeerConfig *cfg) {
  GwSlippiPeer *p;
  ENetAddress local;
  if (!cfg || !cfg->remote_host || !cfg->local_udp_port || !cfg->remote_udp_port ||
      cfg->local_port_idx>3 || cfg->remote_port_idx>3 ||
      cfg->local_port_idx==cfg->remote_port_idx) return NULL;
  if (!enet_refs && enet_initialize()!=0) return NULL;
  enet_refs++;
  p=(GwSlippiPeer*)calloc(1,sizeof *p);
  if (!p) goto fail;
  p->cfg=*cfg;
  memset(&local,0,sizeof local);
  local.port=cfg->local_udp_port;
  if (cfg->loopback_mode) {
    if (enet_address_set_host_ip(&local,"127.0.0.1")<0) goto fail;
  } else local.host=ENET_HOST_ANY;
  if (enet_address_set_host(&p->remote,cfg->remote_host)<0) goto fail;
  p->remote.port=cfg->remote_udp_port;
  p->host=enet_host_create(&local,4,3,0,0);
  if (!p->host) goto fail;
  p->active=enet_host_connect(p->host,&p->remote,3,0);
  if (!p->active) goto fail;
  return p;
fail:
  if (p) { if (p->host) enet_host_destroy(p->host); free(p); }
  if (--enet_refs==0) enet_deinitialize();
  return NULL;
}

static int valid_peer(const GwSlippiPeer *p,const ENetPeer *remote) {
  /* NAT may remap the source UDP port after matchmaking. Dolphin accepts the
   * assigned host's incoming connection and binds the resulting ENetPeer. */
  return remote && remote->address.host==p->remote.host;
}

static ENetPeer *connected_alternative(const GwSlippiPeer *p,const ENetPeer *except) {
  size_t i;
  for (i=0;i<p->host->peerCount;i++) {
    ENetPeer *candidate=&p->host->peers[i];
    if (candidate!=except && candidate->state==ENET_PEER_STATE_CONNECTED &&
        valid_peer(p,candidate)) return candidate;
  }
  return NULL;
}

static void receive_pad(GwSlippiPeer *p,const uint8_t *data,size_t len,int current) {
  GwSlippiWirePad w;
  uint8_t ack[6];
  int min_frame=current>128?current-128:1;
  int max_frame=current<0?7:(current>INT32_MAX-7?INT32_MAX:current+7);
  int i,first;
  if (!gw_slippi_wire_decode_pad(data,len,p->cfg.remote_port_idx,min_frame,max_frame,&w)) {
    p->stats.packets_rejected++; return;
  }
  first=w.frame-(int)w.count+1;
  for (i=first;i<=w.frame;i++) {
    int index=w.frame-i;
    if (i<min_frame || i<=p->stats.last_received_frame) continue;
    if (i!=p->stats.last_received_frame+1) { p->stats.packets_rejected++; return; }
    if (!p->cfg.on_remote_pad || !p->cfg.on_remote_pad(p->cfg.user,i,w.pads[index])) {
      p->stats.packets_rejected++; break;
    }
    p->stats.last_received_frame=i; p->stats.pads_received++;
  }
  if (w.checksum_frame>=0 && w.checksum_frame<=p->stats.last_received_frame) {
    if (w.checksum_frame>p->stats.remote_checksum_frame) {
      p->stats.remote_checksum_frame=w.checksum_frame;
      p->stats.remote_checksum=w.checksum;
      if (p->cfg.on_remote_checksum)
        p->cfg.on_remote_checksum(p->cfg.user,w.checksum_frame,w.checksum);
    }
  }
  if (p->stats.last_received_frame>0 &&
      gw_slippi_wire_encode_ack(ack,sizeof ack,p->stats.last_received_frame,p->cfg.local_port_idx)) {
    if (send_bytes(p,ack,sizeof ack,2,0)) p->stats.acks_sent++;
  }
}

static void receive_ack(GwSlippiPeer *p,const uint8_t *data,size_t len) {
  int32_t frame;
  if (!gw_slippi_wire_decode_ack(data,len,p->cfg.remote_port_idx,1,p->stats.last_sent_frame,&frame)) {
    p->stats.packets_rejected++; return;
  }
  if (frame>p->stats.last_acked_frame) {
    p->stats.last_acked_frame=frame;
    while (p->queued && p->queue[p->queued-1].frame<=frame) p->queued--;
    p->stats.queued_local=p->queued;
  }
  p->stats.acks_received++;
}

static void receive_control(GwSlippiPeer *p,const uint8_t *data,size_t len) {
  GwSlippiWireSelections s;
  GwSlippiWirePrep prep;
  if (!len) { p->stats.packets_rejected++; return; }
  switch (data[0]) {
  case GW_SLIPPI_WIRE_SELECTIONS:
    if (!gw_slippi_wire_decode_selections(data,len,&s) || s.player_idx!=p->cfg.remote_port_idx)
      p->stats.packets_rejected++;
    else if (p->cfg.on_selections) p->cfg.on_selections(p->cfg.user,&s);
    break;
  case GW_SLIPPI_WIRE_PREP:
    if (!gw_slippi_wire_decode_prep(data,len,&prep)) p->stats.packets_rejected++;
    else if (p->cfg.on_prep) p->cfg.on_prep(p->cfg.user,&prep);
    break;
  case GW_SLIPPI_WIRE_CONN_SELECTED:
    if (!gw_slippi_wire_decode_conn_selected(data,len)) p->stats.packets_rejected++;
    else if (p->cfg.on_connection_selected) p->cfg.on_connection_selected(p->cfg.user);
    break;
  default: p->stats.packets_rejected++; break;
  }
}

void gw_slippi_peer_poll(GwSlippiPeer *p,int current_online_frame) {
  ENetEvent ev;
  int rc,iterations=0;
  if (!p) return;
  while (iterations++<128 && (rc=enet_host_service(p->host,&ev,0))>0) {
    if (!valid_peer(p,ev.peer)) {
      if (ev.type==ENET_EVENT_TYPE_CONNECT) enet_peer_disconnect_now(ev.peer,0);
      if (ev.type==ENET_EVENT_TYPE_RECEIVE) enet_packet_destroy(ev.packet);
      p->stats.packets_rejected++; continue;
    }
    if (ev.type==ENET_EVENT_TYPE_CONNECT) {
      if (!p->active || p->active->state!=ENET_PEER_STATE_CONNECTED) p->active=ev.peer;
      p->stats.connected=1; send_queued(p,0);
    } else if (ev.type==ENET_EVENT_TYPE_DISCONNECT) {
      if (p->active==ev.peer) p->active=connected_alternative(p,ev.peer);
      p->stats.connected=p->active!=NULL;
    } else if (ev.type==ENET_EVENT_TYPE_RECEIVE) {
      const uint8_t *data=ev.packet->data; size_t len=ev.packet->dataLength;
      p->stats.packets_received++;
      if (len && data[0]==GW_SLIPPI_WIRE_PAD && ev.channelID==1)
        receive_pad(p,data,len,current_online_frame);
      else if (len && data[0]==GW_SLIPPI_WIRE_ACK && ev.channelID==2)
        receive_ack(p,data,len);
      else if (ev.channelID==0) receive_control(p,data,len);
      else p->stats.packets_rejected++;
      enet_packet_destroy(ev.packet);
    }
  }
  if (p->queued && p->active && (enet_uint32)(enet_time_get()-p->last_resend_ms)>=25u)
    send_queued(p,1);
}

int gw_slippi_peer_send_pad(GwSlippiPeer *p,int frame,const uint8_t pad[8],
                            int checksum_frame,uint32_t checksum) {
  if (!p || !pad || frame<1 || frame!=p->stats.last_sent_frame+1 ||
      checksum_frame>frame) return 0;
  /* Evicting an unacknowledged frame makes the receiver's contiguous stream
   * unrecoverable. Let the caller stop or wait for an ACK instead. */
  if (p->queued==GW_SLIPPI_MAX_PADS) return 0;
  memmove(p->queue+1,p->queue,p->queued*sizeof p->queue[0]);
  p->queue[0].frame=frame; p->queue[0].checksum_frame=checksum_frame;
  p->queue[0].checksum=checksum; memcpy(p->queue[0].bytes,pad,8);
  p->queued++; p->stats.queued_local=p->queued;
  p->stats.last_sent_frame=frame; p->stats.pads_sent++;
  send_queued(p,0);
  return 1;
}
int gw_slippi_peer_send_selections(GwSlippiPeer *p,const GwSlippiWireSelections *s) {
  uint8_t b[14]; size_t n;
  if (!p || !s || s->player_idx!=p->cfg.local_port_idx) return 0;
  n=gw_slippi_wire_encode_selections(b,sizeof b,s); return n && send_bytes(p,b,n,0,1);
}
int gw_slippi_peer_send_prep(GwSlippiPeer *p,const GwSlippiWirePrep *s) {
  uint8_t b[6]; size_t n=gw_slippi_wire_encode_prep(b,sizeof b,s);
  return n && send_bytes(p,b,n,0,1);
}
int gw_slippi_peer_send_connection_selected(GwSlippiPeer *p) {
  uint8_t b[1]; size_t n=gw_slippi_wire_encode_conn_selected(b,sizeof b);
  return n && send_bytes(p,b,n,0,1);
}
void gw_slippi_peer_stats(const GwSlippiPeer *p,GwSlippiPeerStats *out) {
  if (p && out) *out=p->stats;
}
void gw_slippi_peer_close(GwSlippiPeer *p) {
  if (!p) return;
  if (p->active) enet_peer_disconnect_now(p->active,0);
  enet_host_destroy(p->host); free(p);
  if (--enet_refs==0) enet_deinitialize();
}
