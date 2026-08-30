#pragma once

#include <mln/util/logging.hpp>
#include <mln/util/noncopyable.hpp>
#include <memory>
#include <string>

namespace mln {
namespace bridge {

class RustLogObserver : public mln::Log::Observer {
public:
    RustLogObserver() = default;
    ~RustLogObserver() override = default;

    bool onRecord(mln::EventSeverity severity, mln::Event event, int64_t code, const std::string& msg) override;

private:
    static uint32_t severityToU32(mln::EventSeverity severity);
    static uint32_t eventToU32(mln::Event event);
};

// Wrapper function for MapLibre's Log::useLogThread
void Log_useLogThread(bool enable);

} // namespace bridge
} // namespace mln
