# Linux LFS pet access shell

This staged increment adds an actual GTK window to the existing PC-Pet owner.
It is a local pet and browser access surface, with an original sage woodland
companion drawn in code. The full Family Hub remains available directly at
<https://lewisfamilysystems.com>. Native API authentication, persisted pet state,
cross-device sync and complete planned pet behaviors remain future work.

## Launch and dependencies

From the repository root, in your Linux graphical session:

```sh
python3 clients/linux/lfs_pet.py
```

Requires Python 3.9 or newer, GTK 3.24, PyGObject, Pycairo and the GI Cairo
integration for drawing. On Debian/Ubuntu the native dependencies are normally
provided by `python3-gi`, `python3-cairo`, `python3-gi-cairo` and
`gir1.2-gtk-3.0`. This increment does not install packages. A virtual environment
that hides system GI packages will need an appropriate system-package setup.
Run from the desktop session, rather than a headless SSH session. Missing
dependencies or display access produces a diagnostic and the Family Hub URL.

The actual startup check was performed on GTK 3.24 with the available X11
session. Wayland, other distributions and actual owner installation are unqualified.
No systemd unit, automatic login startup, background service or core process is
started. Closing the widget stops this process; rerun the command to restore it.
No local data or settings are written by the shell.

## Offline package and per-user menu launcher

The bounded package builder/installer is `clients/linux/package.py`.
See `clients/linux/PACKAGE.md` in the checkout (`README.md` in the package) for exact offline
verify/install/update/activate/remove/recover commands, dependency/provenance
limits and preserved evidence. It packages only this Python GTK shell, fixed
navigation, the stdlib installer and two guides, with exact source/file digests.
It does not bundle or launch the Rust core, donor assets, telemetry or credentials.
Installation adds a normal per-user application-menu entry; no autostart or
service is added. Closing the original widget remains the normal way to stop it.

Updates retain content-addressed prior versions. Removing the exact managed menu
entry retains packages and recovery history. Interrupted launcher changes have a
durable before-state and an explicit recover command. Unfamiliar files, symlinks,
ownership or edited retained versions cause a visible hold, not an overwrite.
Desktop Exec uses directly quoted absolute Python/script arguments; paths with
spaces are included in the disposable check. The bounded path/desktop matrix is
documented in the package guide. Digests are identity evidence, not signing,
license clearance, universal portability or production acceptance.

## Interaction and browser authority

- Drag the normal title bar to move the pet. The window has normal desktop
  decorations and taskbar presence.
- Activate the pet picture or **Open Family Hub** to open the exact HTTPS origin.
  The pet picture has a descriptive accessible name; the labeled button is the
  full fallback when the drawing is unhelpful.
- Activate the labeled **LFS tools** shortcut for the fixed
  <https://lewisfamilysystems.com/tools> destination. This opens the separate LFS
  platform manager in the browser; the pet does not become that manager. The new
  `/tools` page is separately owned source work under development, not confirmed
  live availability. Browser handoff alone proves no page, sign-in or tool access.
- Expand **Family tools** for **Calendar** (`/calendar`), **Notes & Projects**
  (`/workspace`) and **Connections** (`/settings/connections`). Destinations are
  fixed same-origin routes, not arbitrary input URLs.
- Tab/Shift+Tab traverse native controls; Enter/Space activates the focused
  action. GTK mnemonic letters provide local-window access subject to desktop
  settings. The window defines no global shortcuts or input hooks.
- **Quiet / focus mode** pauses local motion. It is not a system-wide DND switch.
  The pet sends no notifications. Motion also pauses while minimized.
- **Stay above other windows** is an optional window-manager request, off by
  default. **Minimize** uses the taskbar for restoration; **Close** exits.

Browser handoff uses `Gio.AppInfo.launch_default_for_uri` with the exact public
URL. The browser owns sign-in, login redirects, permissions, private tools and
navigation. No cookie, token, user identity, audio URL or pet state is extracted
or passed. The shell does not call APIs or inspect network state. A successful
handoff says **Sent … to your browser**, not that the page loaded or access was
granted. If the desktop launcher fails, the status says so and selects the
read-only address for copying into a browser. A unavailable website or signed-out
session does not stop local rendering; the browser reports its own page/network
outcome. DND does not silence or change the browser.

