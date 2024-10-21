int main (string[] argv) {
    var app = new Gtk.Application ("net.arthurclho.LanKM",
                                   GLib.ApplicationFlags.DEFAULT_FLAGS);

    app.activate.connect(() => {
        var window = new ApplicationWindow (app);

        window.show_all ();
    });
    
    return app.run (argv);
}
