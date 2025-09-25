#pragma once

#include <mbgl/util/logging.hpp>
#include <mbgl/util/noncopyable.hpp>
#include <memory>
#include <string>

namespace mln {
namespace bridge {

class RustLogObserver : public mbgl::Log::Observer {
public:
    RustLogObserver() = default;
    ~RustLogObserver() override = default;

    bool onRecord(mbgl::EventSeverity severity, mbgl::Event event, int64_t code, const std::string& msg) override;

private:
    static uint32_t severityToU32(mbgl::EventSeverity severity);
    static uint32_t eventToU32(mbgl::Event event);
};

// Wrapper function for MapLibre's Log::useLogThread
void Log_useLogThread(bool enable);

} // namespace bridge
} // namespace mln
