#!/usr/bin/env python3
"""Local GTK pet/access shell. No telemetry, core startup or native authentication."""

import argparse
import math
import sys
from navigation import DESTINATIONS, ORIGIN, open_destination

try:
    import gi

    gi.require_version("Gtk", "3.0")
    gi.require_version("Gdk", "3.0")
    from gi.repository import Gdk, Gio, GLib, Gtk
    import cairo
except (ImportError, ValueError) as error:
    print(
        "LFS Pet needs Python 3, PyGObject, GTK 3 and Pycairo. "
        f"See docs/LINUX-PET.md. Dependency error: {error}. "
        "Family Hub: https://lewisfamilysystems.com",
        file=sys.stderr,
    )
    sys.exit(2)


def browser_launcher(url):
    """Let the desktop browser handle all authentication and navigation."""
    try:
        return Gio.AppInfo.launch_default_for_uri(url, None)
    except (GLib.Error, OSError):
        return False


class PetShell(Gtk.Window):
    def __init__(self, smoke_test=False):
        super().__init__(title="LFS Pet")
        self.set_default_size(320, 420)
        self.set_resizable(True)
        self.set_position(Gtk.WindowPosition.CENTER)
        self.connect("destroy", self.on_destroy)
        self.quiet = False
        self.phase = 0
        self.draw_count = 0
        self.smoke_test = smoke_test
        self.timer = None
        self.minimized = False
        self.connect("window-state-event", self.on_window_state)

        css = Gtk.CssProvider()
        css.load_from_data(b"""
            .pet-shell { background: #edf2e8; color: #25382b; }
            .pet-title { font-size: 20px; font-weight: bold; }
            .pet-note { color: #435647; }
            button { padding: 7px; }
            button.pet-picture { background: #dde9d7; border-radius: 18px; }
            button:focus { outline: 2px solid #375d41; outline-offset: 2px; }
        """)
        self.get_style_context().add_class("pet-shell")
        Gtk.StyleContext.add_provider_for_screen(
            self.get_screen(), css, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )

        outer = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=9)
        outer.set_border_width(14)
        self.add(outer)
        title = Gtk.Label(label="Your family, close by")
        title.get_style_context().add_class("pet-title")
        outer.pack_start(title, False, False, 0)

        picture = Gtk.Button()
        picture.get_style_context().add_class("pet-picture")
        picture.set_tooltip_text("Open Family Hub in your browser")
        picture.get_accessible().set_name("Pet: Open Family Hub in your browser")
        picture.connect("clicked", self.open_hub)
        self.canvas = Gtk.DrawingArea()
        self.canvas.set_size_request(240, 165)
        self.canvas.connect("draw", self.draw_pet)
        picture.add(self.canvas)
        outer.pack_start(picture, True, True, 0)

        self.hub_button = Gtk.Button.new_with_mnemonic("Open _Family Hub")
        self.hub_button.connect("clicked", self.open_hub)
        outer.pack_start(self.hub_button, False, False, 0)

        self.tools_button = Gtk.Button(label=DESTINATIONS["tools"][0])
        self.tools_button.set_tooltip_text("Open LFS tools in your browser")
        self.tools_button.connect("clicked", self.open_shortcut, "tools")
        outer.pack_start(self.tools_button, False, False, 0)

        # Native labeled controls remain usable without understanding the drawing.
        self.shortcuts = Gtk.Expander.new_with_mnemonic("Family _tools")
        links = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=5)
        for key in ("calendar", "workspace", "connections"):
            button = Gtk.Button(label=DESTINATIONS[key][0])
            button.connect("clicked", self.open_shortcut, key)
            links.pack_start(button, False, False, 0)
        self.shortcuts.add(links)
        outer.pack_start(self.shortcuts, False, False, 0)

        self.status = Gtk.Label(label="Local pet ready. Family tools open in your browser.")
        self.status.set_line_wrap(True)
        self.status.set_max_width_chars(38)
        self.status.get_style_context().add_class("pet-note")
        self.status.get_accessible().set_name("Pet and browser status")
        outer.pack_start(self.status, False, False, 0)

        self.address = Gtk.Entry()
        self.address.set_text(ORIGIN)
        self.address.set_editable(False)
        self.address.get_accessible().set_name("Family Hub address; select and copy")
        self.address.set_tooltip_text("Select and copy this address into your browser")
        outer.pack_start(self.address, False, False, 0)

        quiet = Gtk.CheckButton.new_with_mnemonic("_Quiet / focus mode (pause motion)")
        quiet.connect("toggled", self.set_quiet)
        outer.pack_start(quiet, False, False, 0)

        above = Gtk.CheckButton.new_with_mnemonic("Stay _above other windows")
        above.set_tooltip_text("Optional window-manager request; off by default")
        above.connect("toggled", lambda button: self.set_keep_above(button.get_active()))
        outer.pack_start(above, False, False, 0)

        controls = Gtk.Box(spacing=6)
        minimize = Gtk.Button.new_with_mnemonic("_Minimize")
        minimize.connect("clicked", lambda _button: self.iconify())
        close = Gtk.Button.new_with_mnemonic("_Close")
        close.connect("clicked", lambda _button: self.destroy())
        controls.pack_start(minimize, True, True, 0)
        controls.pack_start(close, True, True, 0)
        outer.pack_start(controls, False, False, 0)

        self.show_all()
        self.hub_button.grab_focus()
        self.update_animation()
        if smoke_test:
            GLib.timeout_add_seconds(3, self.finish_smoke)

    def open_hub(self, _button):
        self.launch("hub")

    def open_shortcut(self, _button, key):
        self.launch(key)

    def launch(self, key):
        result = open_destination(key, browser_launcher)
        self.status.set_text(result.message)
        self.address.set_text(result.url)
        if not result.handed_off:
            self.address.grab_focus()
            self.address.select_region(0, -1)

    def set_quiet(self, button):
        self.quiet = button.get_active()
        if self.quiet:
            self.status.set_text("Quiet / focus mode: motion paused. Family tools remain available.")
        else:
            self.status.set_text("Local pet ready. Family tools open in your browser.")
        self.update_animation()
        self.canvas.queue_draw()

    def on_window_state(self, _window, event):
        # Stop the animation timer while minimized instead of redrawing unseen work.
        self.minimized = bool(event.new_window_state & Gdk.WindowState.ICONIFIED)
        self.update_animation()
        return False

    def update_animation(self):
        if self.quiet or self.minimized:
            if self.timer is not None:
                GLib.source_remove(self.timer)
                self.timer = None
        elif self.timer is None:
            self.timer = GLib.timeout_add(100, self.tick)

    def tick(self):
        self.phase = (self.phase + 1) % 60
        self.canvas.queue_draw()
        return True

    @staticmethod
    def ellipse(ctx, x, y, rx, ry, color):
        ctx.save()
        ctx.translate(x, y)
        ctx.scale(rx, ry)
        ctx.arc(0, 0, 1, 0, 2 * math.pi)
        ctx.set_source_rgb(*color)
        ctx.fill()
        ctx.restore()

    def draw_pet(self, area, ctx):
        """Original vector artwork: a sage woodland companion with a family heart."""
        self.draw_count += 1
        width, height = area.get_allocated_width(), area.get_allocated_height()
        ctx.save()
        scale = min(width / 260, height / 165)
        ctx.translate((width - 260 * scale) / 2, (height - 165 * scale) / 2)
        ctx.scale(scale, scale)
        sage = (0.46, 0.62, 0.43)
        pale = (0.83, 0.89, 0.74)
        dark = (0.17, 0.28, 0.19)
        self.ellipse(ctx, 130, 147, 53, 8, (0.71, 0.79, 0.66))
        bob = 0 if self.quiet else 2 * math.sin(self.phase * math.pi / 30)
        ctx.translate(0, bob)
        self.ellipse(ctx, 91, 46, 15, 32, sage)
        self.ellipse(ctx, 169, 46, 15, 32, sage)
        self.ellipse(ctx, 91, 46, 7, 21, pale)
        self.ellipse(ctx, 169, 46, 7, 21, pale)
        self.ellipse(ctx, 130, 116, 39, 32, sage)
        self.ellipse(ctx, 107, 140, 16, 8, sage)
        self.ellipse(ctx, 153, 140, 16, 8, sage)
        self.ellipse(ctx, 130, 75, 50, 38, sage)
        self.ellipse(ctx, 130, 87, 32, 22, pale)
        blink = not self.quiet and self.phase in (0, 1)
        ctx.set_source_rgb(*dark)
        ctx.set_line_width(3)
        for x in (110, 150):
            if self.quiet or blink:
                ctx.move_to(x - 5, 72)
                ctx.line_to(x + 5, 72)
                ctx.stroke()
            else:
                self.ellipse(ctx, x, 70, 4, 6, dark)
        self.ellipse(ctx, 130, 84, 4, 3, dark)
        ctx.set_source_rgb(*dark)
        ctx.arc(130, 88, 8, 0.2, math.pi - 0.2)
        ctx.stroke()
        # Leaf crest, and an original simple heart, not an imported asset.
        self.ellipse(ctx, 130, 32, 9, 15, (0.33, 0.52, 0.31))
        ctx.set_source_rgb(0.76, 0.45, 0.43)
        ctx.move_to(130, 130)
        ctx.curve_to(105, 113, 125, 105, 130, 115)
        ctx.curve_to(135, 105, 155, 113, 130, 130)
        ctx.fill()
        ctx.restore()
        return False

    def finish_smoke(self):
        passed = self.get_mapped() and self.draw_count > 0
        print(f"GTK native startup: mapped={self.get_mapped()}, draws={self.draw_count}")
        self.smoke_passed = passed
        self.destroy()
        return False

    def on_destroy(self, _window):
        if self.timer is not None:
            GLib.source_remove(self.timer)
            self.timer = None
        Gtk.main_quit()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--smoke-test", action="store_true",
        help="display the native widget for three seconds, report mapped/drawn state and close; no browser",
    )
    args = parser.parse_args()
    initialized, _remaining = Gtk.init_check()
    if not initialized:
        print(
            "Cannot open a graphical display. Run in your Linux desktop session. "
            f"Family Hub is also available at {ORIGIN}", file=sys.stderr,
        )
        return 2
    shell = PetShell(smoke_test=args.smoke_test)
    Gtk.main()
    if args.smoke_test:
        return 0 if getattr(shell, "smoke_passed", False) else 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
