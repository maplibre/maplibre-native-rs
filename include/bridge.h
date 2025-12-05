#pragma once

#include <cstdint>

namespace mln {
namespace bridge {

struct VoidTrampoline {
    void call() {
        if (trampoline_function) {
            trampoline_function(function_pointer);
        }
    }
private:
    // void* cannot be used, because not supported by cxx
    void(*trampoline_function)(std::int8_t* function_pointer){nullptr};
    std::int8_t* function_pointer{nullptr};
};

} // namespace bridge
} // namespace mln
