#include "rust_log_observer.h"
#include "maplibre_native/src/bridge.rs.h"
#include <mln/util/logging.hpp>

namespace mln {
namespace bridge {

// Wrapper function for MapLibre's Log::useLogThread which takes optional parameters
// All severities are enabled
void Log_useLogThread(bool enable) {
    mln::Log::useLogThread(enable);
}

bool RustLogObserver::onRecord(mln::EventSeverity severity, mln::Event event, int64_t code, const std::string& msg) {
    // Call the Rust logging function through the CXX bridge
    log_from_cpp(severity, event, code, msg);
    return true;
}

} // namespace bridge
} // namespace mln
