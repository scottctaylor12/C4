#include <extism.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void print_plugin_output(ExtismPlugin *plugin, int32_t rc){
  if (rc != EXTISM_SUCCESS) {
    fprintf(stderr, "ERROR: %s\n", extism_plugin_error(plugin));
    return;
  }

  ExtismSize outlen = extism_plugin_output_length(plugin);
  const uint8_t *out = extism_plugin_output_data(plugin);
  fwrite(out, 1, outlen, stdout);
}

int main(void) {
  //FIXME: get formatting correct for manifest, currently broken
  const char *manifest = 
    "{"
        " \"wasm\": [{\"url\": \"https://github.com/extism/plugins/releases/latest/download/count_vowels.wasm\"}], "
        " \"allowed_hosts\": \"{\"*\"}\", "
    "}";

  char *errmsg = NULL;
  ExtismPlugin *plugin = extism_plugin_new(
      (const uint8_t *)manifest, strlen(manifest), NULL, 0, true, &errmsg);
  if (plugin == NULL) {
    fprintf(stderr, "ERROR: %s\n", errmsg);
    extism_plugin_new_error_free(errmsg);
    exit(1);
  }

  const char *input = "Hello, world!";
  print_plugin_output(plugin, extism_plugin_call(plugin, "count_vowels",
                                  (const uint8_t *)input, strlen(input)));
  extism_plugin_free(plugin);
  return 0;
}