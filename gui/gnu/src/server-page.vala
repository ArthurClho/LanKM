using Posix;
using Linux.Network;

[GtkTemplate ( ui = "/net/arthurclho/lankm/ui/server-page.ui" )]
public class ServerPage : Gtk.Box {
    [GtkChild]
    private unowned Gtk.TextView server_log;
    [GtkChild]
    private unowned Gtk.Label server_status;
    [GtkChild]
    private unowned Gtk.Button start_stop_button;
    [GtkChild]
    private unowned Gtk.Label this_ip_label;
    
    private Gtk.TextBuffer text_buffer;

    private GLib.Subprocess? child_process = null;
    private string lankm_headless_path;

    public ServerPage () {
        text_buffer = new Gtk.TextBuffer (null);
        server_log.set_buffer (text_buffer);

        start_stop_button.button_press_event.connect (on_start_stop_button);

        update_button_text ();
        this_ip_label.label = "This machine's IP Addresses: " + get_local_ips();

        try {
            var s = FileUtils.read_link ("/proc/self/exe");
            lankm_headless_path = Path.get_dirname (s) + "/lankm-headless";
        } catch (FileError e) {
            print ("Error reading /proc/self/exe link: %s\n", (string) e);
            lankm_headless_path = "";
        }
    }

    static string get_local_ips() {
        Linux.Network.IfAddrs addrs;
        Linux.Network.getifaddrs(out addrs);
        
        var builder = new StringBuilder ();

        for (unowned Linux.Network.IfAddrs it = addrs;
             it != null;
             it = it.ifa_next) {
            if (it.ifa_addr == null) {
                continue;
            }
            unowned var addr = it.ifa_addr;
            if (addr.sa_family != Posix.AF_INET) {
                continue;
            }

            char node[256];
            char service[0];
            var ret = Posix.getnameinfo(addr, (Posix.socklen_t) sizeof(Posix.SockAddrIn),
                                        node, service,
                                        Posix.NI_NUMERICHOST);

            if (ret != 0) {
                print ("getnameinfo failed\n");
                continue;
            }

            builder.append_printf ("%s; ", (string) node);
        }

        return builder.free_and_steal ();
    }

    void update_button_text() {
        if (child_process == null) {
            start_stop_button.label = "Start";

            // TODO: Display "Crashed" in red if the child exits
            // abnormally or with exit code != 0  
            server_status.label = "Stopped";
        } else {
            start_stop_button.label = "Stop";
            server_status.label = "Running";
        }
    }

    bool on_start_stop_button() {
        if (child_process == null) {
            start_server();
        } else {
            stop_server();
        }

        update_button_text();
        return true;
    }

    void log(string message) {
        Gtk.TextIter iter;
        text_buffer.get_end_iter (out iter);
        text_buffer.insert (ref iter, message, message.length);

        text_buffer.get_end_iter (out iter);
        text_buffer.insert (ref iter, "\n", 1);

        text_buffer.get_end_iter (out iter);
        var mark = text_buffer.create_mark (null, iter, false);
        server_log.scroll_to_mark (mark, 0.0, true, 0.0, 0.0);
    }

    async void log_update() {
        while (child_process != null) {
            var stdout = child_process.get_stdout_pipe ();

            uint8 buffer[256];
            ssize_t count = 0;
            try {
                count = yield stdout.read_async (buffer);
            } catch (Error e) {
                print ("read_async error: %s\n", (string)e);
            }

            if (count == 0) {
                log ("Process exited with code " + child_process.get_status().to_string());
                child_process = null;
                update_button_text();
                break;
            }
            
            var builder = new StringBuilder ();
            builder.append_printf ("%.*s", count, buffer);
            var s = builder.free_and_steal ();

            log(s);
        }
    }

    void start_server() {
        var flags = GLib.SubprocessFlags.STDOUT_PIPE
                  | GLib.SubprocessFlags.STDERR_MERGE;

        try {
            // Placeholder ping to test the server_log scrolling
            log ("Starting " + lankm_headless_path);
            child_process = new GLib.Subprocess (flags, lankm_headless_path);
            log_update.begin();
        } catch (Error e) {
            log ("Error spawning child process: " + (string)e);
        }
    }

    void stop_server() {
        if (child_process == null) {
            return;
        }

        child_process.send_signal (Posix.Signal.TERM);
    }
}
