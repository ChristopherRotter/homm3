"""Does a trained network's trunk know the round's turn order? (E06, the user's question 2026-10-26)

Self-play (both seats external, the network on both sides) on the player kinds; at each decision
after the tactics phase whose chosen entry is not Wait, a sample: the trunk's output (the trained
network's and an untrained one's of the same shape, the probe's control), the candidates, the
order rule's prediction from the observation. Labels from the later decisions of the same battle:
the stack that acts next (a morale repeat left out) and the opponent's stack that acts next.
Linear and small-MLP probes (masked to the candidates) on the trunk's output and, as the probes'
own control, on the observation's order columns alone; trained on 3/4 of the envs, tested on the
rest. A one-off script (`python/records/README.md`); from `python/`:

    uv run --group train python records/inspect/probe_order.py runs/t6d-1/best.pt 600 \
        > records/inspect/t6d-1-order-probe.json
"""

import json
import sys

import numpy as np
import torch

from h3env.batch import BattleBatch
from h3env.train import layout as L
from h3env.train.evaluate import _fill, load
from h3env.train.policy import build

SPEED, GOOD_MORALE = 11, 23
WAIT_PHASE, VIEWER_ATTACKS = 14, 1
N = L.STACK_CAPACITY


def order(st, a, active_attacker, attacker, wait_phase):
    """The order rule (BAT-TRN-001/002/003, approximate: no freezes, blindness, Catapult's turn,
    hypnosis): the slots acting after the active one `a`, over the rest of the round and the next."""
    alive = (st[:, L.PRESENT] > 0) & (st[:, L.ALIVE] > 0) & (st[:, SPEED] > 0)
    speed = st[:, SPEED]
    acted = st[:, L.ACTED] > 0
    waited = st[:, L.WAITED] > 0
    acted[a] = True
    last = active_attacker
    seq = []

    def pick(mask, fastest):
        idx = np.nonzero(mask)[0]
        if len(idx) == 0:
            return None
        sp = speed[idx]
        tied = idx[sp == (sp.max() if fastest else sp.min())]
        pref = tied[attacker[tied] != last]
        return int((pref if len(pref) else tied).min())

    for _ in range(3 * N):
        if not wait_phase:
            r = pick(alive & ~acted & ~waited, True)
            if r is None:
                r = pick(alive & waited & ~acted, False)
                wait_phase = r is not None
        else:
            r = pick(alive & ~acted, False)
        if r is None:
            if not alive.any():
                break
            acted[:] = False
            waited[:] = False
            wait_phase = False
            continue
        seq.append(r)
        acted[r] = True
        last = attacker[r]
    return seq


def main(path, calls):
    torch.manual_seed(0)
    saved = torch.load(path, map_location="cpu", weights_only=True)
    policy = load(path)
    policy.eval()
    envs = 256
    batch = BattleBatch(envs, threads=4, seed=0x0D0E_0001, max_steps=2000, preview=True, final_obs=False)
    batch.add_encounters(
        kinds={"hero": 100, "siege_hero": 50, "siege_garrison": 25},
        attacker="external",
        defender="external",
    )
    batch.reset()
    control = build(saved["config"], batch)
    control.eval()
    gen = torch.Generator().manual_seed(1)

    H, HR, RAW, CAND, OPP, ENV = [], [], [], [], [], []
    cols = [L.PRESENT, L.ALIVE, L.OWN, SPEED, L.ACTED, L.WAITED, L.ACTIVE, GOOD_MORALE]
    rule_next, rule_opp, naive_next = [], [], []
    next_active, next_opp = [], []
    owner_att = []  # per sample: its viewer is the attacker
    open_ = [[] for _ in range(envs)]  # per env: samples awaiting a label
    s0 = L.STACKS
    for _ in range(calls):
        obs = batch.obs
        x = torch.from_numpy(obs)
        with torch.no_grad():
            h = policy.encode(x)[0].numpy().astype(np.float16)
            hr = control.encode(x)[0].numpy().astype(np.float16)
        choice = policy.act(batch, generator=gen)
        _fill(batch, choice)
        st_all = obs[:, s0 : s0 + N * L.STACK_FEATURES].reshape(envs, N, L.STACK_FEATURES)
        for e in np.nonzero(batch.to_move >= 0)[0]:
            st = st_all[e]
            act = np.nonzero(st[:, L.ACTIVE] > 0)[0]
            if len(act) != 1:
                continue
            a = int(act[0])
            va = obs[e, VIEWER_ATTACKS] > 0
            attacker = (st[:, L.OWN] > 0) == va
            # The labels of this env's open samples.
            keep = []
            for p in open_[e]:
                if next_active[p] == -1:
                    next_active[p] = -2 if a == ACTIVE_OF[p] else a
                if next_opp[p] == -1 and attacker[a] != owner_att[p]:
                    next_opp[p] = a
                if next_opp[p] == -1:
                    keep.append(p)
            open_[e] = keep
            if obs[e, L.ROUND] == 0 or int(choice.entry[e]) == 0:
                continue
            alive = (st[:, L.PRESENT] > 0) & (st[:, L.ALIVE] > 0)
            cand = alive.copy()
            cand[a] = False
            if not cand.any():
                continue
            seq = order(st.copy(), a, attacker[a], attacker, obs[e, WAIT_PHASE] > 0)
            opp_now = cand & (attacker != va)
            rn = seq[0] if seq else -1
            ro = next((s for s in seq if attacker[s] != va), -1)
            fresh = cand & (st[:, L.ACTED] == 0) & (st[:, L.WAITED] == 0) & (st[:, SPEED] > 0)
            pool = np.nonzero(fresh if fresh.any() else cand)[0]
            nv = int(pool[np.argmax(st[pool, SPEED])])
            p = len(H)
            H.append(h[e])
            HR.append(hr[e])
            RAW.append(np.concatenate([st[:, cols].ravel(), obs[e, [WAIT_PHASE, VIEWER_ATTACKS]]]).astype(np.float16))
            CAND.append(cand)
            OPP.append(opp_now)
            ENV.append(e)
            rule_next.append(rn)
            rule_opp.append(ro)
            naive_next.append(nv)
            next_active.append(-1)
            next_opp.append(-1)
            owner_att.append(va)
            ACTIVE_OF.append(a)
            open_[e].append(p)
        batch.step()
        for e in np.nonzero(batch.done.astype(bool))[0]:
            open_[e] = []
    batch.close()
    return {
        "H": np.stack(H), "HR": np.stack(HR), "RAW": np.stack(RAW), "CAND": np.stack(CAND), "OPP": np.stack(OPP),
        "ENV": np.array(ENV), "rule_next": np.array(rule_next), "rule_opp": np.array(rule_opp),
        "naive_next": np.array(naive_next), "next_active": np.array(next_active),
        "next_opp": np.array(next_opp),
    }


