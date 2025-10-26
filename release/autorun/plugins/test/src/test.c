#include <stdio.h>

extern void autorun_write(void *plugin, const char *path, const char *content,
                          size_t content_len);

extern void autorun_print(const char *msg);

void autorun_menu_init(void *plugin) {
  autorun_print("Test plugin menu initialized.\n");
}

void autorun_client_init(void *plugin) {
  autorun_write(plugin, "test.txt", "This is a test file.\n", 21);
}
