/* mackes-launcher — xfce4-panel button that opens the Mackes popover.
 *
 * Single-button external panel plugin. Click → spawns
 * `mackes --popover` which renders the 420×600 slide-out at the
 * top-right of the primary monitor (mackes/workbench/popover/window.py).
 *
 * Plugin protocol — external mode (X-XFCE-Internal=false +
 * X-XFCE-Exec= line in the .desktop). We get --socket-id=N from the
 * panel; gtk_plug_new(socket_id) embeds our GtkButton into the
 * panel's GtkSocket. xfce4-panel 4.20 requires X-XFCE-API=2.0 in
 * the .desktop (covered by mackes-launcher.desktop).
 */
#include <gtk/gtk.h>
#include <gtk/gtkx.h>
#include <stdlib.h>
#include <string.h>


typedef struct {
    GtkWidget *button;
    GtkWidget *plug;
} MackesLauncher;


static void
on_button_clicked (GtkButton *btn, gpointer udata)
{
    (void) btn; (void) udata;
    /* Fire-and-forget. `mackes --popover` is idempotent: a second
     * call no-ops if the popover is already open (focus-out closes
     * it; this re-opens). */
    GError *err = NULL;
    gchar *argv[] = { (gchar *) "mackes", (gchar *) "--popover", NULL };
    if (!g_spawn_async (NULL, argv, NULL,
                        G_SPAWN_SEARCH_PATH | G_SPAWN_DO_NOT_REAP_CHILD,
                        NULL, NULL, NULL, &err)) {
        g_warning ("mackes-launcher: spawn failed: %s",
                   err ? err->message : "(unknown)");
        if (err) g_error_free (err);
    }
}


int main (int argc, char **argv)
{
    gtk_init (&argc, &argv);

    Window socket_id = 0;
    for (int i = 1; i < argc; i++) {
        if (g_str_has_prefix (argv[i], "--socket-id=")) {
            socket_id = (Window) g_ascii_strtoull (argv[i] + 12, NULL, 10);
        } else if (g_strcmp0 (argv[i], "--socket-id") == 0 && i + 1 < argc) {
            socket_id = (Window) g_ascii_strtoull (argv[++i], NULL, 10);
        }
    }

    GtkWidget *plug = gtk_plug_new (socket_id);
    GtkWidget *button = gtk_button_new ();
    GtkWidget *image = gtk_image_new_from_icon_name (
        "mackes-shell", GTK_ICON_SIZE_LARGE_TOOLBAR);
    gtk_button_set_image (GTK_BUTTON (button), image);
    gtk_button_set_relief (GTK_BUTTON (button), GTK_RELIEF_NONE);
    gtk_widget_set_tooltip_text (button,
        "Mackes Shell — click or press Super+M to open the popover");
    gtk_container_add (GTK_CONTAINER (plug), button);

    g_signal_connect (button, "clicked", G_CALLBACK (on_button_clicked), NULL);
    g_signal_connect (plug, "destroy", G_CALLBACK (gtk_main_quit), NULL);
    gtk_widget_show_all (plug);
    gtk_main ();
    return 0;
}