ACTIVE_OF: list[int] = []


def fit(X, y, mask, train, test, hidden):
    torch.manual_seed(0)
    X = torch.tensor(X, dtype=torch.float32)
    mu, sd = X[train].mean(0), X[train].std(0) + 1e-3
    X = (X - mu) / sd
    y = torch.tensor(y)
    m = torch.tensor(mask)
    d = X.shape[1]
    net = (
        torch.nn.Linear(d, N)
        if hidden == 0
        else torch.nn.Sequential(torch.nn.Linear(d, hidden), torch.nn.ReLU(), torch.nn.Linear(hidden, N))
    )
    opt = torch.optim.Adam(net.parameters(), lr=1e-3, weight_decay=1e-4)
    tr = torch.tensor(train)
    for _ in range(30):
        perm = tr[torch.randperm(len(tr))]
        for i in range(0, len(perm), 1024):
            b = perm[i : i + 1024]
            logits = net(X[b]).masked_fill(~m[b], -1e9)
            loss = torch.nn.functional.cross_entropy(logits, y[b])
            opt.zero_grad()
            loss.backward()
            opt.step()
    with torch.no_grad():
        te = torch.tensor(test)
        pred = net(X[te]).masked_fill(~m[te], -1e9).argmax(1)
    return float((pred == y[te]).float().mean())


if __name__ == "__main__":
    d = main(sys.argv[1], int(sys.argv[2]) if len(sys.argv) > 2 else 600)
    out = {"samples": int(len(d["H"]))}
    for label, mask_key, rule_key in (("next_active", "CAND", "rule_next"), ("next_opp", "OPP", "rule_opp")):
        y = d[label]
        mask = d[mask_key]
        ok = (y >= 0) & mask[np.arange(len(y)), y.clip(0)]
        train = np.nonzero(ok & (d["ENV"] < 192))[0]
        test = np.nonzero(ok & (d["ENV"] >= 192))[0]
        row = {
            "labelled": int(ok.sum()),
            "test": int(len(test)),
            "uniform": float((1.0 / np.maximum(mask[test].sum(1), 1)).mean()),
            "rule": float((d[rule_key][test] == y[test]).mean()),
        }
        if label == "next_active":
            row["naive_fastest"] = float((d["naive_next"][test] == y[test]).mean())
        for name, X in (("trained", d["H"]), ("untrained", d["HR"]), ("order_columns", d["RAW"])):
            row[f"linear_{name}"] = fit(X, y.clip(0), mask, train, test, 0)
            row[f"mlp_{name}"] = fit(X, y.clip(0), mask, train, test, 256)
        out[label] = row
    print(json.dumps(out, indent=1))
