"""The GPU's memory (E25 T28; the user, 2026-11-10: a run never fills more than 90 % of the card).

PyTorch's caching allocator keeps the blocks it freed for later allocations and gives memory
back only when an allocation fails. A rollout's and an update's tensors change their sizes from
call to call (the frozen networks' rows, a minibatch's chunks), so the cache grows past what the
run holds at any one time. On Linux the card's end is an out-of-memory error, which makes the
allocator free its cached blocks and retry; on Windows the driver instead places the next
allocation in shared system memory (NVIDIA's CUDA sysmem fallback, the driver's default), no
allocation fails, and the update's passes run over the PCIe bus: `e25-structure-1` (T28) slowed
from 100 to 650 to 1 950 seconds an update on the GTX 1070, its rollouts unchanged, until a
restart emptied the cache.

`cap` sets the allocator's own limit (`torch.cuda.set_per_process_memory_fraction`) so that the
card stays at `MEMORY_FRACTION`, what it held at the cap's call outside the allocator (other
programs, this process's CUDA context) counted against it: past the limit the allocator frees
its cached blocks and retries, and raises the out-of-memory error only when the run's live
tensors alone need more. `usage` reports the allocator's memory and the card's share for each
update's record. The module imports torch only in its calls (the command line's `line` reads
`MEMORY_FRACTION` without it).
"""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import torch

#: The share of the card's memory a run may fill, other programs' use included (the user,
#: 2026-11-10).
MEMORY_FRACTION = 0.9
MB = 2**20


class GpuFull(RuntimeError):
    """The card holds more than `MEMORY_FRACTION` of its memory outside this run already."""


def cap_bytes(total: int, free: int, reserved: int, fraction: float = MEMORY_FRACTION) -> int:
    """The bytes this process's caching allocator may reserve for the card to stay at `fraction`
    of its `total`, given its `free` bytes and the allocator's `reserved` ones: the rest of its use
    (`total - free - reserved`: other programs, this process's CUDA context) counts against it."""
    outside = total - free - reserved
    return max(0, int(fraction * total) - outside)


def cap(device: torch.device, fraction: float = MEMORY_FRACTION) -> dict[str, float] | None:
    """Caps `device`'s caching allocator at `cap_bytes` (CUDA only: None for another device) and
    returns the card's total, its use outside the allocator and the cap, in MB. Raises `GpuFull`
    when nothing is left under the cap (close what fills the card, then start again)."""
    if device.type != "cuda":
        return None
    import torch

    free, total = torch.cuda.mem_get_info(device)
    reserved = torch.cuda.memory_reserved(device)
    limit = cap_bytes(total, free, reserved, fraction)
    outside = (total - free - reserved) / MB
    if limit <= 0:
        raise GpuFull(
            f"the GPU holds {outside:.0f} of its {total / MB:.0f} MB outside this run, past "
            f"{fraction:.0%}: close what fills it and start again"
        )
    # torch's call takes the card's index: a plain "cuda" (`ppo.device_of`) is the current card.
    index = torch.cuda.current_device() if device.index is None else device.index
    torch.cuda.set_per_process_memory_fraction(limit / total, index)
    return {"total_mb": total / MB, "outside_mb": outside, "cap_mb": limit / MB}


def usage(device: torch.device) -> dict[str, float]:
    """For an update's record (CUDA only: {} for another device): the allocator's reserved MB now
    and at its peak since the last call (which starts the next peak), and the card's used share,
    other programs' use included."""
    if device.type != "cuda":
        return {}
    import torch

    free, total = torch.cuda.mem_get_info(device)
    out = {
        "gpu_reserved_mb": torch.cuda.memory_reserved(device) / MB,
        "gpu_peak_mb": torch.cuda.max_memory_reserved(device) / MB,
        "gpu_card": (total - free) / total,
    }
    torch.cuda.reset_peak_memory_stats(device)
    return out
