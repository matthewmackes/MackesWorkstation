# Makefile — build the Mackes launcher xfce4-panel external plugin.
# Mirrors data/panel-plugins/mackes-clipboard/Makefile so RPM build
# treatment is uniform.

CC          ?= gcc
PKG_CONFIG  ?= pkg-config
CFLAGS      ?= -O2 -g -Wall -Wextra -Wno-unused-parameter
LDFLAGS     ?=

PKGS        = gtk+-3.0 libxfce4panel-2.0 libxfce4util-1.0 gdk-x11-3.0
PKG_CFLAGS  := $(shell $(PKG_CONFIG) --cflags $(PKGS) 2>/dev/null)
PKG_LIBS    := $(shell $(PKG_CONFIG) --libs   $(PKGS) 2>/dev/null)

BIN         = mackes-launcher
SRC         = mackes-launcher.c

prefix      ?= /usr
libdir      ?= $(prefix)/lib
datadir     ?= $(prefix)/share

PLUGIN_LIBDIR     = $(libdir)/xfce4/panel/plugins
PLUGIN_DESKTOPDIR = $(datadir)/xfce4/panel/plugins

all: $(BIN)

$(BIN): $(SRC)
	$(CC) $(CFLAGS) $(PKG_CFLAGS) $(SRC) -o $(BIN) $(LDFLAGS) $(PKG_LIBS)

install: $(BIN)
	install -D -m 0755 $(BIN) $(DESTDIR)$(PLUGIN_LIBDIR)/$(BIN)
	install -D -m 0644 $(BIN).desktop \
	    $(DESTDIR)$(PLUGIN_DESKTOPDIR)/$(BIN).desktop

uninstall:
	rm -f $(DESTDIR)$(PLUGIN_LIBDIR)/$(BIN)
	rm -f $(DESTDIR)$(PLUGIN_DESKTOPDIR)/$(BIN).desktop

clean:
	rm -f $(BIN) *.o

.PHONY: all install uninstall clean
