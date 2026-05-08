#ifndef RESPONSE_H
#define RESPONSE_H

#include <mbgl/rust/http_response.hpp>

#include "rust/cxx.h"

#include <cstdint>
#include <memory>
#include <string>

namespace mln::bridge {

using Reason = mbgl::Response::Error::Reason;
using Kind = mbgl::Resource::Kind;

inline void http_response_set_data(mbgl::HttpResponse& response, rust::Slice<const std::uint8_t> data) {
	if (data.empty()) {
		return;
	}

	response.set_data(std::make_shared<const std::string>(reinterpret_cast<const char*>(data.data()), data.size()));
}

inline void http_response_set_etag(mbgl::HttpResponse& response, rust::Slice<const std::uint8_t> etag) {
	response.set_etag(std::string(reinterpret_cast<const char*>(etag.data()), etag.size()));
}

inline void http_response_set_no_content(mbgl::HttpResponse& response, bool no_content) {
	response.set_no_content(no_content);
}

inline void http_response_set_not_modified(mbgl::HttpResponse& response, bool not_modified) {
	response.set_not_modified(not_modified);
}

inline void http_response_set_error(mbgl::HttpResponse& response, Reason reason, rust::Str error_message) {
	response.set_error(reason, std::string(error_message));
}

} // namespace mln::bridge

#endif // RESPONSE_H