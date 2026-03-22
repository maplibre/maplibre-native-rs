#include "map_renderer.h"
#include "rust_log_observer.h"
#include "maplibre_native/src/renderer/bridge.rs.h"
#include <mbgl/util/logging.hpp>

namespace mln {
namespace bridge {

// Wrapper function for MapLibre's Log::useLogThread which takes optional parameters
// All severities are enabled
void Log_useLogThread(bool enable) {
    mbgl::Log::useLogThread(enable);
}

bool RustLogObserver::onRecord(mbgl::EventSeverity severity, mbgl::Event event, int64_t code, const std::string& msg) {
    // Call the Rust logging function through the CXX bridge
    log_from_cpp(severity, event, code, msg);
    return true;
}


} // namespace bridge
} // namespace mln
