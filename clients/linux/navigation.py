"""Fixed public navigation only; this module does not access sessions or networks."""

from dataclasses import dataclass
from types import MappingProxyType


ORIGIN = "https://lewisfamilysystems.com"
DESTINATIONS = MappingProxyType({
    "hub": ("Family Hub", ORIGIN),
    "tools": ("LFS tools", ORIGIN + "/tools"),
    "calendar": ("Calendar", ORIGIN + "/calendar"),
    "workspace": ("Notes & Projects", ORIGIN + "/workspace"),
    "connections": ("Connections", ORIGIN + "/settings/connections"),
})


@dataclass(frozen=True)
class LaunchResult:
    handed_off: bool
    message: str
    url: str


def open_destination(key, launcher):
    """Allow only fixed routes; successful handoff does not prove page availability.

    The GUI adapter maps Gio launch errors to False. Unexpected programming errors
    are not swallowed here.
    """
    if key not in DESTINATIONS:
        raise ValueError("Unknown Family Hub destination")
    title, url = DESTINATIONS[key]
    if bool(launcher(url)):
        return LaunchResult(True, f"Sent {title} to your browser. Sign in there if asked.", url)
    return LaunchResult(
        False,
        "Browser launch failed. Open the address below in your browser, then retry.",
        url,
    )
