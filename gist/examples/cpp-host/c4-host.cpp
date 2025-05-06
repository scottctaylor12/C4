#include <extism.hpp>
#include <iostream>
#include <string>

int main(void) {
    const auto manifest =
        extism::Manifest::wasmURL("https://github.com/extism/plugins/releases/"
            "latest/download/count_vowels.wasm");
    extism::Plugin plugin(manifest, true);
    const std::string hello("Hello, World!");
    auto out = plugin.call("count_vowels", hello);
    std::string response(out.string());
    std::cout << response << std::endl;
}