## Essential checks and evidence limits

```sh
python3 clients/linux/check_contract.py
python3 -m py_compile clients/linux/lfs_pet.py clients/linux/navigation.py clients/linux/check_contract.py
python3 clients/linux/lfs_pet.py --smoke-test
```

The contract check uses a fake launcher: five exact routes, handoff versus
failure messaging, retained fallback address and rejection of an unknown route.
It opens no browser and proves no native interaction or authentication journey.
The smoke switch displays only this widget, reports whether GTK mapped it and
the drawing callback ran, then closes after three seconds. It opens no browser.
It proves bounded native startup/rendering, not visual quality, useful clicks,
accessibility, input safety across desktop applications or active-use reliability.
Existing Rust format/build/test and hosted source CI remain integration gates.

Recorded local native smoke: sandbox `Gtk.init_check()` initially returned false;
the authorized display-only invocation passed with `mapped=True`, `draws=37`,
exit 0. Preserve that environmental failure; a headless result alone would not
qualify visibility. The final increment's check receipts live in the integrator's
handoff; this initial draw count is not exact-final-source acceptance.

Required owner journey, still **NOT RUN**: launch this widget normally, click
Open Family Hub, activate it with the keyboard, confirm the exact browser origin,
check signed-out login and authorized/denied tools in the browser, minimize and
restore, pause motion, close and relaunch. An offline pet check must show the
local controls remain usable while the browser reports unavailable Main.

## Integration boundaries and deferred final audit

The existing core's `RenderFrameState` remains a possible bridge. This shell
deliberately has no core IPC integration: current fixed `/tmp` transport and
default telemetry startup require their own privacy/ownership decisions before
connection. The visual's idle blink and quiet state are presentation, not
persisted mood, XP, vitals, awards or synchronized state. There is no invented
database, listener or second service.

Final qualification remains open for actual mouse/keyboard interaction;
screen reader, high contrast, reduced motion and display scaling; desktop
input/focus containment and system DND behavior; multi-monitor restoration;
Wayland/window-manager behavior; actual owner package/install/update/remove and
rollback acceptance beyond the disposable function journey; code/artwork release
licensing/signing; Android physical devices; native
keyring, session revocation and cross-device consent/replay/conflict contracts;
recovery/export/persistence; full integrated feature coverage and sustained use.
The drawing imports no donor sprite asset, but project release licensing still
needs an explicit decision. MacroPet, Action Loop and Phoenix remain separate
held products; using this pet does not resume them.

Stop this increment if it cannot display on the chosen desktop, its route
disagrees with Main, launch failures are hidden, access requires copying
credentials, outside-window input is intercepted, or integrating a donor would
resume an independently held lane.


## Fixed platform-manager shortcut source increment

The2026-10-06 bounded shortcut change is based on exact PC-Pet Main
`4a68bffbe67568bad8c19f40fd2bf490458abf3d` in the isolated
`lfs-pet-manager-20261006` checkout. It adds only fixed navigation key `tools`
and a directly labeled native **LFS tools** button, reusing the existing browser
handoff and failure address fallback. It adds no backend command, API, session,
credential/cookie extraction, dependency, service or authentication shortcut.
The browser retains native sign-in and receiving permissions; `/tools` is a
separate Main source increment under development and is not claimed deployed.

Author verification is limited to the existing fake-launcher route contract
extended for this one destination, changed Python compilation and whitespace.
Exact commands/exits/file digests are retained in the private handoff under
`/tmp/lfs-pet-manager-*`. The earlier GUI/startup/package evidence above remains
historical and is not repeated or promoted to exact-new-source acceptance.
Native GUI interaction, package/install/update/recovery, signing, actual device
and browser page/authentication availability, the full80 pet requirements and
sustained integrated use remain **NOT RUN for this increment**. Quiet/focus,
optional stay-above, minimize/close and original frame behavior are retained.
MacroPet, Action Loop and Phoenix remain independent held products.
