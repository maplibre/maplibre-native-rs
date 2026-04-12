#include "util.h"

namespace mln::bridge {
std::string rustSliceToString(const rust::Slice<const uint8_t>& slice) {
    return std::string(reinterpret_cast<const char*>(slice.data()), slice.size());
}
}
