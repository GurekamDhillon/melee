"""check_results.py A.json B.json KEY_A KEY_B: the two P2P result files of one game are a two-sided record: each
signed by its player's identity (verified with the system libsodium), the same descriptor and transcript digests,
the peer named, the peer's RESULT compared and agreeing, an outcome, no desync, no disconnect, opposite slots."""
import ctypes, json, sys

DOMAIN = b"MeleeUnlockedNet1 result"


def load(path):
    text = open(path, "rb").read().decode()
    j = json.loads(text)
    body = text[len('{"body":'):text.rindex(',"identity":"')]
    return j, body


def verify(sodium, j, body):
    sig = bytes.fromhex(j["signature"])
    pk = bytes.fromhex(j["identity"])
    msg = DOMAIN + body.encode()
    return sodium.crypto_sign_verify_detached(sig, msg, ctypes.c_ulonglong(len(msg)), pk) == 0


def main():
    a_path, b_path, ka, kb = sys.argv[1:5]
    sodium = ctypes.CDLL("libsodium.so.26")
    sodium.sodium_init()
    (a, ab), (b, bb) = load(a_path), load(b_path)
    A, B = a["body"], b["body"]
    checks = [
        ("A signed by A", verify(sodium, a, ab) and a["identity"] == ka),
        ("B signed by B", verify(sodium, b, bb) and b["identity"] == kb),
        ("same descriptor", A["descriptor_digest"] == B["descriptor_digest"]),
        ("same transcript", A["transcript_digest"] == B["transcript_digest"] and A["transcript_frames"] == B["transcript_frames"]),
        ("peers named", A["peer_identity"] == kb and B["peer_identity"] == ka),
        ("peer agreement", A["peer_agreement"] == "match" and B["peer_agreement"] == "match"),
        ("outcome", A["has_outcome"] and B["has_outcome"] and not A["desync"] and not B["desync"]),
        ("connected", not A["disconnected"] and not B["disconnected"]),
        ("slots", (A["local_slot"], B["local_slot"]) == (0, 1)),
        ("same winner", A["winner"] == B["winner"]),
    ]
    for name, ok in checks:
        print(f"check_results: {name}: {'ok' if ok else 'FAIL'}")
    print(f"check_results: {A['transcript_frames']} frames, winner {A['winner']}, route {A['route']}")
    sys.exit(0 if all(ok for _, ok in checks) else 1)


main()
