#ifndef RESPONSE_H
#define RESPONSE_H

#include <mbgl/rust/http_response.hpp>

#include "rust/cxx.h"

#include <cstdint>
#include <memory>
#include <string>

using Reason = mbgl::Response::Error::Reason;

namespace mbgl {

inline void http_response_set_data(HttpResponse& response, rust::Slice<const std::uint8_t> data) {
	if (data.empty()) {
		return;
	}

	response.set_data(std::make_shared<const std::string>(reinterpret_cast<const char*>(data.data()), data.size()));
}

inline void http_response_set_etag(HttpResponse& response, rust::Slice<const std::uint8_t> etag) {
	response.set_etag(std::string(reinterpret_cast<const char*>(etag.data()), etag.size()));
}

inline void http_response_set_no_content(HttpResponse& response, bool no_content) {
	response.set_no_content(no_content);
}

inline void http_response_set_not_modified(HttpResponse& response, bool not_modified) {
	response.set_not_modified(not_modified);
}

inline void http_response_set_error(HttpResponse& response, Reason reason, rust::Str error_message) {
	response.set_error(reason, std::string(error_message));
}

} // namespace mbgl

#endif // RESPONSE_H