
[GtkTemplate ( ui = "/net/arthurclho/lankm/ui/application-window.ui" )]
public class ApplicationWindow : Gtk.ApplicationWindow {
    [GtkChild]
    private unowned Gtk.Notebook notebook;
    
    public ApplicationWindow (Gtk.Application app) {
        Object ( application: app );

        // Server Tab
        var server_page = new ServerPage ();
        var server_tab = new Gtk.Label ("Server");
        notebook.append_page (server_page, server_tab);

        // Client Tab
        var client_page = new Gtk.Label ("Not implemented");
        var client_tab = new Gtk.Label ("Client");
        notebook.append_page (client_page, client_tab);
    }

    
}
