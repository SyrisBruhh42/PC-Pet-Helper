#!/usr/bin/env python3
"""One bounded public-route and launch-failure check. No GUI/browser/network."""

from navigation import DESTINATIONS, ORIGIN, open_destination


def main():
    expected = {
        "hub": "https://lewisfamilysystems.com",
        "tools": "https://lewisfamilysystems.com/tools",
        "calendar": "https://lewisfamilysystems.com/calendar",
        "workspace": "https://lewisfamilysystems.com/workspace",
        "connections": "https://lewisfamilysystems.com/settings/connections",
    }
    assert DESTINATIONS["tools"][0] == "LFS tools"
    assert ORIGIN == expected["hub"]
    assert set(DESTINATIONS) == set(expected)
    calls = []

    def success(url):
        calls.append(url)
        return True

    for key, url in expected.items():
        result = open_destination(key, success)
        assert result.handed_off and result.url == url
        assert "Sent" in result.message and "Sign in" in result.message
        failed = open_destination(key, lambda _url: False)
        assert not failed.handed_off and failed.url == url
        assert "failed" in failed.message and "retry" in failed.message
    assert calls == list(expected.values())
    try:
        open_destination("https://example.invalid", success)
    except ValueError:
        pass
    else:
        raise AssertionError("Unlisted destination accepted")
    assert calls == list(expected.values())
    print("Public route and browser handoff/failure contract: PASS (fake launcher only)")


if __name__ == "__main__":
    main()
