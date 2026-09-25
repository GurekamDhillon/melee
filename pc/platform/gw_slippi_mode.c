/* SPDX-License-Identifier: GPL-2.0-or-later
 * Explicit Slippi replay-driver mode. This adapter only owns local fixture pads;
 * remote truth enters rollback exclusively in the ENet receive callback.
 * Direct uses official matchmaking, followed by the same peer path as loopback.
 * It never uploads a recording or match report. GD/GD acceptance does not prove
 * compatibility with an unmodified Dolphin opponent.
 */
#include "gw.h"
#include "gw_slippi_mode.h"
#include "gw_slippi_mode_config.h"
#include "gw_slippi_pad.h"
#include "gw_slippi_peer.h"
#include "gw_slippi_match.h"
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <wincrypt.h>
#include <stdio.h>

extern const char *gw_replay_scene(void);
extern int gw_Replay_Frame(void);
extern int gw_Replay_SlippiLocalMismatches(void);
extern int gw_Replay_FinishRecording(void);

static struct {
    int tried, active, selected, got_selections, sent_selections, sent_selected;
    int started, complete, last_simulated, local_port, remote_port;
    int got_remote, remote_changes, pending_error;
    DWORD progress_ms, complete_ms;
    SmConfig config;
    GwSlippiFixtureInfo fixture;
    GwSlippiPeer *peer;
    GwSlippiWireSelections local_selection, remote_selection;
    GwSlippiPad last_remote;
    char match_id[128], run_salt[33], account_tag[65], peer_account_tag[65];
    struct { int frame; uint32_t value; } checksums[64];
} sm;

