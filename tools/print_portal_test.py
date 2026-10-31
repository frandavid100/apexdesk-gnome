#!/usr/bin/python3
"""Hermetic Print portal: validates the real GTK handoff, never a real printer.

Run on a private dbus-run-session. Test-only service, not shipped in the bundle.
"""
import os
from pathlib import Path
import sys
from gi.repository import Gio, GLib

mode = sys.argv[1]
output = Path(sys.argv[2])
interface = Gio.DBusNodeInfo.new_for_xml('''<node><interface name="org.freedesktop.portal.Print">
<method name="Print"><arg type="s" direction="in"/><arg type="s" direction="in"/><arg type="h" direction="in"/><arg type="a{sv}" direction="in"/><arg type="o" direction="out"/></method>
<property name="version" type="u" access="read"/>
</interface></node>''').interfaces[0]
connection = Gio.bus_get_sync(Gio.BusType.SESSION, None)
calls = 0

def method(conn, sender, path, iface, name, params, invocation):
    global calls
    calls += 1
    assert calls == 1, "duplicate print submission"
    parent, title, descriptor, options = params.unpack()
    assert title.startswith("Imprimir — ")
    fd = invocation.get_message().get_unix_fd_list().get(descriptor)
    with os.fdopen(fd, "rb") as source:
        pdf = source.read()
    assert pdf.startswith(b"%PDF-") and len(pdf) > 100
    output.write_bytes(pdf)
    token = options["handle_token"]
    request = "/org/freedesktop/portal/desktop/request/" + sender[1:].replace(".", "_") + "/" + token
    invocation.return_value(GLib.Variant("(o)", (request,)))
    def response():
        code = {"success": 0, "cancel": 1, "error": 2}[mode]
        conn.emit_signal(sender, request, "org.freedesktop.portal.Request", "Response", GLib.Variant("(ua{sv})", (code, {})))
        print("PRINT_PORTAL_OK " + mode, flush=True)
        return False
    GLib.timeout_add(150, response)

connection.register_object("/org/freedesktop/portal/desktop", interface, method,
    lambda *args: GLib.Variant("u", 3), None)
Gio.bus_own_name_on_connection(connection, "org.freedesktop.portal.Desktop", Gio.BusNameOwnerFlags.NONE,
    lambda *args: print("PRINT_PORTAL_READY", flush=True), None)
GLib.MainLoop().run()
