#include <string>
#include "rust/cxx.h"

namespace mln::bridge {
std::string rustSliceToString(const rust::Slice<const uint8_t>& slice);
}