static int sm_hex(char c) {
    return c>='0'&&c<='9'?c-'0':c>='a'&&c<='f'?c-'a'+10:-1;
}
static int sm_account_tag(uint64_t fingerprint, const char *salt, char out[65]) {
    static const char domain[]="GD Slippi account v1";
    static const char hex[]="0123456789abcdef";
    HCRYPTPROV provider=0;
    HCRYPTHASH hash=0;
    BYTE bytes[24], digest[32];
    DWORD size=sizeof digest;
    int i,ok=0;
    if (!fingerprint || !salt || strlen(salt)!=32) return 0;
    for (i=0;i<16;i++) {
        int hi=sm_hex(salt[i*2]),lo=sm_hex(salt[i*2+1]);
        if (hi<0 || lo<0) return 0;
        bytes[i]=(BYTE)((hi<<4)|lo);
    }
    for (i=0;i<8;i++) bytes[16+i]=(BYTE)(fingerprint>>(56-8*i));
    if (!CryptAcquireContextA(&provider,NULL,NULL,PROV_RSA_AES,CRYPT_VERIFYCONTEXT) ||
        !CryptCreateHash(provider,CALG_SHA_256,0,0,&hash) ||
        !CryptHashData(hash,(const BYTE*)domain,sizeof domain,0) ||
        !CryptHashData(hash,bytes,sizeof bytes,0) ||
        !CryptGetHashParam(hash,HP_HASHVAL,digest,&size,0) || size!=32) goto done;
    for (i=0;i<32;i++) { out[2*i]=hex[digest[i]>>4]; out[2*i+1]=hex[digest[i]&15]; }
    out[64]=0; ok=1;
done:
    if (hash) CryptDestroyHash(hash);
    if (provider) CryptReleaseContext(provider,0);
    return ok;
}
static void sm_json_string(FILE *file, const char *value) {
    const unsigned char *s=(const unsigned char*)value;
    fputc('"',file);
    for (;*s;s++) {
        if (*s=='"' || *s=='\\') { fputc('\\',file); fputc(*s,file); }
        else if (*s<32) fprintf(file,"\\u%04x",(unsigned)*s);
        else fputc(*s,file);
    }
    fputc('"',file);
}
static int sm_evidence(void) {
    GwSlippiPeerStats stats={0};
    FILE *file;
    if (!sm.config.evidence) return 0;
    gw_slippi_peer_stats(sm.peer,&stats);
    file=fopen(sm.config.evidence,"wb");
    if (!file) return 0;
    fprintf(file,"{\n\"schema\":1,\"mode\":\"%s\",\"role\":%d,\"process_id\":%lu,\n",
            sm.config.direct?"direct":"loopback",sm.local_port+1,(unsigned long)GetCurrentProcessId());
    fprintf(file,"\"connection_selected\":%s,\"match_started\":%s,\"match_completed\":%s,\n",
            sm.selected?"true":"false",sm.started?"true":"false",sm.complete?"true":"false");
    fprintf(file,"\"first_frame\":%d,\"last_frame\":%d,\"confirmed_frame\":%d,\"input_delay\":%d,\n",
            sm.fixture.first_frame,sm.fixture.last_frame,gw_rb_confirmed_frame(),sm.config.delay);
    fprintf(file,"\"sent_local_frames\":%u,\"received_remote_frames\":%u,\n",
            stats.pads_sent,stats.pads_received);
    fprintf(file,"\"local_fixture_reads\":%d,\"remote_fixture_reads\":%d,\n",
            gw_rb_local_fixture_reads(sm.local_port),gw_rb_local_fixture_reads(sm.remote_port));
    fprintf(file,"\"pad_packets_received\":%u,\"ack_packets_received\":%u,\"remote_input_changes\":%d,\n",
            stats.pad_packets_received,stats.acks_received,sm.remote_changes);
    fprintf(file,"\"rollbacks\":%d,\"desyncs\":%d,\"packets_sent\":%u,\"packets_received\":%u,\n",
            gw_rb_rollbacks(),gw_rb_desyncs(),stats.packets_sent,stats.packets_received);
    fprintf(file,"\"last_acked_online_frame\":%d,\"match_id\":",stats.last_acked_frame);
    sm_json_string(file,sm.match_id);
    if (sm.config.direct) {
        fprintf(file,",\n\"run_salt\":\"%s\",\"account_tag\":\"%s\",\"peer_account_tag\":\"%s\"",
                sm.run_salt,sm.account_tag,sm.peer_account_tag);
    }
    fputs("\n}\n",file);
    { int failed=ferror(file); return fclose(file)==0 && !failed; }
}
static void sm_exit(int code, const char *reason) {
    if (!gw_Replay_FinishRecording() && !code) {
        gw_log("slippi: could not finalize recording"); code=2;
    }
    if (code) sm.complete=0;
    gw_log("slippi: %s (role %d, frame %d, confirmed %d)",reason,sm.local_port+1,
           gw_Replay_Frame(),gw_rb_confirmed_frame());
    if (!sm_evidence()) { gw_log("slippi: could not finalize evidence"); code=2; }
    gw_slippi_peer_close(sm.peer); sm.peer=NULL;
    gw_slippi_match_close();
    /* Like rp_parity_end: the finite run has flushed its artifacts. CRT static
     * destructors are unsafe for Dawn here (see shim_vi.c's gw_exit_clean). */
    fflush(NULL);
    _exit(code);
}
static int sm_remote_pad(void *user,int online_frame,const uint8_t bytes[8]) {
    GwSlippiPad pad;
    (void)user;
    memcpy(pad.bytes,bytes,8);
    if (!gw_rb_slippi_receive(gw_rb_epoch(),sm.remote_port,online_frame,&pad)) return 0;
    if (sm.got_remote && memcmp(sm.last_remote.bytes,pad.bytes,8)) sm.remote_changes++;
    sm.got_remote=1; sm.last_remote=pad;
    return 1;
}
static void sm_remote_checksum(void *user,int frame,uint32_t value) {
    unsigned slot=(unsigned)frame%64;
    (void)user;
    if (frame<1 || !value) return;
    if (sm.checksums[slot].value && sm.checksums[slot].frame!=frame) sm.pending_error=1;
    sm.checksums[slot].frame=frame; sm.checksums[slot].value=value;
}
static void sm_remote_selection(void *user,const GwSlippiWireSelections *selection) {
    const uint8_t *player=sm.fixture.game_info+0x60+0x24*sm.remote_port;
    uint32_t seed=sm.remote_port==0?sm.fixture.seed:0;
    (void)user;
    if (selection->character_id!=player[0] || selection->character_color!=player[3] ||
        !selection->character_selected || !selection->stage_selected ||
        selection->stage_id!=sm.local_selection.stage_id || selection->rng_offset!=seed ||
        selection->alt_stage_mode!=0) { sm.pending_error=1; return; }
    sm.remote_selection=*selection; sm.got_selections=1;
}
static void sm_remote_selected(void *user) { (void)user; sm.selected=1; }

void gw_SlippiMode_Tick(int online_frame) {
    GwSlippiPeerStats stats={0};
    GwSlippiPad pad;
    int next, checksum_frame, current, i;
    uint32_t checksum;
    DWORD now=GetTickCount();
    if (!sm.active || !sm.peer) return;
    gw_slippi_peer_poll(sm.peer,online_frame);
    if (sm.pending_error || gw_rb_desyncs()) sm_exit(2,"peer checksum or selection mismatch");
    if (gw_Replay_SlippiLocalMismatches()) sm_exit(2,"raw input processing differs from fixture");
    for (i=0;i<64;i++) if (sm.checksums[i].value) {
        checksum=gw_rb_checksum(gw_SlippiPad_SlpFrame(sm.checksums[i].frame));
        if (checksum) {
            if (checksum!=sm.checksums[i].value) sm_exit(2,"finalized gameplay checksum mismatch");
            sm.checksums[i].value=0;
        }
    }
    current=gw_Replay_Frame();
    if (current>=sm.fixture.first_frame) sm.started=1;
    if (current>sm.last_simulated) { sm.last_simulated=current; sm.progress_ms=now; }
    if ((DWORD)(now-sm.progress_ms)>60000u) sm_exit(2,"no gameplay progress for 60 seconds");
    gw_slippi_peer_stats(sm.peer,&stats);
    checksum_frame=gw_rb_confirmed_frame();
    if (checksum_frame>current) checksum_frame=current;
    checksum=gw_rb_checksum(checksum_frame);
    checksum_frame=checksum?gw_SlippiPad_OnlineFrame(checksum_frame):-1;
    for (next=stats.last_sent_frame+1;
         next<=online_frame+sm.config.delay && gw_rb_slippi_local_pad(next,&pad);next++) {
        if (!gw_slippi_peer_send_pad(sm.peer,next,pad.bytes,checksum_frame,checksum)) break;
    }
    gw_slippi_peer_stats(sm.peer,&stats);
    if (gw_rb_slippi_finalized() &&
        stats.last_acked_frame>=gw_SlippiPad_OnlineFrame(sm.fixture.last_frame)) {
        if (!sm.complete_ms) sm.complete_ms=now;
        /* Keep answering the other client's final retransmissions until its ACK
         * has had time to arrive. Final state/checksum validation is also offline. */
        if ((DWORD)(now-sm.complete_ms)>=1000u) {
            sm.complete=1; sm_exit(0,"complete; finalized artifacts ready for comparison");
        }
    }
}

const char *gw_SlippiMode_Scene(void) {
    const char *why=NULL,*scene;
    GwSlippiPeerConfig peer={0};
    GwSlippiMatchAssignment assignment={0};
    GwSlippiPeerStats stats={0};
    char host[64]="127.0.0.1";
    const uint8_t *player;
    DWORD begin;
    int status;
    if (sm.tried) return sm.active?gw_replay_scene():NULL;
    sm.tried=1;
    status=sm_config(&sm.config,&why);
    if (!status) return NULL;
    if (status<0) { gw_log("slippi: invalid configuration: %s",why); fflush(NULL); _exit(2); }
    if (!gw_Replay_SlippiFixtureInfo(&sm.fixture)) {
        gw_log("slippi: rejected fixture: %s",gw_Replay_SlippiFixtureReason()); fflush(NULL); _exit(2);
    }
    sm.local_port=sm.config.role-1;
    sm.last_simulated=sm.fixture.first_frame-1;
    if (sm.config.direct) {
        if (gw_slippi_match_start(sm.config.profile,sm.config.code)<0)
            sm_exit(2,gw_slippi_match_error());
        begin=GetTickCount();
        do {
            status=gw_slippi_match_poll(&assignment);
            if (status<0) sm_exit(2,gw_slippi_match_error());
            if (status>0) break;
            if ((DWORD)(GetTickCount()-begin)>90000u) sm_exit(2,"Direct matchmaking timed out");
            Sleep(2);
        } while (1);
        sm.local_port=assignment.local_port;
        sm.config.local_udp_port=assignment.local_udp_port;
        {
            const char *address=assignment.peer_public;
            const char *colon;
            char *end;
            long port;
            if (assignment.same_external_ip && assignment.peer_lan[0]) address=assignment.peer_lan;
            colon=strrchr(address,':');
            if (!colon || colon==address || (size_t)(colon-address)>=sizeof host)
                sm_exit(2,"invalid assigned peer address");
            memcpy(host,address,(size_t)(colon-address)); host[colon-address]=0;
            errno=0; port=strtol(colon+1,&end,10);
            if (errno || end==colon+1 || *end || port<1 || port>65535)
                sm_exit(2,"invalid assigned peer port");
            sm.config.remote_udp_port=(int)port;
        }
        if (!sm_account_tag(assignment.local_uid_hash,sm_env("MELEE_SLIPPI_RUN_SALT"),sm.account_tag) ||
            !sm_account_tag(assignment.remote_uid_hash,sm_env("MELEE_SLIPPI_RUN_SALT"),sm.peer_account_tag) ||
            !strcmp(sm.account_tag,sm.peer_account_tag)) sm_exit(2,"invalid two-account identity evidence");
        memcpy(sm.run_salt,sm_env("MELEE_SLIPPI_RUN_SALT"),33);
        snprintf(sm.match_id,sizeof sm.match_id,"%s",assignment.match_id);
        gw_slippi_match_close(); /* release the bound port before peer host creation */
    } else {
        int lo=sm.config.local_udp_port<sm.config.remote_udp_port?sm.config.local_udp_port:sm.config.remote_udp_port;
        int hi=sm.config.local_udp_port>sm.config.remote_udp_port?sm.config.local_udp_port:sm.config.remote_udp_port;
        const char *run_id=sm_env("MELEE_SLIPPI_MATCH_ID");
        if (run_id) snprintf(sm.match_id,sizeof sm.match_id,"%s",run_id);
        else snprintf(sm.match_id,sizeof sm.match_id,"loopback-%d-%d-%08x",lo,hi,sm.fixture.seed);
    }
    if (sm.local_port<0 || sm.local_port>1) sm_exit(2,"invalid assigned player port");
    sm.remote_port=1-sm.local_port;
    if (!gw_rb_slippi_configure(sm.local_port,sm.config.delay,gw_SlippiMode_Tick))
        sm_exit(2,"fixture cannot drive the raw rollback input source");
    player=sm.fixture.game_info+0x60+0x24*sm.local_port;
    sm.local_selection.character_id=player[0]; sm.local_selection.character_color=player[3];
    sm.local_selection.character_selected=1; sm.local_selection.player_idx=(uint8_t)sm.local_port;
    sm.local_selection.stage_id=gw_r16(sm.fixture.game_info+0x0e);
    sm.local_selection.stage_selected=1;
    sm.local_selection.rng_offset=sm.local_port==0?sm.fixture.seed:0;
    peer.remote_host=host; peer.local_udp_port=(uint16_t)sm.config.local_udp_port;
    peer.remote_udp_port=(uint16_t)sm.config.remote_udp_port;
    peer.local_port_idx=(uint8_t)sm.local_port; peer.remote_port_idx=(uint8_t)sm.remote_port;
    peer.match_id=sm.match_id; peer.loopback_mode=!sm.config.direct;
    peer.on_remote_pad=sm_remote_pad; peer.on_remote_checksum=sm_remote_checksum;
    peer.on_selections=sm_remote_selection; peer.on_connection_selected=sm_remote_selected;
    sm.peer=gw_slippi_peer_start(&peer);
    if (!sm.peer) sm_exit(2,"could not open Slippi peer socket");
    begin=GetTickCount();
    do {
        gw_slippi_peer_poll(sm.peer,1);
        gw_slippi_peer_stats(sm.peer,&stats);
        if (sm.pending_error) sm_exit(2,"peer selected different fixture settings");
        if (stats.connected) {
            if (!sm.sent_selections) sm.sent_selections=gw_slippi_peer_send_selections(sm.peer,&sm.local_selection);
            if (!sm.sent_selected) sm.sent_selected=gw_slippi_peer_send_connection_selected(sm.peer);
        }
        if (sm.got_selections && sm.selected && sm.sent_selected && sm.sent_selections) break;
        if ((DWORD)(GetTickCount()-begin)>60000u) sm_exit(2,"Slippi peer handshake timed out");
        Sleep(2);
    } while (1);
    scene=gw_replay_scene();
    if (!scene || !*scene) sm_exit(2,"fixture does not describe a launchable match");
    sm.active=1; sm.progress_ms=GetTickCount();
    gw_log("slippi: %s peer selected, P%d local / P%d remote, fixture frames %d..%d, delay %d",
           sm.config.direct?"Direct":"loopback",sm.local_port+1,sm.remote_port+1,
           sm.fixture.first_frame,sm.fixture.last_frame,sm.config.delay);
    return scene;
}
